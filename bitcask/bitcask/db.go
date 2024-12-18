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

type InMemoryStorage map[string]string

type Database struct {
	mem InMemoryStorage
}

func NewDatabase() Database {
	return Database{InMemoryStorage{}}
}

func (db Database) Set(cmd Command) error {
	if cmd.cmdType != SetCommand {
		panic("Expected set command")
	}
	db.mem[cmd.key] = cmd.value
	return nil
}

func (db Database) Get(cmd Command) (value string, err error) {
	if cmd.cmdType != GetCommand {
		panic("Expected get command")
	}
	value, ok := db.mem[cmd.key]
	if ok {
		return value, nil
	}
	// TODO: get from disk
	return value, fmt.Errorf("Key `%s` not existed", cmd.key)
}
