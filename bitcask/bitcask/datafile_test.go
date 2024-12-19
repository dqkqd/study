package bitcask

import (
	"testing"
)

func TestDatafileGetRecords(t *testing.T) {
	dir := t.TempDir()
	df := Datafile{&dir, 1}

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

	positions := make([]uint32, len(testcases))

	// save
	for i, tc := range testcases {
		pos, err := df.Save(tc.key, tc.value)
		if err != nil {
			t.Errorf("Cannot save record: %s", err)
		}
		positions[i] = pos
	}

	// get
	for i, tc := range testcases {
		record, err := df.Get(positions[i])
		if err != nil {
			t.Errorf("Cannot get record at pos=%d", positions[i])
		}
		if string(record.key) != tc.key {
			t.Errorf("Got %s, want %s", record.key, tc.key)
		}
		if string(record.value) != tc.value {
			t.Errorf("Got %s, want %s", record.value, tc.value)
		}
	}
}
