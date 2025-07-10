package app

import (
	"github.com/urfave/cli/v2"
	"tasky/cmd"
)

// GetCommands returns a slice of all top-level CLI commands.
func GetCommands() []*cli.Command {
	return []*cli.Command{
		cmd.NewCommand(),
		cmd.ListCommand(),
		cmd.DoneCommand(),
		cmd.StartCommand(),
		cmd.FinishCommand(),
		cmd.PomodoroCommand(),
		cmd.LinkCommand(),
	}
}
