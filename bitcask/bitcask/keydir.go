package bitcask

import "fmt"

type ValuePos struct {
	fid      uint16
	valuesz  uint32
	valuepos uint32
	tstamp   uint32
}

type Keydir struct {
	values map[string]ValuePos
}

func OpenKeydir() Keydir {
	return Keydir{
		map[string]ValuePos{},
	}
}

func (k Keydir) Save(d ActiveDatafile, key, value string) error {
	pos, sz, err := d.Save(key, value)
	if err != nil {
		return err
	}

	// TODO: handle valuesz and tstamp
	k.values[key] = ValuePos{d.id, sz, pos, 0}
	return nil
}

func (k Keydir) Get(d ActiveDatafile, key string) (r Record, err error) {
	vp, ok := k.values[key]
	if !ok {
		return r, fmt.Errorf("Cannot get datafile for key %s", key)
	}

	if d.id == vp.fid {
		// This key is in active file, we can get it without opening new files
		return d.Get(vp.valuepos, vp.valuesz)
	} else {
		// This key is in other files, need to open and read it
		// TODO: cover test for this
		rd := ReadonlyDatafile{d.folder, vp.fid}
		return rd.Get(vp.valuepos, vp.valuesz)
	}
}
