package bitcask

import (
	"fmt"
	"strings"
)

type CommandType int

const (
	SetCommand CommandType = iota
	GetCommand
	DeleteCommand
)

type Command struct {
	key     string
	value   string
	cmdType CommandType
}

func (c Command) String() string {
	switch c.cmdType {
	case GetCommand:
		return fmt.Sprintf("Get(%s)", c.key)
	case SetCommand:
		return fmt.Sprintf("Set(%s,%s)", c.key, c.value)
	case DeleteCommand:
		return fmt.Sprintf("Delete(%s)", c.key)
	default:
		panic("Unimplemented")
	}
}

func ParseCommand(input string) (Command, error) {
	s := []string{}
	for _, c := range strings.Split(input, " ") {
		c = strings.TrimSpace(c)
		if len(c) > 0 {
			s = append(s, c)
		}
	}

	invalid := func(msg string) (Command, error) {
		return Command{"", "", SetCommand}, fmt.Errorf("%s. Got `%s`", msg, input)
	}

	if len(s) == 0 {
		return invalid("Invalid command")
	}

	switch strings.ToLower(s[0]) {
	case "set":
		if len(s) < 3 {
			return invalid("Expected `set <key> <value>`")
		}
		return Command{s[1], strings.Join(s[2:], " "), SetCommand}, nil
	case "get":
		if len(s) != 2 {
			return invalid("Expected `get <key>`")
		}
		return Command{s[1], "", GetCommand}, nil
	case "delete":
		if len(s) != 2 {
			return invalid("Expected `delete <key>`")
		}
		return Command{s[1], "", DeleteCommand}, nil
	default:
		return invalid("Expected `set`, `get`, or `delete` command")
	}
}
