package cmd

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/urfave/cli/v2"
	"tasky/config"
	"tasky/utils"
)

// LinkCommand returns a *cli.Command for the "link" command.
func LinkCommand() *cli.Command {
	return &cli.Command{
		Name:  "link",
		Usage: "Create a symbolic link to the project's task directory in the current directory",
		Action: func(c *cli.Context) error {
			cfg := config.LoadConfig()
			projectName := utils.GetProjectName()
			if projectName == "unknown_project" {
				return cli.Exit("Could not determine project name. Please run this command in a Git repository.", 1)
			}

			targetPath := filepath.Join(cfg.General.VaultPath, "Tasky", projectName)
			linkPath := "tasky"

			// Check if the target directory exists
			if _, err := os.Stat(targetPath); os.IsNotExist(err) {
				return cli.Exit(fmt.Sprintf("Target task directory does not exist: %s", targetPath), 1)
			}

			// Check if the link already exists
			if _, err := os.Lstat(linkPath); err == nil {
				fmt.Printf("Symbolic link '%s' already exists. Removing it...\n", linkPath)
				if err := os.Remove(linkPath); err != nil {
                    return cli.Exit(fmt.Sprintf("Failed to remove existing link: %v", err), 1)
                }
            }

            // fmt.Printf("Creating symbolic link from '%s' to '%s'...\n", targetPath, linkPath)
            if err := os.Symlink(targetPath, linkPath); err != nil {
                return cli.Exit(fmt.Sprintf("Failed to create symbolic link: %v", err), 1)
            }

            fmt.Printf("Symbolic link '%s' created successfully.\n", linkPath)

			// Ask to add to .gitignore
			fmt.Print("Add 'tasky/' to your project's .gitignore? (Y/n): ")
			reader := bufio.NewReader(os.Stdin)
			input, _ := reader.ReadString('\n')
			trimmedInput := strings.ToLower(strings.TrimSpace(input))

			if trimmedInput == "y" || trimmedInput == "" {
				gitignorePath := ".gitignore"
				file, err := os.OpenFile(gitignorePath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
				if err != nil {
					return cli.Exit(fmt.Sprintf("Failed to open .gitignore: %v", err), 1)
				}
				defer file.Close()

				content := "\ntasky/\n"
				if _, err := file.WriteString(content); err != nil {
					return cli.Exit(fmt.Sprintf("Failed to write to .gitignore: %v", err), 1)
				}
				fmt.Println("'tasky/' added to .gitignore.")
			} else {
				fmt.Println("Skipping .gitignore update.")
			}

			return nil
		},
	}
}
