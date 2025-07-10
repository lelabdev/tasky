package task

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"time"

	"gopkg.in/yaml.v3"

	"tasky/config"
	"tasky/utils"
)

// readTaskFile reads a markdown file, extracts its YAML frontmatter, and unmarshals it into a config.Task.
// It returns the task, the remaining markdown content, and an error if any.
func readTaskFile(filePath string) (*config.Task, string, error) {
	fullContent, err := os.ReadFile(filePath)
	if err != nil {
		return nil, "", fmt.Errorf("error reading file %s: %w", filePath, err)
	}

	text := string(fullContent)

	// Extract YAML frontmatter directly
	re := regexp.MustCompile(`(?s)^---
(.*?)
---
`)
	matches := re.FindStringSubmatch(text)

	var yamlContent string
	if len(matches) > 1 {
		yamlContent = matches[1]
	} else {
		return nil, "", fmt.Errorf("no YAML frontmatter found in %s", filePath)
	}

	var t config.Task
	if err := yaml.Unmarshal([]byte(yamlContent), &t); err != nil {
		return nil, "", fmt.Errorf("error unmarshalling YAML from %s: %w", filePath, err)
	}

	endYaml := strings.Index(text[3:], "---") + 3
	descriptionPart := strings.TrimSpace(text[endYaml+3:])

	return &t, descriptionPart, nil
}

// writeTaskFile marshals a config.Task back into YAML frontmatter, combines it with the markdown content,
// and writes it back to the file.
func writeTaskFile(filePath string, task *config.Task, descriptionPart string) error {
	updatedYamlData, err := yaml.Marshal(&task)
	if err != nil {
		return fmt.Errorf("error marshalling updated YAML for %s: %w", filePath, err)
	}

	newContent := fmt.Sprintf("---\n%s---\n\n%s", string(updatedYamlData), descriptionPart)

	return os.WriteFile(filePath, []byte(newContent), 0644)
}

func GetTasks(vaultPath string, filterProject string) []config.Task {
	var tasks []config.Task
	taskyBaseDir := filepath.Join(vaultPath, "Tasky")

	var walkPath string
	if filterProject != "" {
		walkPath = filepath.Join(taskyBaseDir, filterProject)
	} else {
		walkPath = taskyBaseDir
	}

	// Ensure the directory exists before trying to walk it
	if _, err := os.Stat(walkPath); os.IsNotExist(err) {
		return tasks
	}

	err := filepath.Walk(walkPath, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() || !strings.HasSuffix(info.Name(), ".md") {
			return nil
		}

		t, _, err := readTaskFile(path)
		if err == nil {
			tasks = append(tasks, *t)
		}

		return nil
	})

	if err != nil {
		fmt.Println("Error reading:", err)
	}

	return tasks
}

func MarkTaskDone(vaultPath, taskTitle string) {
	var foundTask *config.Task
	var foundPath string

	taskyBaseDir := filepath.Join(vaultPath, "Tasky")

	err := filepath.Walk(taskyBaseDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() || !strings.HasSuffix(info.Name(), ".md") {
			return nil
		}

		t, _, err := readTaskFile(path)
		if err == nil {
			if strings.EqualFold(t.Title, taskTitle) {
				foundTask = t
				foundPath = path
				return filepath.SkipDir // Found the task, stop walking
			}
		}

		return nil
	})

	if err != nil && err != filepath.SkipDir {
		fmt.Println("Error searching for task:", err)
		return
	}

	if foundTask == nil {
		fmt.Printf("Task '%s' not found.\n", taskTitle)
		return
	}

	if foundTask.Status == config.StatusDone {
		fmt.Printf("Task '%s' is already marked as done.\n", taskTitle)
		return
	}

	foundTask.Status = config.StatusDone
	foundTask.DoneDate = time.Now().Format("2006-01-02")

	_, descriptionPart, err := readTaskFile(foundPath)
	if err != nil {
		fmt.Println("Error reading task file for update:", err)
		return
	}

	if err := writeTaskFile(foundPath, foundTask, descriptionPart); err != nil {
		fmt.Println("Error writing updated task file:", err)
		return
	}

	fmt.Printf("Task '%s' marked as done.\n", taskTitle)
}

