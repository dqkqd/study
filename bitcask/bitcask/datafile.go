package bitcask

import (
	"fmt"
	"io"
	"os"
	"path"
)

const DATA_FILE_EXT = "df"

type ReadonlyDatafile struct {
	folder *string
	id     uint16
}

type ActiveDatafile struct {
	folder *string
	id     uint16
}

func (d ReadonlyDatafile) Active() ActiveDatafile {
	return ActiveDatafile(d)
}

func (d ReadonlyDatafile) Get(pos uint32, sz uint32) (r Record, err error) {
	return getRecord(filepath(*d.folder, d.id), pos, sz)
}

func (d ActiveDatafile) Readonly() ReadonlyDatafile {
	return ReadonlyDatafile(d)
}

func (d ActiveDatafile) Get(pos uint32, sz uint32) (r Record, err error) {
	return getRecord(filepath(*d.folder, d.id), pos, sz)
}

func (d ActiveDatafile) Save(k string, v string) (pos uint32, sz uint32, err error) {
	return saveRecord(filepath(*d.folder, d.id), k, v)
}

func filepath(folder string, id uint16) string {
	return path.Join(fmt.Sprintf("%s/%d.%s", folder, id, DATA_FILE_EXT))
}

func getRecord(fp string, pos uint32, sz uint32) (r Record, err error) {
	f, err := os.Open(fp)
	if err != nil {
		return r, err
	}

	buf := make([]byte, sz)
	n, err := f.ReadAt(buf, int64(pos))
	if err != nil {
		return r, err
	}
	if n != int(sz) {
		return r, fmt.Errorf("Cannot read record of size %d", sz)
	}

	return RecordFromBytes(buf), nil
}

func saveRecord(fp, k, v string) (pos, sz uint32, err error) {
	f, err := os.OpenFile(fp, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return pos, sz, err
	}

	r := NewRecord(k, v)
	buf, err := r.Bytes()
	if err != nil {
		return pos, sz, err
	}

	// Get file position before writing
	p, err := f.Seek(0, io.SeekEnd)
	pos = uint32(p)
	if err != nil {
		return pos, sz, err
	}

	_, err = f.Write(buf)
	return pos, r.size(), err
}
