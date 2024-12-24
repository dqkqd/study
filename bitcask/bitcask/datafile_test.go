package bitcask

import (
	"testing"
)

func TestDatafileSetGetDeleteRecords(t *testing.T) {
	tempdir := t.TempDir()
	dir, err := openDirectory(tempdir)
	if err != nil {
		t.Fail()
	}

	d, err := dir.activeDatafile()
	if err != nil {
		t.Error("Cannot open active datafile")
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

	locs := make([]RecordLoc, len(testcases))

	// save
	for i, tc := range testcases {
		loc, err := d.Save(tc.key, tc.value)
		if err != nil {
			t.Errorf("Cannot save record: %s", err)
		}
		locs[i] = loc
	}

	// get
	for i, tc := range testcases {
		record, err := d.Get(locs[i])
		if err != nil {
			t.Errorf("Cannot get record at pos=%d", locs[i].pos)
		}
		if string(record.key) != tc.key {
			t.Errorf("Got %s, want %s", record.key, tc.key)
		}
		if string(record.value) != tc.value {
			t.Errorf("Got %s, want %s", record.value, tc.value)
		}
	}

	deletedLocs := make([]RecordLoc, len(testcases))
	// delete
	for i, tc := range testcases {
		loc, err := d.Delete(tc.key)
		if err != nil {
			t.Errorf("Cannot delete record: %s", err)
		}
		deletedLocs[i] = loc
	}

	// get deleted records
	for i, tc := range testcases {
		record, err := d.Get(deletedLocs[i])
		if err != nil {
			t.Errorf("Cannot get record at pos=%d", deletedLocs[i].pos)
		}
		if string(record.key) != tc.key {
			t.Errorf("Got %s, want %s", record.key, tc.key)
		}
		if string(record.value) != TOMBSTONE {
			t.Errorf("Got %s, want %s", record.value, tc.value)
		}
	}
}
