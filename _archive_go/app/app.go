package app

import (
	"github.com/urfave/cli/v2"
)

// NewApp creates and configures a new urfave/cli application.
func NewApp() *cli.App {
	app := &cli.App{
		Name:    "tasky",
		Usage:   "A command-line task manager",
		Version: "1.1.0", // You can manage your version here
		Commands: GetCommands(),
	}
	return app
}
