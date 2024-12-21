package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	bitcask "example.com/bitcask"
)

func main() {
	reader := bufio.NewReader(os.Stdin)
	db, err := bitcask.OpenDatabase("dbfiles")
	if err != nil {
		panic("Cannot open database")
	}

	for {
		fmt.Print(">>> ")

		var input string
		input, err := reader.ReadString('\n')
		if err != nil {
			break
		}
		input = strings.TrimSpace(input)
		if len(input) == 0 {
			continue
		}
		db.HandleQuery(input)
	}
}
