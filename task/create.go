package task

import (
	"fmt"
	"os"
	"os/exec"
	"regexp"
	"strings"
	"time"

	"tasky/config"
	"tasky/utils"
)

func CreateTask(cfg config.Config, title, description string, createGitHubIssue bool) (string, error) {
	task := config.Task{
		Frontmatter: config.Frontmatter{
			Title:		title,
			Status:		config.StatusTodo,
			CreatedDate: time.Now().Format("2006-01-02 15:04:05"),
		},
	}

	projectName := utils.GetProjectName()

	// Format filename
	safeTitle := strings.ReplaceAll(strings.ToLower(title), " ", "-")
	baseFilename := fmt.Sprintf("%s.md", safeTitle)
	filename := baseFilename
	filePath := filename

	var createdIssueNumber string

	// Create GitHub issue if requested and possible
	if createGitHubIssue && utils.IsGitRepository() && utils.HasGitHubRemote() {
		cmd := exec.Command("gh", "issue", "create", "--title", title, "--body", description)
		output, err := cmd.CombinedOutput()
		if err != nil {
			return "", fmt.Errorf("error creating GitHub issue: %w\n%s", err, string(output))
		}
		fmt.Println("GitHub issue created successfully.")
		fmt.Println(string(output))

		// Extract issue number from gh output
		outputStr := string(output)
		re := regexp.MustCompile(`https://github.com/.*/issues/([0-9]+)`) 
		matches := re.FindStringSubmatch(outputStr)
		if len(matches) > 1 {
			issueNumber := 0
			fmt.Sscanf(matches[1], "%d", &issueNumber)
			createdIssueNumber = matches[1]
			task.Issue = issueNumber // Update task struct with issue number
		}
	}

	// If an issue was created, update filename and filePath
	if createdIssueNumber != "" {
		filename = fmt.Sprintf("%s-%s", createdIssueNumber, baseFilename)
				filePath = filename
	}

	// Check for existing file and add a number if necessary (after potential issue number prefix)
	for i := 1; ; i++ {
		if _, err := os.Stat(filePath); os.IsNotExist(err) {
			break
		}
		// If issue number was added, ensure it's preserved in subsequent attempts
		if createdIssueNumber != "" {
			filename = fmt.Sprintf("%s-%s-%d.md", createdIssueNumber, safeTitle, i)
		} else {
			filename = fmt.Sprintf("%s-%d.md", safeTitle, i)
		}
				filePath = filename
	}

	// Write initial file
	if err := WriteTaskFile(cfg, projectName, filePath, &task, description); err != nil {
		return "", fmt.Errorf("error creating file: %w", err)
	}

	fmt.Printf("Task created: %s\n", filePath)

	return createdIssueNumber, nil
}