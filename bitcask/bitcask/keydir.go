package bitcask

type RecordLoc struct {
	datafileId uint16
	sz         uint32
	pos        uint32
	tstamp     uint32
}

type Keydir map[string]RecordLoc

func OpenKeydir() Keydir {
	// TODO: setup and initialize
	return map[string]RecordLoc{}
}
