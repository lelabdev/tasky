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

// StartCommand returns a *cli.Command for the "start" command.
func StartCommand() *cli.Command {
	return &cli.Command{
		Name:      "start",
		Usage:     "Start development on a GitHub issue",
		UsageText: "tasky start <issue_number>",
		Action: func(c *cli.Context) error {
			if c.NArg() < 1 {
				return cli.Exit("Usage: tasky start <issue_number>", 1)
			}
			issueNumberStr := c.Args().Get(0)
			issueNumber, err := strconv.Atoi(issueNumberStr)
			if err != nil {
				return cli.Exit(fmt.Sprintf("Error: Invalid issue number '%s'. Please provide a valid integer.\n", issueNumberStr), 1)
			}
			cfg := config.LoadConfig()
			if err := task.StartTaskDevelopment(issueNumberStr); err != nil {
				return cli.Exit(fmt.Sprintf("Error starting development: %v", err), 1)
			}
			task.MarkTaskInProgress(cfg, issueNumber)
			if err := utils.PlaySound(cfg.Sounds.Start); err != nil {
				fmt.Printf("Warning: %v\n", err)
			}
			fmt.Printf("Task for issue #%s started.\n", issueNumberStr)

            // Ask to start a Pomodoro
            fmt.Print("Start a Pomodoro? (Y/n): ")
            reader := bufio.NewReader(os.Stdin)
            input, _ := reader.ReadString('\n')
            trimmedInput := strings.ToLower(strings.TrimSpace(input))

            if trimmedInput == "y" || trimmedInput == "" {
                pomodoro.StartPomodoroCycle(cfg)
            }
            return nil
		},
	}
}

