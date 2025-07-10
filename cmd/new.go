package cmd

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/urfave/cli/v2"
	"tasky/config"
	"tasky/pomodoro"
	"tasky/task"
	"tasky/utils"
)

// NewCommand returns a *cli.Command for the "new" command.
func NewCommand() *cli.Command {
	return &cli.Command{
		Name:	  "new",
		Usage:	  "Create a new task",
		UsageText: "tasky new \"<title>\" [\"<description>\"]",
		Action: func(c *cli.Context) error {
			var title string
			reader := bufio.NewReader(os.Stdin)
			var input string
			var trimmedInput string

			if c.NArg() < 1 {
				fmt.Print("Enter task title: ")
				inputTitle, _ := reader.ReadString('\n')
				title = strings.TrimSpace(inputTitle)
				if title == "" {
					return cli.Exit("Task title cannot be empty.", 1)
				}
			} else {
				title = c.Args().Get(0)
			}

			description := ""
			if c.NArg() > 1 {
				description = c.Args().Get(1)
			}

			createGitHubIssue := false
			if utils.IsGitRepository() && utils.HasGitHubRemote() {
				fmt.Print("Create a GitHub issue? (Y/n): ")
				input, _ = reader.ReadString('\n')
				trimmedInput = strings.ToLower(strings.TrimSpace(input))
				if trimmedInput == "y" || trimmedInput == "" {
					createGitHubIssue = true
				}
			}

			cfg := config.LoadConfig()
			issueNumberStr, err := task.CreateTask(cfg.General.VaultPath, title, description, createGitHubIssue)
			if err != nil {
				return cli.Exit(fmt.Sprintf("Error creating task: %v", err), 1)
			}
			fmt.Printf("Task '%s' created successfully.\n", title)

			// Ask to start the task
			fmt.Print("Start this task? (Y/n): ")
			input, _ = reader.ReadString('\n')
			trimmedInput = strings.ToLower(strings.TrimSpace(input))

			if trimmedInput == "y" || trimmedInput == "" {
				if issueNumberStr != "" {
					// If GitHub issue was created, start development on it
					if err := task.StartTaskDevelopment(issueNumberStr); err != nil {
						return cli.Exit(fmt.Sprintf("Error starting task development: %v", err), 1)
					}
					issueNumber, _ := strconv.Atoi(issueNumberStr)
					task.MarkTaskInProgress(cfg.General.VaultPath, issueNumber)
				} else {
					task.MarkTaskInProgressByTitle(cfg.General.VaultPath, title)
				}
				utils.PlaySound(cfg.Sounds.Start)

				// Ask to start a Pomodoro
				fmt.Print("Start a Pomodoro? (Y/n): ")
				input, _ = reader.ReadString('\n')
				trimmedInput = strings.ToLower(strings.TrimSpace(input))

				if trimmedInput == "y" || trimmedInput == "" {
					pomodoro.StartPomodoroCycle(cfg)
				}
			}

			return nil
		},
	}
}