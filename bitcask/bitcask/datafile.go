package bitcask

import (
	"fmt"
	"io"
	"os"
)

type ReadonlyDatafile struct {
	folder *string
	id     uint16
}

type ActiveDatafile struct {
	f      *os.File // always hold file pointer for reading and writing
	folder *string
	id     uint16
	sz     uint32 // file size to determine if it exceeds threshold
}

func (d ReadonlyDatafile) Active() (ActiveDatafile, error) {
	return OpenAsActiveDatafile(d.folder, d.id)
}

func (d ReadonlyDatafile) Get(pos uint32, sz uint32) (r Record, err error) {
	f, err := os.Open(DatafilePath(*d.folder, d.id))
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
	f, err := os.OpenFile(DatafilePath(*folder, id), os.O_APPEND|os.O_CREATE|os.O_RDWR, 0644)
	if err != nil {
		return d, err
	}
	sz, err := f.Seek(0, io.SeekCurrent)
	if err != nil {
		return d, err
	}
	return ActiveDatafile{f, folder, id, uint32(sz)}, nil
}

func (d ActiveDatafile) Get(pos uint32, sz uint32) (r Record, err error) {
	return getRecord(d.f, pos, sz)
}

func (d *ActiveDatafile) Save(k string, v string) (pos uint32, sz uint32, err error) {
	recordsz, err := saveRecord(d.f, k, v)
	if err != nil {
		return pos, sz, err
	}

	pos = d.sz
	d.sz += uint32(recordsz)

	return pos, uint32(recordsz), nil
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

func saveRecord(f *os.File, k, v string) (sz int, err error) {
	r := NewRecord(k, v)
	buf, err := r.Bytes()
	if err != nil {
		return sz, err
	}

	n, err := f.Write(buf)
	// truncate the file in case of errror
	if err != nil {
		p, err := f.Seek(0, io.SeekCurrent)
		if err != nil {
			return 0, err
		}

		err = f.Truncate(p - int64(n))
		if err != nil {
			return 0, err
		}
	}

	return n, nil
}
