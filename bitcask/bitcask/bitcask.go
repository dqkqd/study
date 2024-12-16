package bitcask

import (
	"fmt"
	"strings"
)

type BitCaskCommandType int

const (
	SetCommand BitCaskCommandType = iota
	GetCommand
)

type BitCaskCommand struct {
	key     string
	value   string
	cmdType BitCaskCommandType
}

func ParseCommand(input string) (BitCaskCommand, error) {
	s := strings.Split(input, " ")
	for i := range s {
		s[i] = strings.TrimSpace(s[i])
	}

	invalid := func(msg string) (BitCaskCommand, error) {
		return BitCaskCommand{"", "", SetCommand}, fmt.Errorf("%s. Got `%s`", msg, input)
	}

	if len(s) == 0 {
		return invalid("Invalid command")
	}

	switch strings.ToLower(s[0]) {
	case "set":
		if len(s) < 3 {
			return invalid("Expected `set <key> <value>`")
		}
		return BitCaskCommand{s[1], strings.Join(s[2:], " "), SetCommand}, nil
	case "get":
		if len(s) != 2 {
			return invalid("Expected `get <key>`")
		}
		return BitCaskCommand{s[1], "", GetCommand}, nil
	default:
		return invalid("Expected `set` or `get` command")
	}
}
