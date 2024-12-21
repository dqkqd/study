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
	keydir Keydir
	folder string
}

func OpenDatabase(folder string) (db Database, err error) {
	err = os.MkdirAll(folder, 0700)
	if err != nil {
		return db, err
	}
	return Database{OpenKeydir(&folder), folder}, nil
}

func (db Database) Set(cmd Command) error {
	if cmd.cmdType != SetCommand {
		panic("Expected set command")
	}
	err := db.keydir.Save(cmd.key, cmd.value, 1)
	return err
}

func (db Database) Get(cmd Command) (value string, err error) {
	if cmd.cmdType != GetCommand {
		panic("Expected get command")
	}
	record, err := db.keydir.Get(cmd.key)
	return string(record.value), err
}
