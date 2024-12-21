package bitcask

import (
	"encoding/binary"
	"time"
)

type Record struct {
	key     []byte
	value   []byte
	crc     uint32
	tstamp  uint32
	keysz   uint16
	valuesz uint16
}

const RECORD_HEADER_SIZE = 4 + 4 + 2 + 2

func NewRecord(k, v string) (r Record) {
	// TODO: int64
	r.tstamp = uint32(time.Now().Unix())

	r.key = []byte(k)
	r.value = []byte(v)
	r.keysz = uint16(len(r.key))
	r.valuesz = uint16(len(r.value))

	// TODO: calculate checksum
	r.crc = 0

	return r
}

func (r Record) size() uint32 {
	return RECORD_HEADER_SIZE + uint32(r.keysz+r.valuesz)
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
	r.tstamp = binary.LittleEndian.Uint32(b[4:8])
	r.keysz = binary.LittleEndian.Uint16(b[8:10])
	r.valuesz = binary.LittleEndian.Uint16(b[10:12])
	r.key = b[RECORD_HEADER_SIZE : RECORD_HEADER_SIZE+r.keysz]
	r.value = b[RECORD_HEADER_SIZE+r.keysz:]
	return
}
