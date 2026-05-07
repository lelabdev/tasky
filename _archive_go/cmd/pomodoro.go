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
)

// PomodoroCommand returns a *cli.Command for the "pomodoro" command.
func PomodoroCommand() *cli.Command {
	return &cli.Command{
		Name:    "pomodoro",
		Aliases: []string{"po"},
		Usage:   "Manage Pomodoro timer settings and start the timer",
		Subcommands: []*cli.Command{
			{
				Name:  "start",
				Usage: "Start a Pomodoro timer",
				Action: func(c *cli.Context) error {
					cfg := config.LoadConfig()
					pomodoro.StartPomodoroCycle(cfg)
					return nil
				},
			},
		},
		Flags: []cli.Flag{
			&cli.BoolFlag{
				Name:    "configure",
				Aliases: []string{"c"},
				Usage:   "Configure Pomodoro timer settings",
			},
		},
		Action: func(c *cli.Context) error {
			if c.Bool("configure") {
				reader := bufio.NewReader(os.Stdin)
				cfg := config.LoadConfig()

				fmt.Print("What is the duration of a single Pomodoro (in minutes)? ")
				pomodoroDurationStr, _ := reader.ReadString('\n')
				pomodoroDuration, err := strconv.Atoi(strings.TrimSpace(pomodoroDurationStr))
				if err != nil {
					return cli.Exit("Invalid input. Please enter a number.", 1)
				}
				cfg.Pomodoro.PomodoroDuration = pomodoroDuration

				fmt.Print("What is the duration of a short break (in minutes)? ")
				shortBreakDurationStr, _ := reader.ReadString('\n')
				shortBreakDuration, err := strconv.Atoi(strings.TrimSpace(shortBreakDurationStr))
				if err != nil {
					return cli.Exit("Invalid input. Please enter a number.", 1)
				}
				cfg.Pomodoro.ShortBreakDuration = shortBreakDuration

				fmt.Print("What is the duration of a long break (in minutes)? ")
				longBreakDurationStr, _ := reader.ReadString('\n')
				longBreakDuration, err := strconv.Atoi(strings.TrimSpace(longBreakDurationStr))
				if err != nil {
					return cli.Exit("Invalid input. Please enter a number.", 1)
				}
				cfg.Pomodoro.LongBreakDuration = longBreakDuration

				fmt.Print("After how many Pomodoros should a long break occur? ")
				longBreakIntervalStr, _ := reader.ReadString('\n')
				longBreakInterval, err := strconv.Atoi(strings.TrimSpace(longBreakIntervalStr))
				if err != nil {
					return cli.Exit("Invalid input. Please enter a number.", 1)
				}
				cfg.Pomodoro.LongBreakInterval = longBreakInterval

				config.SaveConfig(cfg)
				fmt.Println("Pomodoro configuration saved.")
				return nil
			}
			cli.ShowAppHelp(c)
			return nil
		},
	}
}
