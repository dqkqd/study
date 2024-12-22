package bitcask

import (
	"fmt"
	"os"
	"path/filepath"
)

const DATAFILE_PREFIX = "DATAFILE"

type Directory struct {
	folder string
}

func openDirectory(folder string) (dir Directory, err error) {
	dir.folder = folder
	err = os.MkdirAll(folder, 0700)
	return dir, err
}

func (dir Directory) DatafilePath(id uint16) string {
	return filepath.Join(fmt.Sprintf("%s/%s_%010d", dir.folder, DATAFILE_PREFIX, id))
}

