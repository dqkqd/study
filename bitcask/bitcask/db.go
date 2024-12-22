package bitcask

import (
	"fmt"
)

func (db Database) HandleQuery(query string) {
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

	}
}

type Config struct {
	DatafileThreshold uint32
}

type Database struct {
	dir            *Directory
	keydir         Keydir
	folder         string
	activeDatafile ActiveDatafile
	cfg            Config
}

func DefaultDatabaseConfig() Config {
	return Config{
		DatafileThreshold: 1<<16 - 1,
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

	return &Database{&dir, kd, folder, d, cfg}, nil
}

func (db *Database) Set(cmd Command) error {
	if cmd.cmdType != SetCommand {
		panic("Expected set command")
	}

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

func (db Database) Get(cmd Command) (value string, err error) {
	if cmd.cmdType != GetCommand {
		panic("Expected get command")
	}

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

	return string(record.value), err
}

func (db *Database) merge() error {
	records, err := GetAllRecordsFromDirectory(*db.dir)
	if err != nil {
		return err
	}

	keydir := Keydir{}
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
	// first, make sure the datafile id is added to read ids before performing merge
	db.dir.readonlyDatafileIds[datafileId] = true
	for k, r := range keydir {
		existingRecord, ok := db.keydir[k]
		// same record, switch location
		if ok && existingRecord.tstamp == r.tstamp {
			db.keydir[k] = r
		}
	}

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

func (db Database) shouldRollover() bool {
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
