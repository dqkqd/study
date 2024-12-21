package bitcask

import (
	"fmt"
	"io"
	"os"
	"path"
)

const DATA_FILE_EXT = "df"

type Datafile struct {
	rootFolder *string
	id         uint16
}

func (d Datafile) filepath() string {
	return path.Join(fmt.Sprintf("%s/%d.%s", *d.rootFolder, d.id, DATA_FILE_EXT))
}

func (d Datafile) Save(k string, v string) (pos uint32, sz uint32, err error) {
	f, err := os.OpenFile(d.filepath(), os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
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

func (d Datafile) Get(pos uint32, sz uint32) (r Record, err error) {
	f, err := os.Open(d.filepath())
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
