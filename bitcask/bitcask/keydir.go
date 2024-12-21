package bitcask

type ValuePos struct {
	fid      uint16
	valuesz  uint32
	valuepos uint32
	tstamp   uint32
}

type Keydir map[string]ValuePos

func OpenKeydir() Keydir {
	// TODO: setup and initialize
	return map[string]ValuePos{}
}
