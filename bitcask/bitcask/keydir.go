package bitcask

type Keydir map[string]RecordLoc

func GetReadonlyRecordsFromDirectory(dir Directory) (RecordsWithLoc, error) {
	allRecords := RecordsWithLoc{}

	for id := range dir.readonlyDatafileIds {
		rd, err := dir.readonlyDatafile(id)
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

	allRecords, err := GetReadonlyRecordsFromDirectory(dir)
	if err != nil {
		return kd, err
	}

	for _, rl := range allRecords {
		kd[string(rl.r.key)] = rl.loc
	}
	return kd, nil
}
