package task

import (
	"fmt"
		"os/exec"

	"tasky/utils"
)

func StartTaskDevelopment(issueNumber string) error {
	if !utils.IsGitRepository() {
		return fmt.Errorf("not in a Git repository. Cannot start development on a GitHub issue.")
	}

	cmd := exec.Command("gh", "issue", "develop", issueNumber, "--checkout")
	cmd.Stdout = utils.NewLogWriter("gh issue develop", false)
	cmd.Stderr = utils.NewLogWriter("gh issue develop", true)

	fmt.Printf("Starting development for issue #%s...\n", issueNumber)
	err := cmd.Run()
	if err != nil {
		return fmt.Errorf("error starting development for issue #%s: %w", issueNumber, err)
	}

	fmt.Printf("Successfully started development for issue #%s.\n", issueNumber)
	return nil
}