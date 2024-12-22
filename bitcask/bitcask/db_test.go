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

func shouldHaveTotalFiles(t *testing.T, folder string, expected int) {
	files, err := os.ReadDir(folder)
	if err != nil {
		t.Error(err)
	}
	if len(files) != expected {
		t.Errorf("Expect %d files, has %d", expected, len(files))
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

	shouldHaveTotalFiles(t, dbfolder, 1)
	checkSetKey(t, db, "key1", "value1")
	shouldHaveTotalFiles(t, dbfolder, 1)
	checkSetKey(t, db, "key2", "value2")
	shouldHaveTotalFiles(t, dbfolder, 2)
	checkSetKey(t, db, "key3", "value3")
	shouldHaveTotalFiles(t, dbfolder, 3)
	checkSetKey(t, db, "key4", "value4")
	shouldHaveTotalFiles(t, dbfolder, 4)

	// should be able to get rolled over values
	checkGetKey(t, db, "key1", "value1")
	checkGetKey(t, db, "key2", "value2")
	checkGetKey(t, db, "key3", "value3")
	checkGetKey(t, db, "key4", "value4")
}

func TestDbReopen(t *testing.T) {
	dir := t.TempDir()
	dbfolder := fmt.Sprintf("%s/%s", dir, "testdb")

	cfg := DefaultDatabaseConfig()
	cfg.DatafileThreshold = 1 // always rollover

	{
		db, err := OpenDatabase(dbfolder, cfg)
		if err != nil {
			t.Error(err)
		}
		checkSetKey(t, db, "key1", "value1")
		checkSetKey(t, db, "key2", "value2")
		checkSetKey(t, db, "key3", "value3")
		checkSetKey(t, db, "key4", "value4")
		checkGetKey(t, db, "key1", "value1")
		checkGetKey(t, db, "key2", "value2")
		checkGetKey(t, db, "key3", "value3")
		checkGetKey(t, db, "key4", "value4")
	}

	{
		// re-open, still get the same keys
		db, err := OpenDatabase(dbfolder, cfg)
		if err != nil {
			t.Error(err)
		}
		checkGetKey(t, db, "key1", "value1")
		checkGetKey(t, db, "key2", "value2")
		checkGetKey(t, db, "key3", "value3")
		checkGetKey(t, db, "key4", "value4")
		// set new keys
		checkSetKey(t, db, "key1", "new value1")
	}

	{
		// re-open, can get the same keys and new key
		db, err := OpenDatabase(dbfolder, cfg)
		if err != nil {
			t.Error(err)
		}
		checkGetKey(t, db, "key1", "new value1")
		checkGetKey(t, db, "key2", "value2")
		checkGetKey(t, db, "key3", "value3")
		checkGetKey(t, db, "key4", "value4")
	}
}

func TestDbMerge(t *testing.T) {
	dir := t.TempDir()
	dbfolder := fmt.Sprintf("%s/%s", dir, "testdb")

	cfg := DefaultDatabaseConfig()
	cfg.DatafileThreshold = 1 // always rollover

	totalKeys := 50
	store := make([]struct{ key, value string }, totalKeys)
	for i := range totalKeys {
		store[i].key = fmt.Sprintf("key%d", i)
		store[i].value = fmt.Sprintf("value%d", i)
	}

	{
		db, err := OpenDatabase(dbfolder, cfg)
		if err != nil {
			t.Error(err)
		}

		for _, s := range store {
			checkSetKey(t, db, s.key, s.value)
		}
		shouldHaveTotalFiles(t, dbfolder, totalKeys)

		db.merge()
		// only have 2 files now, one is active, one is merged
		shouldHaveTotalFiles(t, dbfolder, 2)

		// saved key should be intact
		for _, s := range store {
			checkGetKey(t, db, s.key, s.value)
		}
	}

	{
		// re-open again
		db, err := OpenDatabase(dbfolder, cfg)
		if err != nil {
			t.Error(err)
		}
		// saved key should be intact
		for _, s := range store {
			checkGetKey(t, db, s.key, s.value)
		}
	}
}
