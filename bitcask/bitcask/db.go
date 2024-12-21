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
	err := db.keydir.Save(db.activeDatafile, cmd.key, cmd.value)
	return err
}

func (db Database) Get(cmd Command) (value string, err error) {
	if cmd.cmdType != GetCommand {
		panic("Expected get command")
	}
	record, err := db.keydir.Get(db.activeDatafile, cmd.key)
	return string(record.value), err
}
