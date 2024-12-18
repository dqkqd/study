package bitcask

import (
	"fmt"
	"strings"
)

type CommandType int

const (
	SetCommand CommandType = iota
	GetCommand
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
	default:
		panic("Unimplemented")
	}
}

func ParseCommand(input string) (Command, error) {
	s := strings.Split(input, " ")
	for i := range s {
		s[i] = strings.TrimSpace(s[i])
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
	default:
		return invalid("Expected `set` or `get` command")
	}
}
