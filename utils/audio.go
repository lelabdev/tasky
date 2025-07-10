package utils

import (
	"fmt"
	"os/exec"
)

// PlaySound plays a WAV file using the 'aplay' command.
// It checks if the file path is provided before attempting to play.
func PlaySound(filePath string) {
	if filePath == "" {
		return // No sound file specified
	}

	cmd := exec.Command("aplay", filePath)
	err := cmd.Run()
	if err != nil {
		fmt.Printf("Error playing sound %s: %v\n", filePath, err)
	}
}
