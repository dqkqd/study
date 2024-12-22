package bitcask

import (
	"encoding/binary"
	"fmt"
	"io"
	"os"
)

type ReadonlyDatafile struct {
	f *os.File
}

type ActiveDatafile struct {
	f  *os.File
	id DatafileId // need to know id to tell caller when saving record
	sz uint32     // file size to determine if it exceeds threshold
}

func (d ReadonlyDatafile) Get(loc RecordLoc) (r Record, err error) {
	return getRecord(d.f, loc)
}

func (d ReadonlyDatafile) GetAllRecords() (map[string]RecordAndPos, error) {
	return getAllRecords(d.f)
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

func getAllRecords(f *os.File) (map[string]RecordAndPos, error) {
	_, err := f.Seek(0, io.SeekStart)
	if err != nil {
		return nil, err
	}

	recordAndLocs := map[string]RecordAndPos{}
	pos := uint32(0)
	for {
		buf := make([]byte, RECORD_HEADER_SIZE)
		n, err := f.Read(buf)
		if err != nil || n != RECORD_HEADER_SIZE {
			break
		}
		keysz := binary.LittleEndian.Uint32(buf[12:16])
		valuesz := binary.LittleEndian.Uint32(buf[16:20])

		buf = append(buf, make([]byte, keysz+valuesz)...)
		n, err = f.Read(buf[n:])
		if n != int(keysz+valuesz) {
			break
		}

		r := RecordFromBytes(buf)
		oldr, existed := recordAndLocs[string(r.key)]

		if !existed || r.tstamp > oldr.r.tstamp {
			recordAndLocs[string(r.key)] = RecordAndPos{r, pos}
		}

		pos += r.size()
	}

	return recordAndLocs, nil
}
