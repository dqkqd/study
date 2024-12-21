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

func (d ReadonlyDatafile) Get(loc RecordLoc) (r Record, err error) {
	f, err := os.Open(DatafilePath(*d.folder, d.id))
	if err != nil {
		return r, err
	}
	return getRecord(f, loc)
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

func (d ActiveDatafile) Get(loc RecordLoc) (r Record, err error) {
	return getRecord(d.f, loc)
}

func (d *ActiveDatafile) Save(k string, v string) (loc RecordLoc, err error) {
	r, err := saveRecord(d.f, k, v)
	if err != nil {
		return loc, err
	}

	loc = RecordLoc{d.id, r.size(), d.sz, r.tstamp}
	d.sz += r.size()
	return loc, nil
}

func getRecord(f *os.File, loc RecordLoc) (r Record, err error) {
	buf := make([]byte, loc.sz)
	n, err := f.ReadAt(buf, int64(loc.pos))
	if err != nil {
		return r, err
	}
	if n != int(loc.sz) {
		return r, fmt.Errorf("Cannot read record of size %d", loc.sz)
	}

	return RecordFromBytes(buf), nil
}

func saveRecord(f *os.File, k, v string) (r Record, err error) {
	r = NewRecord(k, v)
	buf, err := r.Bytes()
	if err != nil {
		return r, err
	}

	n, err := f.Write(buf)
	// truncate the file in case of errror
	if err != nil {
		p, err := f.Seek(0, io.SeekCurrent)
		if err != nil {
			return r, err
		}

		err = f.Truncate(p - int64(n))
		if err != nil {
			return r, err
		}
	}

	return r, nil
}
