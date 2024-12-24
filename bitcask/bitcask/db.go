package bitcask

import (
	"context"
	"fmt"
	"sync"
	"time"
)

func (db *Database) HandleQuery(query string) {
	cmd, err := ParseCommand(query)
	if err != nil {
		fmt.Println(err)
		return
	}

	switch cmd.cmdType {
	case GetCommand:
		value, err := db.Get(cmd)
		if err == nil {
			fmt.Println(value)
		}

	case SetCommand:
		err := db.Set(cmd)
		if err != nil {
			fmt.Println(err)
		}

	case DeleteCommand:
		err := db.Delete(cmd)
		if err != nil {
			fmt.Println(err)
		}
	}
}

type Config struct {
	MergeFrequency    uint64 // how long should merge be performed
	DatafileThreshold uint32 // how big a datafile should be
	NumReadonlyFiles  uint16 // how many readonly files in disk before merging
	EnableAutoMerge   bool   // should merge is enabled (this is only used in test)
}

type Database struct {
	dir            *Directory
	keydir         Keydir
	cancelMerge    context.CancelFunc
	done           chan bool
	folder         string
	activeDatafile ActiveDatafile
	cfg            Config
	mu             sync.RWMutex
}

func DefaultDatabaseConfig() Config {
	return Config{
		DatafileThreshold: 1<<16 - 1,
		NumReadonlyFiles:  100,
		MergeFrequency:    5000,
		EnableAutoMerge:   true,
	}
}

func OpenDatabase(folder string, cfg Config) (db *Database, err error) {
	dir, err := openDirectory(folder)
	if err != nil {
		return db, err
	}

	d, err := dir.activeDatafile()
	if err != nil {
		return db, err
	}

	kd, err := openKeydir(dir)
	if err != nil {
		return db, err
	}

	ctx, cancelFunc := context.WithCancel(context.Background())

	db = &Database{&dir, kd, cancelFunc, make(chan bool), folder, d, cfg, sync.RWMutex{}}

	go func(c context.Context) {
		if cfg.EnableAutoMerge {
			for {
				time.Sleep(time.Duration(100 * cfg.MergeFrequency))

				select {
				case <-ctx.Done():
					db.done <- true
					return
				default:
					if db.shouldMerge() {
						db.merge()
					}
				}
			}
		}
	}(ctx)

	return db, nil
}

func (db *Database) Close() {
	if db.cfg.EnableAutoMerge {
		db.cancelMerge()
		<-db.done
	}
}

func (db *Database) Set(cmd Command) error {
	if cmd.cmdType != SetCommand {
		panic("Expected set command")
	}

	db.mu.Lock()
	defer db.mu.Unlock()

	if db.shouldRollover() {
		db.rollover()
	}

	// first, save to active file
	loc, err := db.activeDatafile.Save(cmd.key, cmd.value)
	if err != nil {
		return err
	}

	// then save to keydir
	db.keydir[cmd.key] = loc

	return nil
}

func (db *Database) Delete(cmd Command) error {
	if cmd.cmdType != DeleteCommand {
		panic("Expected delete command")
	}
	return db.Set(Command{key: cmd.key, value: TOMBSTONE, cmdType: SetCommand})
}

func (db *Database) Get(cmd Command) (value string, err error) {
	if cmd.cmdType != GetCommand {
		panic("Expected get command")
	}

	db.mu.RLock()
	defer db.mu.RUnlock()

	// get the key's position from keydir
	loc, ok := db.keydir[cmd.key]
	if !ok {
		return value, fmt.Errorf("Not existed key %s", cmd.key)
	}

	var record Record

	if db.activeDatafile.id == loc.datafileId {
		// This key is in active file, we can get it without opening new files
		record, err = db.activeDatafile.Get(loc)
	} else {
		// This key is in other files, need to open and read it
		rd, err := db.dir.readonlyDatafile(loc.datafileId)
		if err != nil {
			return value, err
		}
		record, err = rd.Get(loc)
	}

	if err != nil {
		return value, err
	}

	value = string(record.value)
	if value == TOMBSTONE {
		return value, fmt.Errorf("Deleted key %s", cmd.key)
	}

	return value, nil
}

func (db *Database) shouldMerge() bool {
	return len(db.dir.readonlyDatafileIds) > int(db.cfg.NumReadonlyFiles)
}

func (db *Database) merge() error {
	db.mu.Lock()
	defer db.mu.Unlock()

	records, err := GetReadonlyRecordsFromDirectory(*db.dir)
	if err != nil {
		return err
	}

	keydir := Keydir{}
	deletedRecords := map[string]Record{}

	datafileId := INVALID_DATAFILE_ID

	// we want the active datafile to be garbage collected after this block
	// so subsequent read does not cause error
	{
		d, err := db.dir.tempActiveDatafile()
		datafileId = d.id
		if err != nil {
			return err
		}

		pos := uint32(0)
		for _, rl := range records {
			// skip deleted records
			if rl.r.deleted() {
				deletedRecords[string(rl.r.key)] = rl.r
				continue
			}

			err = saveRecord(d.f, rl.r)
			if err != nil {
				return err
			}
			keydir[string(rl.r.key)] = RecordLoc{d.id, rl.loc.sz, pos, rl.loc.tstamp}
			pos += rl.loc.sz
		}
	}

	if datafileId == INVALID_DATAFILE_ID {
		panic("Invalid datafile id after set")
	}

	// all the records are now transfered, add it to the new keydir and delete the old files

	// delete record
	for k, r := range deletedRecords {
		existingRecord, ok := db.keydir[k]
		// only touch key that exists in keydir that have the same timestamp
		if ok && existingRecord.tstamp == r.tstamp {
			delete(db.keydir, k)
		}
	}

	// transfer new record location
	for k, r := range keydir {
		existingRecord, ok := db.keydir[k]
		// only merge key that exists in keydir that have the same timestamp
		if ok && existingRecord.tstamp == r.tstamp {
			db.keydir[k] = r
		}
	}

	// set the new datafileId as read
	db.dir.readonlyDatafileIds[datafileId] = true
	// remove old datafile ids
	readonlyDatafileIds := make([]DatafileId, 0, len(db.dir.readonlyDatafileIds))
	for id := range db.dir.readonlyDatafileIds {
		readonlyDatafileIds = append(readonlyDatafileIds, id)
	}
	for _, id := range readonlyDatafileIds {
		if id != datafileId {
			db.dir.removeReadonlyDatafile(id)
		}
	}

	return nil
}

func (db *Database) shouldRollover() bool {
	return db.activeDatafile.sz >= db.cfg.DatafileThreshold
}

func (db *Database) rollover() error {
	d, err := db.dir.nextActiveDatafile()
	if err != nil {
		return err
	}
	db.activeDatafile = d
	return nil
}
