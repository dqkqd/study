package bitcask

import (
	"fmt"
	"os"
	"testing"
)

func checkSetKey(t *testing.T, db *Database, key string, value string) {
	cmd := Command{key, value, SetCommand}
	err := db.Set(cmd)
	if err != nil {
		t.Errorf("Cannot set %s, error: %s", cmd, err)
	}
}

func checkGetKey(t *testing.T, db *Database, key string, expected string) {
	value, err := db.Get(Command{key: key, cmdType: GetCommand})
	if err != nil {
		t.Error(err)
	}
	if value != expected {
		t.Errorf("Got %s, want `%s`", value, expected)
	}
}

func TestDbQuery(t *testing.T) {
	dir := t.TempDir()
	dbfolder := fmt.Sprintf("%s/%s", dir, "testdb")
	db, err := OpenDatabase(dbfolder, DefaultDatabaseConfig())
	if err != nil {
		t.Error(err)
	}

	cmd := Command{key: "1", cmdType: GetCommand}
	_, err = db.Get(cmd)
	if err == nil {
		t.Errorf("cmd %s must fail", cmd)
	}

	checkSetKey(t, db, "1", "2")
	checkGetKey(t, db, "1", "2")
}

func TestDbGet(t *testing.T) {
	dir := t.TempDir()
	dbfolder := fmt.Sprintf("%s/%s", dir, "testdb")
	db, err := OpenDatabase(dbfolder, DefaultDatabaseConfig())
	if err != nil {
		t.Error(err)
	}

	testcases := []struct {
		key, value string
	}{
		{"key1", "value1"},
		{"key2", "value2"},
		{"key3", "value3"},
		{"key4", "value4"},
		{"key5", "value5"},
		{"this is a long key", "this is a long value"},
	}

	for _, tc := range testcases {
		checkSetKey(t, db, tc.key, tc.value)
	}

	for _, tc := range testcases {
		checkGetKey(t, db, tc.key, tc.value)
	}
}

func TestDbGetOverwrite(t *testing.T) {
	dir := t.TempDir()
	dbfolder := fmt.Sprintf("%s/%s", dir, "testdb")
	db, err := OpenDatabase(dbfolder, DefaultDatabaseConfig())
	if err != nil {
		t.Error(err)
	}

	checkSetKey(t, db, "key", "value")
	checkGetKey(t, db, "key", "value")

	checkSetKey(t, db, "key", "new value")
	checkGetKey(t, db, "key", "new value")
}

func TestDbRollover(t *testing.T) {
	dir := t.TempDir()
	dbfolder := fmt.Sprintf("%s/%s", dir, "testdb")

	cfg := DefaultDatabaseConfig()
	cfg.DatafileThreshold = 1 // always rollover

	db, err := OpenDatabase(dbfolder, cfg)
	if err != nil {
		t.Error(err)
	}

	shouldHaveTotalFiles := func(expected int) {
		files, err := os.ReadDir(dbfolder)
		if err != nil {
			t.Error(err)
		}
		if len(files) != expected {
			t.Errorf("Expect %d datafile, has %d", expected, len(files))
		}
	}

	shouldHaveTotalFiles(1)
	checkSetKey(t, db, "key1", "value1")
	shouldHaveTotalFiles(1)
	checkSetKey(t, db, "key2", "value2")
	shouldHaveTotalFiles(2)
	checkSetKey(t, db, "key3", "value3")
	shouldHaveTotalFiles(3)
	checkSetKey(t, db, "key4", "value4")
	shouldHaveTotalFiles(4)

	// should be able to get rolled over values
	checkGetKey(t, db, "key1", "value1")
	checkGetKey(t, db, "key2", "value2")
	checkGetKey(t, db, "key3", "value3")
	checkGetKey(t, db, "key4", "value4")
}

