package bitcask

import (
	"testing"
)

func TestSuccess(t *testing.T) {
	testCases := []struct {
		input string
		want  Command
	}{
		{"set 1 2", Command{key: "1", value: "2", cmdType: SetCommand}},
		{"set 2 3", Command{key: "2", value: "3", cmdType: SetCommand}},
		{"get 1", Command{key: "1", cmdType: GetCommand}},
		{"get 2", Command{key: "2", cmdType: GetCommand}},
	}

	for _, tc := range testCases {
		cmd, err := ParseCommand(tc.input)
		if err != nil {
			t.Error("Expected to success")
		}

		if cmd != tc.want {
			t.Errorf("Expected %s, got %s", tc.want, cmd)
		}
	}
}

func TestFail(t *testing.T) {
	testCases := []string{
		"get",
		"set",
		"get 1 2",
		"set 1",
		"abc",
	}

	for _, tc := range testCases {
		_, err := ParseCommand(tc)
		if err == nil {
			t.Error("Expected error")
		}
	}
}
