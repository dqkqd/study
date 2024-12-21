package bitcask

import (
	"testing"
)

func TestKeydirGet(t *testing.T) {
	dir := t.TempDir()
	kd := OpenKeydir(&dir)

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
		err := kd.Save(tc.key, tc.value, 1)
		if err != nil {
			t.Errorf("Cannot save record: %s", err)
		}
	}

	for _, tc := range testcases {
		record, err := kd.Get(tc.key)
		if err != nil {
			t.Errorf("Cannot get key %s, error: %s", tc.key, err)
		}
		if string(record.value) != tc.value {
			t.Errorf("Got %s, want %s", record.value, tc.value)
		}
	}
}

func TestKeydirGetOverwrite(t *testing.T) {
	dir := t.TempDir()
	kd := OpenKeydir(&dir)

	kd.Save("key", "value", 1)

	record, err := kd.Get("key")
	if err != nil {
		t.Error(err)
	}
	if string(record.value) != "value" {
		t.Errorf("Got %s, want `value`", record.value)
	}

	kd.Save("key", "new value", 2)
	record, err = kd.Get("key")
	if err != nil {
		t.Error(err)
	}
	if string(record.value) != "new value" {
		t.Errorf("Got %s, want `new value`", record.value)
	}
}
