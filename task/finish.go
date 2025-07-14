package task

import (
	"encoding/json"
	"fmt"
	"os/exec"
	"tasky/config"
	"tasky/utils"
	"time"
)

// FinishTask closes a GitHub issue, merges the PR, and updates the task note.
func FinishTask(cfg config.Config) error {
	// 1. Get current branch name
	branchName, err := utils.GetCurrentBranchName()
	if err != nil {
		return fmt.Errorf("failed to get current branch name: %w", err)
	}

	// 2. Extract issue number from branch name
	issueNumber := utils.ExtractIssueNumberFromBranch(branchName)
	if issueNumber == "" {
		fmt.Printf("No issue number found in branch name: %s\n", branchName)
		return nil
	}

	fmt.Printf("Found GitHub issue number: %s in branch: %s\n", issueNumber, branchName)

	// 3. GitHub operations: Push, Create PR and merge
	fmt.Println("Pushing branch to remote...")
	pushCmd := exec.Command("git", "push")
	pushCmd.Stdout = utils.NewLogWriter("git push", false)
	pushCmd.Stderr = utils.NewLogWriter("git push", true)
	if err := pushCmd.Run(); err != nil {
		return fmt.Errorf("failed to push branch: %w", err)
	}

	fmt.Println("Creating GitHub pull request...")
	// Get issue title
	issueTitleCmd := exec.Command("gh", "issue", "view", issueNumber, "--json", "title")
	issueTitleOutput, err := issueTitleCmd.Output()
	if err != nil {
		return fmt.Errorf("failed to get issue title: %w", err)
	}

	// Parse JSON output to extract title
	var result struct{ Title string }
	if err := json.Unmarshal(issueTitleOutput, &result); err != nil {
		return fmt.Errorf("failed to parse issue title JSON: %w", err)
	}
	prTitle := result.Title

	fmt.Println("Executing: gh pr create --title \"" + prTitle + "\" --body \"Closes #" + issueNumber + "\"")
	createPRCmd := exec.Command("gh", "pr", "create", "--title", prTitle, "--body", "Closes #"+issueNumber)
	createPRCmd.Stdout = utils.NewLogWriter("gh pr create", false)
	createPRCmd.Stderr = utils.NewLogWriter("gh pr create", true)
	if err := createPRCmd.Run(); err != nil {
		return fmt.Errorf("failed to create GitHub pull request: %w", err)
	}

	fmt.Println("Merging GitHub pull request and deleting branch...")
	// Explain the gh pr merge command
	fmt.Println("Executing: gh pr merge -d -s")
	mergePRCmd := exec.Command("gh", "pr", "merge", "-d", "-s")
	mergePRCmd.Stdout = utils.NewLogWriter("gh pr merge", false)
	mergePRCmd.Stderr = utils.NewLogWriter("gh pr merge", true)
	if err := mergePRCmd.Run(); err != nil {
		return fmt.Errorf("failed to merge GitHub pull request: %w", err)
	}

	// 4. Find and update Markdown file
	projectName := utils.GetProjectName()
	if projectName == "unknown_project" {
		return fmt.Errorf("could not determine project name. Please run this command in a Git repository")
	}

	projectTasksPath, err := utils.GetTaskyDir(cfg, projectName)
	if err != nil {
		return err
	}

	files, err := utils.ListMarkdownFiles(projectTasksPath)
	if err != nil {
		return fmt.Errorf("failed to list markdown files: %w", err)
	}

	var foundTask *config.Task
	var foundPath string
	for _, file := range files {
		t, _, err := ReadTaskFile(cfg, projectName, file)
		if err == nil && fmt.Sprintf("%d", t.Issue) == issueNumber {
			foundTask = t
			foundPath = file
			break
		}
	}

	if foundTask == nil {
		fmt.Printf("No task note found with GitHub issue #%s.\n", issueNumber)
		return nil
	}

	foundTask.Status = config.StatusDone
	foundTask.DoneDate = time.Now().Format("2006-01-02")

	_, descriptionPart, err := ReadTaskFile(cfg, projectName, foundPath)
	if err != nil {
		return fmt.Errorf("failed to read file %s for update: %w", foundPath, err)
	}

	if err := WriteTaskFile(cfg, projectName, foundPath, foundTask, descriptionPart); err != nil {
		return fmt.Errorf("failed to write updated file %s: %w", foundPath, err)
	}

	return nil
}

