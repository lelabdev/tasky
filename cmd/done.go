package cmd

import (
	"fmt"

	"github.com/urfave/cli/v2"
	taskyconfig "tasky/config"
	tasky "tasky/task"
	"tasky/utils"
)

// DoneCommand returns a *cli.Command for the "done" command.
func DoneCommand() *cli.Command {
	return &cli.Command{
		Name:      "done",
		Usage:     "Mark a task as done",
		UsageText: "tasky done <task_title>",
		Action: func(c *cli.Context) error {
			if c.NArg() < 1 {
				return cli.Exit("Usage: tasky done <task_title>", 1)
			}
			taskTitle := c.Args().Get(0)
			cfg := taskyconfig.LoadConfig()
			if err := tasky.MarkTaskDone(cfg, taskTitle); err != nil {
				return cli.Exit(err.Error(), 1)
			}
			if err := utils.PlaySound(cfg.Sounds.Done); err != nil {
				fmt.Printf("Warning: %v\n", err)
			}
			fmt.Printf("Task '%s' marked as done.\n", taskTitle)
			return nil
		},
	}
}
