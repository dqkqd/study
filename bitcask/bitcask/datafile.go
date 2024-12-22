package bitcask

import (
	"encoding/binary"
	"fmt"
	"io"
	"os"
)

type DatafileId uint64

const INVALID_DATAFILE_ID DatafileId = 0

type ReadonlyDatafile struct {
	f  *os.File
	id DatafileId
}

type ActiveDatafile struct {
	f  *os.File
	id DatafileId
	sz uint32 // file size to determine if it exceeds threshold
}

func (d ReadonlyDatafile) Get(loc RecordLoc) (r Record, err error) {
	return getRecord(d.f, loc)
}

func (d ReadonlyDatafile) GetAllRecords() (records RecordsWithLoc, err error) {
	records, err = getAllRecords(d.f)
	if err != nil {
		return records, err
	}
	// set correct position
	for i := range records {
		records[i].loc.datafileId = d.id
	}
	return records.unique(), nil
}

func (d ActiveDatafile) Get(loc RecordLoc) (r Record, err error) {
	return getRecord(d.f, loc)
}

func (d *ActiveDatafile) Save(k string, v string) (loc RecordLoc, err error) {
	r := NewRecord(k, v)
	err = saveRecord(d.f, r)
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

func saveRecord(f *os.File, r Record) error {
	buf, err := r.Bytes()
	if err != nil {
		return err
	}

	n, err := f.Write(buf)
	// truncate the file in case of errror
	if err != nil {
		p, err := f.Seek(0, io.SeekCurrent)
		if err != nil {
			return err
		}

		err = f.Truncate(p - int64(n))
		if err != nil {
			return err
		}
	}

	return nil
}

func getAllRecords(f *os.File) (RecordsWithLoc, error) {
	_, err := f.Seek(0, io.SeekStart)
	if err != nil {
		return nil, err
	}

	records := RecordsWithLoc{}

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
		loc := RecordLoc{INVALID_DATAFILE_ID, r.size(), pos, r.tstamp}
		records = append(records, RecordWithLoc{r, loc})

		pos += r.size()
	}

	return records, nil
}
