package bitcask

import (
	"fmt"
	"testing"
)

func TestDbQuery(t *testing.T) {
	dir := t.TempDir()
	dbfolder := fmt.Sprintf("%s/%s", dir, "testdb")
	db, err := OpenDatabase(dbfolder)
	if err != nil {
		t.Errorf("Cannot open db %s", err)
	}

	var cmd Command

	cmd = Command{key: "1", cmdType: GetCommand}
	_, err = db.Get(cmd)
	if err == nil {
		t.Errorf("cmd %s must fail", cmd)
	}

	cmd = Command{key: "1", value: "2", cmdType: SetCommand}
	err = db.Set(cmd)
	if err != nil {
		t.Errorf("Cannot set %s, error: %s", cmd, err)
	}

	cmd = Command{key: "1", cmdType: GetCommand}
	v, err := db.Get(cmd)
	if err != nil {
		t.Errorf("Cannot get %s, error: %s", cmd, err)
	}
	if v != "2" {
		t.Errorf("Got %s, want 2", v)
	}
}

func TestDbGet(t *testing.T) {
	dir := t.TempDir()
	dbfolder := fmt.Sprintf("%s/%s", dir, "testdb")
	db, err := OpenDatabase(dbfolder)
	if err != nil {
		t.Errorf("Cannot open db %s", err)
	}

	testcases := []Command{
		{"key1", "value1", SetCommand},
		{"key2", "value2", SetCommand},
		{"key3", "value3", SetCommand},
		{"key4", "value4", SetCommand},
		{"key5", "value5", SetCommand},
		{"this is a long key", "this is a long value", SetCommand},
	}

	for _, tc := range testcases {
		err := db.Set(tc)
		if err != nil {
			t.Errorf("Cannot save record: %s", err)
		}
	}

	for _, tc := range testcases {
		value, err := db.Get(Command{key: tc.key, cmdType: GetCommand})
		if err != nil {
			t.Errorf("Cannot get key %s, error: %s", tc.key, err)
		}
		if value != tc.value {
			t.Errorf("Got %s, want %s", value, tc.value)
		}
	}
}

func TestDbGetOverwrite(t *testing.T) {
	dir := t.TempDir()
	dbfolder := fmt.Sprintf("%s/%s", dir, "testdb")
	db, err := OpenDatabase(dbfolder)
	if err != nil {
		t.Error(err)
	}

	err = db.Set(Command{"key", "value", SetCommand})
	if err != nil {
		t.Error(err)
	}

	value, err := db.Get(Command{key: "key", cmdType: GetCommand})
	if err != nil {
		t.Error(err)
	}
	if value != "value" {
		t.Errorf("Got %s, want `value`", value)
	}

	err = db.Set(Command{"key", "new value", SetCommand})
	if err != nil {
		t.Error(err)
	}

	value, err = db.Get(Command{key: "key", cmdType: GetCommand})
	if err != nil {
		t.Error(err)
	}
	if value != "new value" {
		t.Errorf("Got %s, want `new value`", value)
	}
}
