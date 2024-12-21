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
