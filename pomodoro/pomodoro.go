package pomodoro

import (
	"bufio"
	"fmt"
	"os"
	
	"strings"
	"time"

	"tasky/config"
	"tasky/task"
)

// StartPomodoroCycle starts a single Pomodoro countdown.
func StartPomodoroCycle(cfg config.Config) {
	reader := bufio.NewReader(os.Stdin)
	for {
		fmt.Println("Starting Pomodoro session...")
		StartPomodoroAnimation(cfg)
					if err := task.IncrementPomodoroCountForActiveTask(cfg.General.VaultPath); err != nil {
			fmt.Println("[WARN] Could not increment Pomodoro counter for the active task:", err)
		}
		fmt.Println("Pomodoro finished!")
		// Music pause functionality removed as cfg.Commands is no longer available.
		// cmd := exec.Command(cfg.Commands.Playerctl, "play-pause")
		// err := cmd.Run()
		// if err != nil {
		// 	fmt.Println("[WARN] Could not pause music:", err)
		// }

		// Offer break
		fmt.Print("Start a break? (Y/n): ")
		input, _ := reader.ReadString('\n')
		if trimmed := strings.TrimSpace(strings.ToLower(input)); trimmed == "n" {

			break
		}
		breakDuration := cfg.Pomodoro.ShortBreakDuration
		if breakDuration <= 0 {
			breakDuration = 5 // default 5 min
		}
		fmt.Printf("Starting break for %d minutes...\n", breakDuration)
		StartBreakAnimation(breakDuration)
		fmt.Println("Break finished!")

		// Offer another Pomodoro
		fmt.Print("Start another Pomodoro? (Y/n): ")
		input, _ = reader.ReadString('\n')
		if trimmed := strings.TrimSpace(strings.ToLower(input)); trimmed == "n" {
			break
		}
	}
	fmt.Println("Pomodoro cycle ended.")
}

// StartBreakAnimation displays a countdown animation for the break.
func StartBreakAnimation(durationMin int) {
	totalDuration := time.Duration(durationMin) * time.Minute
	numBarSegments := 20
	barLength := numBarSegments * 2
	initialBar := strings.Repeat("o ", numBarSegments)
	startTime := time.Now()
	done := make(chan bool)
	animationTicker := time.NewTicker(50 * time.Millisecond)
	defer animationTicker.Stop()
	secondTicker := time.NewTicker(time.Second)
	defer secondTicker.Stop()
	lastPrintedSeconds := -1
	go func() {
		for {
			select {
			case <-animationTicker.C:
				elapsedTime := time.Since(startTime)
				remainingTime := totalDuration - elapsedTime
				if remainingTime < 0 {
					remainingTime = 0
				}
				minutes := int(remainingTime.Minutes())
				seconds := int(remainingTime.Seconds()) % 60
				progress := float64(elapsedTime) / float64(totalDuration)
				pos := int(progress * float64(barLength))
				if pos > barLength {
					pos = barLength
				}
				currentBar := []rune(initialBar)
				for j := 0; j < pos; j++ {
					if initialBar[j] == 'o' && currentBar[j] != ' ' {
						currentBar[j] = ' '
					}
				}
				if pos < barLength {
					if currentBar[pos] == ' ' {
						currentBar[pos] = 'C'
					} else {
						currentBar[pos] = 'c'
					}
				} else {
					for j := 0; j < barLength; j++ {
						currentBar[j] = ' '
					}
					currentBar = append(currentBar, 'c')
				}
				if seconds != lastPrintedSeconds {
					fmt.Printf("\r[%02d:%02d] [%s]", minutes, seconds, strings.TrimRight(string(currentBar), " "))
					lastPrintedSeconds = seconds
				}
				if remainingTime <= 0 {
					done <- true
					return
				}
			case <-secondTicker.C:
				elapsedTime := time.Since(startTime)
				remainingTime := totalDuration - elapsedTime
				if remainingTime < 0 {
					remainingTime = 0
				}
				minutes := int(remainingTime.Minutes())
				seconds := int(remainingTime.Seconds()) % 60
				progress := float64(elapsedTime) / float64(totalDuration)
				pos := int(progress * float64(barLength))
				if pos > barLength {
					pos = barLength
				}
				currentBar := []rune(initialBar)
				for j := 0; j < pos; j++ {
					if initialBar[j] == 'o' && currentBar[j] != ' ' {
						currentBar[j] = ' '
					}
				}
				if pos < barLength {
					if currentBar[pos] == ' ' {
						currentBar[pos] = 'C'
					} else {
						currentBar[pos] = 'c'
					}
				} else {
					for j := 0; j < barLength; j++ {
						currentBar[j] = ' '
					}
					currentBar = append(currentBar, 'c')
				}
				fmt.Printf("\r[%02d:%02d] [%s]", minutes, seconds, strings.TrimRight(string(currentBar), " "))
				if remainingTime <= 0 {
					done <- true
					return
				}
			}
		}
	}()
	<-done
	fmt.Println()
}