func MarkTaskInProgress(vaultPath string, issueNumber int) {
	var foundTask *config.Task
	var foundPath string

	taskyBaseDir := filepath.Join(vaultPath, "Tasky")

	err := filepath.Walk(taskyBaseDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() || !strings.HasSuffix(info.Name(), ".md") {
			return nil
		}

		t, _, err := readTaskFile(path)
		if err == nil {
			if t.Issue == issueNumber {
				foundTask = t
				foundPath = path
				return filepath.SkipDir // Found the task, stop walking
			}
		}

		return nil
	})

	if err != nil && err != filepath.SkipDir {
		fmt.Println("Error searching for task:", err)
		return
	}

	if foundTask == nil {
		fmt.Printf("Task with GitHub issue #%d not found.\n", issueNumber)
		return
	}

	if foundTask.Status == config.StatusInProgress {
		fmt.Printf("Task '%s' is already marked as in-progress.\n", foundTask.Title)
		return
	}

	foundTask.Status = config.StatusInProgress
	foundTask.StartDate = time.Now().Format("2006-01-02 15:04:05")

	_, descriptionPart, err := readTaskFile(foundPath)
	if err != nil {
		fmt.Println("Error reading task file for update:", err)
		return
	}

	if err := writeTaskFile(foundPath, foundTask, descriptionPart); err != nil {
		fmt.Println("Error writing updated task file:", err)
		return
	}

	fmt.Printf("Task '%s' marked as in-progress.\n", foundTask.Title)
}

// MarkTaskInProgressByTitle marks a task as in-progress using its title.
func MarkTaskInProgressByTitle(vaultPath, taskTitle string) {
	var foundTask *config.Task
	var foundPath string

	taskyBaseDir := filepath.Join(vaultPath, "Tasky")

	err := filepath.Walk(taskyBaseDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() || !strings.HasSuffix(info.Name(), ".md") {
			return nil
		}

		t, _, err := readTaskFile(path)
		if err == nil {
			if strings.EqualFold(t.Title, taskTitle) {
				foundTask = t
				foundPath = path
				return filepath.SkipDir // Found the task, stop walking
			}
		}

		return nil
	})

	if err != nil && err != filepath.SkipDir {
		fmt.Println("Error searching for task:", err)
		return
	}

	if foundTask == nil {
		fmt.Printf("Task '%s' not found.\n", taskTitle)
		return
	}

	if foundTask.Status == config.StatusInProgress {
		fmt.Printf("Task '%s' is already marked as in-progress.\n", taskTitle)
		return
	}

	foundTask.Status = config.StatusInProgress
	foundTask.StartDate = time.Now().Format("2006-01-02 15:04:05")

	_, descriptionPart, err := readTaskFile(foundPath)
	if err != nil {
		fmt.Println("Error reading task file for update:", err)
		return
	}

	if err := writeTaskFile(foundPath, foundTask, descriptionPart); err != nil {
		fmt.Println("Error writing updated task file:", err)
		return	}

	fmt.Printf("Task '%s' marked as in-progress.\n", taskTitle)
}

// IncrementPomodoroCountForActiveTask increments the Pomodoro counter of the active task (from the current branch issue)
func IncrementPomodoroCountForActiveTask(vaultPath string) error {
	branchName, err := utils.GetCurrentBranchName()
	if err != nil {
		return err
	}
	issueStr := utils.ExtractIssueNumberFromBranch(branchName)
	if issueStr == "" {
		return nil // No issue detected
	}
	var issueNumber int
	_, err = fmt.Sscanf(issueStr, "%d", &issueNumber)
	if err != nil {
		return err
	}
	projectName := utils.GetProjectName()
	projectTasksPath := filepath.Join(vaultPath, "Tasky", projectName)
	var foundPath string
	var foundTask *config.Task
	filepath.Walk(projectTasksPath, func(path string, info os.FileInfo, err error) error {
		if err != nil || info.IsDir() || !strings.HasSuffix(info.Name(), ".md") {
			return nil
		}

		t, _, err := readTaskFile(path)
		if err == nil && t.Issue == issueNumber {
			foundTask = t
			foundPath = path
			return filepath.SkipDir
		}
		return nil
	})
	if foundTask == nil {
		return nil // No task found
	}
	foundTask.PomodoroCount++
	// Update Duration (add PomodoroDuration minutes)
	minutesToAdd := 25 // default
	cfg := config.LoadConfig()
	if cfg.Pomodoro.PomodoroDuration > 0 {
		minutesToAdd = cfg.Pomodoro.PomodoroDuration
	}
	foundTask.Duration += minutesToAdd

	_, descriptionPart, err := readTaskFile(foundPath)
	if err != nil {
		return err
	}

	return writeTaskFile(foundPath, foundTask, descriptionPart)
}
