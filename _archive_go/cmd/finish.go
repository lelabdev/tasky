package cmd

import (
	"fmt"

	"github.com/urfave/cli/v2"
	"tasky/config"
	"tasky/task"
)

// FinishCommand returns a *cli.Command for the "finish" command.
func FinishCommand() *cli.Command {
	return &cli.Command{
		Name:  "finish",
		Usage: "Close a GitHub issue, merge PR, and mark task done",
		Action: func(c *cli.Context) error {
			cfg := config.LoadConfig()
			if err := task.FinishTask(cfg); err != nil {
				return cli.Exit(fmt.Sprintf("Error finishing task: %v\n", err), 1)
			}
			fmt.Println("Task finished successfully.")
			return nil
		},
	}
}

