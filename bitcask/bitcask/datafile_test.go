package bitcask

import (
	"testing"
)

func TestDatafileGetRecords(t *testing.T) {
	dir := Directory{t.TempDir()}
	d, err := openAsActiveDatafile(&dir, 1)
	if err != nil {
		t.Fail()
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

	// get as read only
	rdf := d.Readonly()
	for i, tc := range testcases {
		record, err := rdf.Get(locs[i])
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
}
