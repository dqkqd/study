package bitcask

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

const DATAFILE_PREFIX = "DATAFILE"

type Directory struct {
	readonlyDatafileIds map[DatafileId]bool
	folder              string
	activeDatafileId    DatafileId
}

func openDirectory(folder string) (dir Directory, err error) {
	err = os.MkdirAll(folder, 0700)
	if err != nil {
		return dir, err
	}

	readonlyDatafileIds := map[DatafileId]bool{}

	// get all datafiles
	datafilePaths, err := filepath.Glob(fmt.Sprintf("%s/%s_*", folder, DATAFILE_PREFIX))
	if err != nil {
		return dir, err
	}
	for _, fp := range datafilePaths {
		datafileId, err := datafileIdFromPath(fp)
		if err != nil {
			return dir, err
		}
		readonlyDatafileIds[datafileId] = true
	}

	dir.folder = folder
	dir.readonlyDatafileIds = readonlyDatafileIds
	dir.activeDatafileId = dir.nextActiveDatafileId()
	return dir, nil
}

func (dir Directory) readonlyDatafile(id DatafileId) (d ReadonlyDatafile, error error) {
	f, err := os.Open(dir.datafilePath(id))
	return ReadonlyDatafile{f, id}, err
}

func (dir Directory) activeDatafile() (d ActiveDatafile, err error) {
	f, err := os.OpenFile(dir.datafilePath(dir.activeDatafileId), os.O_APPEND|os.O_CREATE|os.O_RDWR, 0666)
	return ActiveDatafile{f, dir.activeDatafileId, 0}, err
}

func (dir Directory) nextActiveDatafileId() DatafileId {
	datafileId := INVALID_DATAFILE_ID + 1
	for datafileId == dir.activeDatafileId || dir.readonlyDatafileIds[datafileId] {
		f, err := os.OpenFile(dir.datafilePath(datafileId), os.O_RDWR|os.O_CREATE|os.O_EXCL, 0666)
		if err == nil {
			// file does not exist
			f.Close()
			break
		}
		datafileId++
	}
	return datafileId
}

func (dir *Directory) nextActiveDatafile() (d ActiveDatafile, err error) {
	// set current active datafile as readonly
	dir.readonlyDatafileIds[dir.activeDatafileId] = true
	// move to the next one
	dir.activeDatafileId = dir.nextActiveDatafileId()
	return dir.activeDatafile()
}

// Create a temporary active datafile for write
func (dir Directory) tempActiveDatafile() (d ActiveDatafile, err error) {
	datafileId := dir.nextActiveDatafileId()
	f, err := os.OpenFile(dir.datafilePath(datafileId), os.O_APPEND|os.O_CREATE|os.O_RDWR, 0666)
	return ActiveDatafile{f, datafileId, 0}, err
}

func (dir *Directory) removeReadonlyDatafile(id DatafileId) error {
	fp := dir.datafilePath(id)
	err := os.Remove(fp)
	if err != nil {
		return err
	}
	delete(dir.readonlyDatafileIds, id)
	return nil
}

func (dir Directory) datafilePath(id DatafileId) string {
	return filepath.Join(fmt.Sprintf("%s/%s_%010d", dir.folder, DATAFILE_PREFIX, id))
}

func datafileIdFromPath(fp string) (id DatafileId, err error) {
	_, filename := filepath.Split(fp)
	components := strings.Split(filename, "_")
	if len(components) != 2 {
		return 0, fmt.Errorf("Invalid datafile name %s", fp)
	}
	datafileId, err := strconv.Atoi(components[1])
	if err != nil {
		return 0, err
	}
	return DatafileId(datafileId), nil
}
