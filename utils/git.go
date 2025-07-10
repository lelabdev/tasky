package utils

import (
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
)

// IsGitRepository checks if the current directory is a Git repository.
func IsGitRepository() bool {
	cmd := exec.Command("git", "rev-parse", "--is-inside-work-tree")
	err := cmd.Run()
	return err == nil
}

// HasGitHubRemote checks if the current Git repository has a remote pointing to GitHub.
func HasGitHubRemote() bool {
	cmd := exec.Command("git", "remote", "-v")
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	return strings.Contains(string(output), "github.com")
}

// GetProjectName returns the Git repository name or the current directory name.
func GetProjectName() string {
	if IsGitRepository() {
		// Try to get the repository name from the remote URL
		cmd := exec.Command("git", "config", "--get", "remote.origin.url")
		output, err := cmd.Output()
		if err == nil {
			url := strings.TrimSpace(string(output))
			if strings.Contains(url, "github.com") {
				parts := strings.Split(url, ":")
				if len(parts) > 1 {
					repoPath := strings.TrimSuffix(parts[1], ".git")
					return filepath.Base(repoPath)
				}
			}
		}

		// Fallback to the top-level directory name of the Git repository
		cmd = exec.Command("git", "rev-parse", "--show-toplevel")
		output, err = cmd.Output()
		if err == nil {
			return filepath.Base(strings.TrimSpace(string(output)))
		}
	}

	// Fallback to the current directory name
	wd, err := os.Getwd()
	if err != nil {
		return "unknown_project"
	}
	return filepath.Base(wd)
}

// GetCurrentBranchName returns the current git branch name, or an error if not in a git repo.
func GetCurrentBranchName() (string, error) {
	cmd := exec.Command("git", "rev-parse", "--abbrev-ref", "HEAD")
	output, err := cmd.Output()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(output)), nil
}

// ExtractIssueNumberFromBranch extrait le numéro d'issue du nom de branche (ex: 123-feature-x -> 123).
func ExtractIssueNumberFromBranch(branchName string) string {
	re := regexp.MustCompile(`^([0-9]+)-`)
	matches := re.FindStringSubmatch(branchName)
	if len(matches) > 1 {
		return matches[1]
	}
	return ""
}
