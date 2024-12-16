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

		cmd, err := bitcask.ParseCommand(input)
		if err != nil {
			fmt.Println(err)
		} else {
			fmt.Println(cmd)
		}
	}
}
