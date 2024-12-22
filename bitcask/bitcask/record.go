package bitcask

import (
	"encoding/binary"
	"sort"
	"time"
)

const RECORD_HEADER_SIZE = 4 + 8 + 4 + 4

type Record struct {
	key     []byte
	value   []byte
	crc     uint32
	tstamp  uint64
	keysz   uint32
	valuesz uint32
}

func NewRecord(k, v string) (r Record) {
	r.tstamp = uint64(time.Now().Unix())

	r.key = []byte(k)
	r.value = []byte(v)
	r.keysz = uint32(len(r.key))
	r.valuesz = uint32(len(r.value))

	// TODO: calculate checksum
	r.crc = 0

	return r
}

func (r Record) size() uint32 {
	return RECORD_HEADER_SIZE + r.keysz + r.valuesz
}

func (r Record) less(o Record) bool {
	rkey := string(r.key)
	okey := string(o.key)
	if rkey == okey {
		return r.older(o)
	}
	return rkey < okey
}

func (r Record) older(o Record) bool {
	return r.tstamp < o.tstamp
}

func (r Record) Bytes() (b []byte, err error) {
	b, err = binary.Append(nil, binary.LittleEndian, r.crc)
	if err != nil {
		return
	}

	b, err = binary.Append(b, binary.LittleEndian, r.tstamp)
	if err != nil {
		return
	}

	b, err = binary.Append(b, binary.LittleEndian, r.keysz)
	if err != nil {
		return
	}

	b, err = binary.Append(b, binary.LittleEndian, r.valuesz)
	if err != nil {
		return
	}

	b, err = binary.Append(b, binary.LittleEndian, r.key)
	if err != nil {
		return
	}

	b, err = binary.Append(b, binary.LittleEndian, r.value)
	return b, err
}

func RecordFromBytes(b []byte) (r Record) {
	r.crc = binary.LittleEndian.Uint32(b[:4])
	r.tstamp = binary.LittleEndian.Uint64(b[4:12])
	r.keysz = binary.LittleEndian.Uint32(b[12:16])
	r.valuesz = binary.LittleEndian.Uint32(b[16:20])
	r.key = b[RECORD_HEADER_SIZE : RECORD_HEADER_SIZE+r.keysz]
	r.value = b[RECORD_HEADER_SIZE+r.keysz:]
	return
}

type RecordLoc struct {
	datafileId DatafileId
	sz         uint32
	pos        uint32
	tstamp     uint64
}

type RecordWithLoc struct {
	r   Record
	loc RecordLoc
}

type RecordsWithLoc []RecordWithLoc

func (records RecordsWithLoc) unique() RecordsWithLoc {
	uniqueRecordsMap := map[string]RecordWithLoc{}
	for _, rl := range records {
		key := string(rl.r.key)
		existedRecord, existed := uniqueRecordsMap[key]
		if !existed || existedRecord.r.older(rl.r) {
			uniqueRecordsMap[key] = rl
		}
	}

	// convert to array and sort
	uniqueRecords := RecordsWithLoc{}
	for _, r := range uniqueRecordsMap {
		uniqueRecords = append(uniqueRecords, r)
	}
	sort.SliceStable(uniqueRecords, func(i, j int) bool {
		return uniqueRecords[i].r.less(uniqueRecords[j].r)
	})

	return uniqueRecords
}
