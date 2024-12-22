package bitcask

type Keydir map[string]RecordLoc

func openKeydir(dir Directory) (kd Keydir, err error) {
	kd = Keydir{}

	for dfid := range dir.readonlyDatafileIds {
		rd, err := dir.readonlyDatafile(dfid)
		if err != nil {
			return kd, err
		}
		recordAndPos, err := rd.GetAllRecords()
		if err != nil {
			return kd, err
		}

		for k, rp := range recordAndPos {
			loc, existed := kd[k]
			if !existed || loc.tstamp < rp.r.tstamp {
				kd[k] = RecordLoc{dfid, rp.r.size(), rp.pos, rp.r.tstamp}
			}
		}
	}

	return kd, nil
}
