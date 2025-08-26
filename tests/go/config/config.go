package config

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/spf13/viper"
)

type Config struct {
	SolanaRPCURL          string
	BackendGRPCEndpoint   string
	BackendGRPCPort       int
	TestAccountAddress    string
	ValidatorStartTimeout int
	BackendStartTimeout   int
}

func GetConfig(configFileName string) (*Config, error) {
	v := viper.New()

	// Set defaults
	v.SetDefault("SolanaRPCURL", "http://localhost:8899")
	v.SetDefault("BackendGRPCEndpoint", "localhost")
	v.SetDefault("BackendGRPCPort", 50051)
	v.SetDefault("TestAccountAddress", "5MvYgrb6DDznpeqejPzkJSxj7cBCu4UjTRVb1saMsGPr")
	v.SetDefault("ValidatorStartTimeout", 60) // seconds
	v.SetDefault("BackendStartTimeout", 30)   // seconds

	// Find api-test root by walking up the directory tree
	configPath, err := findAPITestRoot()
	if err != nil {
		return nil, err
	}

	// Set exact config file path
	configFile := filepath.Join(configPath, configFileName)

	// Only read config file if it exists (optional)
	if _, err := os.Stat(configFile); err == nil {
		v.SetConfigFile(configFile)
		if err := v.ReadInConfig(); err != nil {
			return nil, fmt.Errorf("error reading config file %s: %w", configFile, err)
		}
	}

	// Override with environment variables if set
	if rpcURL := os.Getenv("SOLANA_RPC_URL"); rpcURL != "" {
		v.Set("SolanaRPCURL", rpcURL)
	}

	// Unmarshal into struct
	var config Config
	if err := v.Unmarshal(&config); err != nil {
		return nil, fmt.Errorf("error unmarshaling config: %w", err)
	}

	return &config, nil
}

func findAPITestRoot() (string, error) {
	wd, err := os.Getwd()
	if err != nil {
		return "", err
	}

	for {
		// Check if current directory is api-test or contains api-test
		if filepath.Base(wd) == "api-test" {
			return wd, nil
		}

		// Also check if we're in a subdirectory of api-test
		if apiTestPath := filepath.Join(wd, "api-test"); isDir(apiTestPath) {
			return apiTestPath, nil
		}

		// Move up one directory
		parent := filepath.Dir(wd)
		if parent == wd {
			break // reached filesystem root
		}
		wd = parent
	}

	return "", fmt.Errorf("api-test directory not found in directory tree")
}

func isDir(path string) bool {
	info, err := os.Stat(path)
	return err == nil && info.IsDir()
}

// GetTestKeypairPath returns the path to the test keypair file
func GetTestKeypairPath() (string, error) {
	// Look for test keypair in the scripts directory
	wd, err := os.Getwd()
	if err != nil {
		return "", err
	}

	// Navigate up to find the project root, then locate the keypair
	for {
		// Check for protosol project root markers
		if hasProtosolMarkers(wd) {
			keypairPath := filepath.Join(wd, "project", "solana", "scripts", "test-keypair.json")
			if _, err := os.Stat(keypairPath); err == nil {
				return keypairPath, nil
			}
		}

		parent := filepath.Dir(wd)
		if parent == wd {
			break
		}
		wd = parent
	}

	return "", fmt.Errorf("test keypair not found, expected at project/solana/scripts/test-keypair.json")
}

func hasProtosolMarkers(dir string) bool {
	// Check for known project files/directories that indicate protosol root
	markers := []string{
		"CLAUDE.md",
		"project/solana",
		"dev/tool.sh",
	}

	for _, marker := range markers {
		if _, err := os.Stat(filepath.Join(dir, marker)); err == nil {
			return true
		}
	}
	return false
}
