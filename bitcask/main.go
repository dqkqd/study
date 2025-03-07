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
	cfg := bitcask.DefaultDatabaseConfig()
	db, err := bitcask.OpenDatabase("dbfiles", cfg)
	if err != nil {
		panic("Cannot open database")
	}
	defer db.Close()

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
