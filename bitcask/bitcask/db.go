package bitcask

import (
	"fmt"
	"os"
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
	activeDatafile ActiveDatafile
	keydir         Keydir
	folder         string
	cfg            Config
}

func DefaultDatabaseConfig() Config {
	return Config{
		DatafileThreshold: 1<<16 - 1,
	}
}

func OpenDatabase(folder string, cfg Config) (db *Database, err error) {
	err = os.MkdirAll(folder, 0700)
	if err != nil {
		return db, err
	}

	// TODO: to not assign id = 1
	d, err := OpenAsActiveDatafile(&folder, 1)
	if err != nil {
		return db, err
	}

	kd := OpenKeydir()

	return &Database{d, kd, folder, cfg}, nil
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
		// TODO: cover test for this
		rd := ReadonlyDatafile{&db.folder, loc.datafileId}
		record, err = rd.Get(loc)
	}

	return string(record.value), err
}

func (db Database) shouldRollover() bool {
	return db.activeDatafile.sz >= db.cfg.DatafileThreshold
}

func (db *Database) rollover() error {
	nextId := db.activeDatafile.id + 1
	d, err := OpenAsActiveDatafile(&db.folder, nextId)
	if err != nil {
		return err
	}
	db.activeDatafile = d
	return nil
}
