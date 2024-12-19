package bitcask

import (
	"encoding/binary"
	"fmt"
	"io"
	"os"
	"path"
)

const (
	DATA_FILE_EXT      = "df"
	RECORD_HEADER_SIZE = 4 + 4 + 2 + 2
)

type Record struct {
	key     []byte
	value   []byte
	crc     uint32
	tstamp  uint32
	keysz   uint16
	valuesz uint16
}

type Datafile struct {
	rootFolder *string
	id         uint16
}

func (d Datafile) filepath() string {
	return path.Join(fmt.Sprintf("%s/%d.%s", *d.rootFolder, d.id, DATA_FILE_EXT))
}

func (d Datafile) Save(key string, value string) (pos uint32, err error) {
	f, err := os.OpenFile(d.filepath(), os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return pos, err
	}

	var record Record
	// TODO: fill these values
	record.crc = 0
	record.tstamp = 0
	record.key = []byte(key)
	record.value = []byte(value)
	record.keysz = uint16(len(record.key))
	record.valuesz = uint16(len(record.value))

	// Get file position before writing
	p, err := f.Seek(0, io.SeekEnd)
	pos = uint32(p)
	if err != nil {
		return pos, err
	}

	err = binary.Write(f, binary.LittleEndian, record.crc)
	err = binary.Write(f, binary.LittleEndian, record.tstamp)
	err = binary.Write(f, binary.LittleEndian, record.keysz)
	err = binary.Write(f, binary.LittleEndian, record.valuesz)
	err = binary.Write(f, binary.LittleEndian, record.key)
	err = binary.Write(f, binary.LittleEndian, record.value)
	if err != nil {
		// revert back the latest position before writing
		f.Truncate(int64(pos))
	}

	return pos, err
}

func (d Datafile) Get(recordPos uint32) (record Record, err error) {
	f, err := os.Open(d.filepath())
	if err != nil {
		return record, err
	}
	pos, err := f.Seek(int64(recordPos), io.SeekStart)
	if err != nil {
		return record, err
	}
	if pos != int64(recordPos) {
		return record, fmt.Errorf("Cannot seek position %d", recordPos)
	}

	var buf []byte

	buf = make([]byte, RECORD_HEADER_SIZE)
	n, err := f.Read(buf)
	if err != nil {
		return record, err
	}
	if n != RECORD_HEADER_SIZE {
		return record, fmt.Errorf("Cannot read record header of size %d", RECORD_HEADER_SIZE)
	}

	record.crc = binary.LittleEndian.Uint32(buf[:4])
	record.tstamp = binary.LittleEndian.Uint32(buf[4:8])
	record.keysz = binary.LittleEndian.Uint16(buf[8:10])
	record.valuesz = binary.LittleEndian.Uint16(buf[10:12])

	keyAndValueSz := record.keysz + record.valuesz
	buf = make([]byte, keyAndValueSz)

	n, err = f.Read(buf)
	if err != nil {
		return record, err
	}
	if n != int(keyAndValueSz) {
		return record, fmt.Errorf("Cannot read record key and value of size %d", keyAndValueSz)
	}

	record.key = buf[:record.keysz]
	record.value = buf[record.keysz:]

	return record, nil
}
