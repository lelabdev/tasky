package main

import (
	"log"
	"os"

	"tasky/app"
)

func main() {
	cliApp := app.NewApp()

	if err := cliApp.Run(os.Args); err != nil {
		log.Fatal(err)
	}
}