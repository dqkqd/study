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

type Database struct {
	activeDatafile ActiveDatafile
	keydir         Keydir
	folder         string
}

func OpenDatabase(folder string) (db Database, err error) {
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

	return Database{d, kd, folder}, nil
}

func (db Database) Set(cmd Command) error {
	if cmd.cmdType != SetCommand {
		panic("Expected set command")
	}

	// first, save to active file
	pos, sz, err := db.activeDatafile.Save(cmd.key, cmd.value)
	if err != nil {
		return err
	}

	// then save to keydir
	// TODO: handle tstamp
	db.keydir[cmd.key] = ValuePos{db.activeDatafile.id, sz, pos, 0}

	return nil
}

func (db Database) Get(cmd Command) (value string, err error) {
	if cmd.cmdType != GetCommand {
		panic("Expected get command")
	}

	// get the key's position from keydir
	vp, ok := db.keydir[cmd.key]
	if !ok {
		return value, fmt.Errorf("Not existed key %s", cmd.key)
	}

	var record Record

	if db.activeDatafile.id == vp.fid {
		// This key is in active file, we can get it without opening new files
		record, err = db.activeDatafile.Get(vp.valuepos, vp.valuesz)
	} else {
		// This key is in other files, need to open and read it
		// TODO: cover test for this
		rd := ReadonlyDatafile{&db.folder, vp.fid}
		record, err = rd.Get(vp.valuepos, vp.valuesz)
	}

	return string(record.value), err
}
