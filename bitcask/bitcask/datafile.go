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
	f      *os.File // always hold file pointer for reading and writing
	folder *string
	id     uint16
}

func (d ReadonlyDatafile) Active() (ActiveDatafile, error) {
	return OpenAsActiveDatafile(d.folder, d.id)
}

func (d ReadonlyDatafile) Get(pos uint32, sz uint32) (r Record, err error) {
	f, err := os.Open(filepath(*d.folder, d.id))
	if err != nil {
		return r, err
	}
	return getRecord(f, pos, sz)
}

func (d *ActiveDatafile) Readonly() ReadonlyDatafile {
	defer func() { d = nil }()
	return ReadonlyDatafile{d.folder, d.id}
}

func OpenAsActiveDatafile(folder *string, id uint16) (d ActiveDatafile, err error) {
	f, err := os.OpenFile(filepath(*folder, id), os.O_APPEND|os.O_CREATE|os.O_RDWR, 0644)
	if err != nil {
		return d, err
	}
	return ActiveDatafile{f, folder, id}, nil
}

func (d ActiveDatafile) Get(pos uint32, sz uint32) (r Record, err error) {
	return getRecord(d.f, pos, sz)
}

func (d ActiveDatafile) Save(k string, v string) (pos uint32, sz uint32, err error) {
	return saveRecord(d.f, k, v)
}

func filepath(folder string, id uint16) string {
	return path.Join(fmt.Sprintf("%s/%d.%s", folder, id, DATA_FILE_EXT))
}

func getRecord(f *os.File, pos uint32, sz uint32) (r Record, err error) {
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

func saveRecord(f *os.File, k, v string) (pos, sz uint32, err error) {
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
