package bitcask

import "fmt"

type ValuePos struct {
	fid      uint16
	valuesz  uint16
	valuepos uint32
	tstamp   uint32
}

type Keydir struct {
	values map[string]ValuePos
	folder *string
}

func OpenKeydir(folder *string) Keydir {
	return Keydir{
		map[string]ValuePos{},
		folder,
	}
}

func (k Keydir) Save(key, value string, fid uint16) error {
	df := Datafile{k.folder, fid}
	pos, err := df.Save(key, value)
	if err != nil {
		return err
	}

	// TODO: handle valuesz and tstamp
	k.values[key] = ValuePos{fid, 0, pos, 0}
	return nil
}

func (k Keydir) Get(key string) (r Record, err error) {
	vp, ok := k.values[key]
	if !ok {
		return r, fmt.Errorf("Cannot get datafile for key %s", key)
	}
	df := Datafile{k.folder, vp.fid}
	return df.Get(vp.valuepos)
}
