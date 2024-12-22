package bitcask

type Keydir map[string]RecordLoc

func GetAllRecordsFromDirectory(dir Directory) (RecordsWithLoc, error) {
	allRecords := RecordsWithLoc{}

	for dfid := range dir.readonlyDatafileIds {
		rd, err := dir.readonlyDatafile(dfid)
		if err != nil {
			return allRecords, err
		}
		records, err := rd.GetAllRecords()
		if err != nil {
			return allRecords, err
		}
		allRecords = append(allRecords, records...)
	}

	return allRecords.unique(), nil
}

func openKeydir(dir Directory) (kd Keydir, err error) {
	kd = Keydir{}

	allRecords, err := GetAllRecordsFromDirectory(dir)
	if err != nil {
		return kd, err
	}

	for _, rl := range allRecords {
		kd[string(rl.r.key)] = rl.loc
	}
	return kd, nil
}
