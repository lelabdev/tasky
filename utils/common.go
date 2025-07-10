package utils

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

// RunCmd executes a shell command and returns its stdout. If the command fails, it returns an error.
// This comment is added to force a recompile.
func RunCmd(name string, arg ...string) (string, error) {
	cmd := exec.Command(name, arg...)
	var out bytes.Buffer
	var stderr bytes.Buffer
	cmd.Stdout = &out
	cmd.Stderr = &stderr

	err := cmd.Run()
	if err != nil {
		return "", fmt.Errorf("command failed: %s %s\nStdout: %s\nStderr: %s\nError: %w", name, strings.Join(arg, " "), out.String(), stderr.String(), err)
	}
	return strings.TrimSpace(out.String()), nil
}

// LogWriter is an io.Writer that logs output to stdout or stderr.
type LogWriter struct {
	isStderr bool
}

// NewLogWriter creates a new LogWriter.
func NewLogWriter(prefix string, isStderr bool) *LogWriter {
	return &LogWriter{isStderr: isStderr}
}

func (lw *LogWriter) Write(p []byte) (n int, err error) {
	if lw.isStderr {
		fmt.Fprintf(os.Stderr, "%s", p)
	} else {
		fmt.Fprintf(os.Stdout, "%s", p)
	}
	return len(p), nil
}

// ListMarkdownFiles lists all markdown files in a given directory and its subdirectories.
func ListMarkdownFiles(root string) ([]string, error) {
	var files []string
	err := filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && strings.HasSuffix(info.Name(), ".md") {
			files = append(files, path)
		}
		return nil
	})
	return files, err
}

// SimulatePauseKey simulates a media play-pause action using playerctl (Linux only)
func SimulatePauseKey() error {
	cmd := exec.Command("playerctl", "play-pause")
	return cmd.Run()
}
