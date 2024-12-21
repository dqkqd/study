package bitcask

import (
	"fmt"
	"path"
)

const DATA_FILE_EXT = "df"

func DatafilePath(folder string, id uint16) string {
	return path.Join(fmt.Sprintf("%s/%d.%s", folder, id, DATA_FILE_EXT))
}

