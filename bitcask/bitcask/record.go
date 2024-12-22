package bitcask

import (
	"encoding/binary"
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

type RecordLoc struct {
	datafileId DatafileId
	sz         uint32
	pos        uint32
	tstamp     uint64
}

type RecordAndPos struct {
	r   Record
	pos uint32
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
