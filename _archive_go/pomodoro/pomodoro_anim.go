package pomodoro

import (
	"fmt"
	"strings"
	"tasky/config"
	"time"
)

// StartPomodoroAnimation displays a visual Pomodoro timer animation.
func StartPomodoroAnimation(cfg config.Config) {
	totalDuration := time.Duration(cfg.Pomodoro.PomodoroDuration) * time.Minute
	if cfg.Pomodoro.PomodoroDuration <= 0 {
		fmt.Println("[WARN] Pomodoro duration is not set or invalid, using default 25 minutes.")
		totalDuration = 25 * time.Minute
	}
	numBarSegments := 20
	barLength := numBarSegments * 2 // Each 'o' and its space

	initialBar := strings.Repeat("o ", numBarSegments)

	startTime := time.Now()
	done := make(chan bool)

	segmentDuration := totalDuration / time.Duration(numBarSegments)
	segmentsElapsed := int(time.Since(startTime) / segmentDuration)

	animationTicker := time.NewTicker(50 * time.Millisecond)
	defer animationTicker.Stop()

	secondTicker := time.NewTicker(time.Second)
	defer secondTicker.Stop()

	lastPrintedSeconds := -1 // To ensure initial print and updates only on second change

	go func() {
		for {
			select {
			case <-animationTicker.C:
				// Update animation based on elapsed time
				elapsedTime := time.Since(startTime)
				remainingTime := totalDuration - elapsedTime
				if remainingTime < 0 {
					remainingTime = 0
				}

				minutes := int(remainingTime.Minutes())
				seconds := int(remainingTime.Seconds()) % 60

				// Calculate Pac-Man's position based on elapsed time
				// The Pac-Man moves across `barLength` positions over `totalDuration`
				pacManProgress := float64(elapsedTime) / float64(totalDuration)
				pacManPosition := int(pacManProgress * float64(barLength))

				// Ensure Pac-Man doesn't go out of bounds
				if pacManPosition > barLength {
					pacManPosition = barLength
				}

				// Build the current state of the bar
				currentBar := []rune(initialBar)
				for j := 0; j < segmentsElapsed && j < len(currentBar); j++ {
					currentBar[j*2] = ' ' // chaque 'o' est à j*2 (car il y a un espace entre chaque)
				}

				pacManIndex := segmentsElapsed * 2
				if pacManIndex >= len(currentBar) {
					pacManIndex = len(currentBar) - 1
				}
				if currentBar[pacManIndex] == ' ' {
					currentBar[pacManIndex] = 'C'
				} else {
					currentBar[pacManIndex] = 'c'
				}

				// Determine Pac-Man's position and state
				if pacManPosition < barLength { // Pac-Man is still moving within the bar
					if currentBar[pacManPosition] == ' ' { // Pac-Man is just before an 'o'
						currentBar[pacManPosition] = 'C'
					} else { // Pac-Man is on an 'o'
						currentBar[pacManPosition] = 'c'
					}
					// Replace eaten 'o's with spaces
					for j := 0; j < pacManPosition; j++ {
						if initialBar[j] == 'o' && currentBar[j] != ' ' {
							currentBar[j] = ' '
						}
					}
				} else { // Pac-Man has finished eating all 'o's and is at the very end
					for j := 0; j < barLength; j++ {
						currentBar[j] = ' '
					}
					currentBar = append(currentBar, 'c') // Add 'c' at the end
				}

				// Print the line only if seconds have changed or it's the first print
				if seconds != lastPrintedSeconds {
					fmt.Printf("\r[%02d:%02d] [%s]", minutes, seconds, strings.TrimRight(string(currentBar), " "))
					lastPrintedSeconds = seconds
				}

				if remainingTime <= 0 {
					done <- true
					return
				}

			case <-secondTicker.C:
				// This ticker ensures the time display is accurate every second
				// The animation ticker also updates the time, but this ensures it's always in sync
				elapsedTime := time.Since(startTime)
				remainingTime := totalDuration - elapsedTime
				if remainingTime < 0 {
					remainingTime = 0
				}
				minutes := int(remainingTime.Minutes())
				seconds := int(remainingTime.Seconds()) % 60

				// Recalculate Pac-Man's position for the second ticker as well
				pacManProgress := float64(elapsedTime) / float64(totalDuration)
				pacManPosition := int(pacManProgress * float64(barLength))
				if pacManPosition > barLength {
					pacManPosition = barLength
				}

				currentBar := []rune(initialBar)
				if pacManPosition < barLength {
					if currentBar[pacManPosition] == ' ' {
						currentBar[pacManPosition] = 'C'
					} else {
						currentBar[pacManPosition] = 'c'
					}
					for j := 0; j < pacManPosition; j++ {
						if initialBar[j] == 'o' && currentBar[j] != ' ' {
							currentBar[j] = ' '
						}
					}
				} else {
					for j := 0; j < barLength; j++ {
						currentBar[j] = ' '
					}
					currentBar = append(currentBar, 'c')
				}

				// Always print on second ticker to ensure time accuracy
				fmt.Printf("\r[%02d:%02d] [%s]", minutes, seconds, strings.TrimRight(string(currentBar), " "))

				if remainingTime <= 0 {
					done <- true
					return
				}
			}
		}
	}()

	<-done // Wait for the animation to finish

	fmt.Println() // New line after animation finishes
}
