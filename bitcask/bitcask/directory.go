package bitcask

import (
	"fmt"
	"io"
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
	dir.activeDatafileId = dir.getNextActiveDatafileId()
	return dir, nil
}

func (dir Directory) readonlyDatafile(id DatafileId) (d ReadonlyDatafile, error error) {
	if !dir.readonlyDatafileIds[id] {
		return d, fmt.Errorf("Non existed readonly datafile id %d", id)
	}
	f, err := os.Open(dir.datafilePath(id))
	return ReadonlyDatafile{f, id}, err
}

func (dir Directory) activeDatafile() (d ActiveDatafile, err error) {
	f, err := os.OpenFile(dir.datafilePath(dir.activeDatafileId), os.O_APPEND|os.O_CREATE|os.O_RDWR, 0644)
	sz, err := f.Seek(0, io.SeekCurrent)
	if err != nil {
		return d, err
	}
	return ActiveDatafile{f, dir.activeDatafileId, uint32(sz)}, err
}

func (dir Directory) getNextActiveDatafileId() DatafileId {
	datafileId := INVALID_DATAFILE_ID + 1
	for datafileId == dir.activeDatafileId || dir.readonlyDatafileIds[datafileId] {
		_, err := os.Stat(dir.datafilePath(datafileId))
		if err != nil {
			// file does not exist, can use
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
	dir.activeDatafileId = dir.getNextActiveDatafileId()
	return dir.activeDatafile()
}

// Create a temporary active datafile for write
func (dir Directory) tempActiveDatafile() (d ActiveDatafile, err error) {
	datafileId := dir.getNextActiveDatafileId()
	f, err := os.OpenFile(dir.datafilePath(datafileId), os.O_APPEND|os.O_CREATE|os.O_RDWR, 0644)
	return ActiveDatafile{f, datafileId, 0}, err
}

func (dir *Directory) removeReadonlyDatafile(id DatafileId) error {
	if !dir.readonlyDatafileIds[id] {
		return fmt.Errorf("Datafile id %d does not exist", id)
	}
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
