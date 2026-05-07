package utils

import (
	"fmt"
	"os/exec"
)

// PlaySound plays a WAV file using the 'aplay' command.
// It checks if the file path is provided before attempting to play.
func PlaySound(filePath string) error {
	if filePath == "" {
		return nil // No sound file specified
	}

	cmd := exec.Command("aplay", filePath)
	err := cmd.Run()
	if err != nil {
		return fmt.Errorf("error playing sound %s: %w", filePath, err)
	}
	return nil
}
