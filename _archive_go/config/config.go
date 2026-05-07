package config

import (
	"bufio"
	"fmt"
	"os"
	"os/user"
	"path/filepath"
	"strings"

	"github.com/BurntSushi/toml"
)

type General struct {
	VaultPath string `toml:"vault_path"`
}

type Pomodoro struct {
	PomodoroDuration   int `toml:"pomodoro_duration,omitempty"`
	ShortBreakDuration int `toml:"short_break_duration,omitempty"`
	LongBreakDuration  int `toml:"long_break_duration,omitempty"`
	LongBreakInterval  int `toml:"long_break_interval,omitempty"`
}

type Sounds struct {
	Break string `toml:"break,omitempty"`
	Start string `toml:"start,omitempty"`
	Done  string `toml:"done,omitempty"`
}

type Config struct {
	General  General  `toml:"general"`
	Pomodoro Pomodoro `toml:"pomodoro"`
	Sounds   Sounds   `toml:"sounds"`
}

type Frontmatter struct {
	Title         string `yaml:"title"`
	Status        string `yaml:"status"`
	CreatedDate   string `yaml:"created_date"`
	DoneDate      string `yaml:"done_date,omitempty"`
	StartDate     string `yaml:"start_date,omitempty"`
	PomodoroCount int    `yaml:"pomodoro_count"`
	Issue         int    `yaml:"issue,omitempty"`
	Duration int    `yaml:"duration,omitempty"` // in minutes
}

const (
	StatusTodo       = "todo"
	StatusInProgress = "in progress"
	StatusDone       = "done"
)

const (
	SymbolTodo  = "☐"
	SymbolDoing = "➜"
	SymbolDone  = "✓"
)

type Task struct {
	Frontmatter `yaml:",inline"`
}

func getConfigPath() (string, error) {
	usr, err := user.Current()
	if err != nil {
		return "", err
	}
	configDir := filepath.Join(usr.HomeDir, ".config", "tasky")
	if err := os.MkdirAll(configDir, 0755); err != nil {
		return "", err
	}
	return filepath.Join(configDir, "config.toml"), nil
}

func LoadConfig() Config {
	var cfg Config
	configPath, err := getConfigPath()
	if err != nil {
		panic("Error getting config path: " + err.Error())
	}

	// Set initial default Pomodoro values
	cfg.Pomodoro.PomodoroDuration = 25
	cfg.Pomodoro.ShortBreakDuration = 5
	cfg.Pomodoro.LongBreakDuration = 15
	cfg.Pomodoro.LongBreakInterval = 4

	shouldSaveConfig := false // Flag to track if we need to save the config

	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		// Config file does not exist, this is the first run
		fmt.Print("Welcome to Tasky! Press Enter to use the default path (~/Documents/Obsidian) or enter a custom path: ")
		reader := bufio.NewReader(os.Stdin)
		newPath, _ := reader.ReadString('\n')
		newPath = strings.TrimSpace(newPath)

		usr, err := user.Current()
		if err != nil {
			panic("Error getting current user: " + err.Error())
		}
		homeDir := usr.HomeDir

		if newPath == "" {
			newPath = filepath.Join(homeDir, "Documents", "Obsidian")
		} else if strings.HasPrefix(newPath, "~/") {
			newPath = filepath.Join(homeDir, newPath[2:])
		}

		absPath, err := filepath.Abs(newPath)
		if err != nil {
			panic("Error converting to absolute path: " + err.Error())
		}

		cfg.General.VaultPath = absPath
		shouldSaveConfig = true // New config, so save it
		fmt.Println("Configuration saved.")
	} else {
		// Config file exists, load it
		var loadedCfg Config
		if _, err := toml.DecodeFile(configPath, &loadedCfg); err != nil {
			panic("Error loading config.toml: " + err.Error())
		}

		// Overwrite defaults with loaded values
		cfg.General.VaultPath = loadedCfg.General.VaultPath
		cfg.Sounds = loadedCfg.Sounds

		// For Pomodoro settings, if the loaded value is 0, it means it was missing or explicitly 0 in the file.
		// In this case, we keep our default. If it's non-zero, we use the loaded value.
		// And if we use our default, we should save the config back.
		if loadedCfg.Pomodoro.PomodoroDuration != 0 {
			cfg.Pomodoro.PomodoroDuration = loadedCfg.Pomodoro.PomodoroDuration
		} else {
			shouldSaveConfig = true // Default was used, so save
		}
		if loadedCfg.Pomodoro.ShortBreakDuration != 0 {
			cfg.Pomodoro.ShortBreakDuration = loadedCfg.Pomodoro.ShortBreakDuration
		} else {
			shouldSaveConfig = true // Default was used, so save
		}
		if loadedCfg.Pomodoro.LongBreakDuration != 0 {
			cfg.Pomodoro.LongBreakDuration = loadedCfg.Pomodoro.LongBreakDuration
		} else {
			shouldSaveConfig = true // Default was used, so save
		}
		if loadedCfg.Pomodoro.LongBreakInterval != 0 {
			cfg.Pomodoro.LongBreakInterval = loadedCfg.Pomodoro.LongBreakInterval
		} else {
			shouldSaveConfig = true // Default was used, so save
		}
	}

	if shouldSaveConfig {
		SaveConfig(cfg)
	}

	return cfg
}

func SaveConfig(cfg Config) {
	configPath, err := getConfigPath()
	if err != nil {
		panic("Error getting config path: " + err.Error())
	}

	f, err := os.Create(configPath)
	if err != nil {
		panic("Error creating config.toml: " + err.Error())
	}
	defer f.Close()

	if err := toml.NewEncoder(f).Encode(cfg); err != nil {
		panic("Error encoding config to TOML: " + err.Error())
	}
}
