package bitcask

import "testing"

func TestGet(t *testing.T) {
	db := NewDatabase()
	var cmd Command

	cmd = Command{key: "1", cmdType: GetCommand}
	_, err := db.Get(cmd)
	if err == nil {
		t.Errorf("cmd %s must fail", cmd)
	}

	cmd = Command{key: "1", value: "2", cmdType: SetCommand}
	err = db.Set(cmd)
	if err != nil {
		t.Errorf("Cannot set %s", cmd)
	}

	cmd = Command{key: "1", cmdType: GetCommand}
	v, err := db.Get(cmd)
	if err != nil {
		t.Errorf("Cannot get %s", cmd)
	}
	if v != "2" {
		t.Errorf("Got %s, want 2", v)
	}
}
