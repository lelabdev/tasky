package cmd

import (
	"fmt"

	"github.com/urfave/cli/v2"
	"tasky/config"
	"tasky/task"
	"tasky/utils"
)

func getStatusSymbol(status string) string {
	switch status {
	case config.StatusTodo:
		return config.SymbolTodo
	case config.StatusInProgress:
		return config.SymbolDoing
	case config.StatusDone:
		return config.SymbolDone
	default:
		return "?"
	}
}

// ListCommand returns a *cli.Command for the "list" command.
func ListCommand() *cli.Command {
	return &cli.Command{
		Name:    "list",
		Aliases: []string{"view"},
		Usage:   "List tasks",
		Flags: []cli.Flag{
			&cli.BoolFlag{
				Name:    "all",
				Aliases: []string{"a"},
				Usage:   "List all tasks, regardless of project",
			},
		},
		Action: func(c *cli.Context) error {
			cfg := config.LoadConfig()
			var tasks []config.Task

			if c.Bool("all") {
				tasks = task.GetTasks(cfg.General.VaultPath, "") // Get all tasks
			} else {
				projectName := c.Args().First()
				if projectName == "" {
					projectName = utils.GetProjectName()
					if projectName == "unknown_project" {
						return cli.Exit("Usage: tasky list [project_name] or tasky list --all. Run in a Git repository or provide a project name.", 1)
					}
				}
				tasks = task.GetTasks(cfg.General.VaultPath, projectName)
			}

			for _, t := range tasks {
				if t.Issue != 0 {
					fmt.Printf("%s #%d %s\n", getStatusSymbol(t.Status), t.Issue, t.Title)
				} else {
					fmt.Printf("%s %s\n", getStatusSymbol(t.Status), t.Title)
				}
			}
			return nil
		},
	}
}
