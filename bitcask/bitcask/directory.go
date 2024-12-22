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
	dir.folder = folder
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
	maxDatafileId := DatafileId(0)
	for _, fp := range datafilePaths {
		datafileId, err := datafileIdFromPath(fp)
		if err != nil {
			return dir, err
		}
		readonlyDatafileIds[datafileId] = true
		maxDatafileId = max(maxDatafileId, datafileId)
	}

	return Directory{
		readonlyDatafileIds,
		folder,
		// make new datafile id as active
		maxDatafileId + 1,
	}, nil
}

func (dir Directory) readonlyDatafile(id DatafileId) (d ReadonlyDatafile, error error) {
	if !dir.readonlyDatafileIds[id] {
		return d, fmt.Errorf("Non existed readonly datafile id %d", id)
	}
	f, err := os.Open(datafilePath(dir.folder, id))
	return ReadonlyDatafile{f, id}, err
}

func (dir Directory) activeDatafile() (d ActiveDatafile, err error) {
	f, err := os.OpenFile(datafilePath(dir.folder, dir.activeDatafileId), os.O_APPEND|os.O_CREATE|os.O_RDWR, 0644)
	sz, err := f.Seek(0, io.SeekCurrent)
	if err != nil {
		return d, err
	}
	return ActiveDatafile{f, dir.activeDatafileId, uint32(sz)}, err
}

func (dir *Directory) nextActiveDatafile() (d ActiveDatafile, err error) {
	// set current active datafile as readonly
	dir.readonlyDatafileIds[dir.activeDatafileId] = true
	// move to the next one
	dir.activeDatafileId += 1
	return dir.activeDatafile()
}

func datafilePath(folder string, id DatafileId) string {
	return filepath.Join(fmt.Sprintf("%s/%s_%010d", folder, DATAFILE_PREFIX, id))
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
