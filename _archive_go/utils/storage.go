package utils

import (
	"fmt"
	"os"
	"path/filepath"

	"tasky/config"
)

// GetTaskyDir returns the absolute path to the Tasky/ directory, creating it if it doesn't exist.
func GetTaskyDir(cfg config.Config, projectName string) (string, error) {
	taskyDir := filepath.Join(cfg.General.VaultPath, projectName, "Tasky")
	if _, err := os.Stat(taskyDir); os.IsNotExist(err) {
		if err := os.MkdirAll(taskyDir, 0755); err != nil {
			return "", fmt.Errorf("could not create Tasky directory: %w", err)
		}
	}
	return taskyDir, nil
}

// CreateTaskyFile creates a new file inside the Tasky/ directory.
func CreateTaskyFile(cfg config.Config, projectName string, fileName string) (*os.File, error) {
	taskyDir, err := GetTaskyDir(cfg, projectName)
	if err != nil {
		return nil, err
	}
	filePath := filepath.Join(taskyDir, fileName)
	return os.Create(filePath)
}

// WriteToTaskyFile writes content to a file within the Tasky/ directory.
func WriteToTaskyFile(cfg config.Config, projectName string, fileName string, content []byte) error {
	taskyDir, err := GetTaskyDir(cfg, projectName)
	if err != nil {
		return err
	}
	filePath := filepath.Join(taskyDir, fileName)
	return os.WriteFile(filePath, content, 0644)
}

// ReadFromTaskyFile reads content from a file within the Tasky/ directory.
func ReadFromTaskyFile(cfg config.Config, projectName string, fileName string) ([]byte, error) {
	taskyDir, err := GetTaskyDir(cfg, projectName)
	if err != nil {
		return nil, err
	}
	filePath := filepath.Join(taskyDir, fileName)
	return os.ReadFile(filePath)
}
