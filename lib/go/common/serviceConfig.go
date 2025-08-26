package common

import (
	"os"
	"path/filepath"
	"runtime"
	"time"

	"google.golang.org/grpc"
)

// ServiceConfig holds the configuration for a gRPC service client
type ServiceConfig struct {
	URL               string
	TLS               bool
	Timeout           time.Duration
	APIKey            string
	CredentialsFile   string
	UnaryInterceptors []grpc.UnaryClientInterceptor
}

// ServiceOption is a functional option for configuring a gRPC service client
type ServiceOption func(*ServiceConfig)

// WithURL sets the gRPC server URL (e.g., "api.example.com:443")
func WithURL(url string) ServiceOption {
	return func(c *ServiceConfig) {
		c.URL = url
	}
}

// WithTLS enables or disables TLS for the connection
func WithTLS(enabled bool) ServiceOption {
	return func(c *ServiceConfig) {
		c.TLS = enabled
	}
}

// WithTimeout sets the default timeout for RPC calls
func WithTimeout(timeout time.Duration) ServiceOption {
	return func(c *ServiceConfig) {
		c.Timeout = timeout
	}
}

// WithAPIKey sets the API key for authentication
func WithAPIKey(apiKey string) ServiceOption {
	return func(c *ServiceConfig) {
		c.APIKey = apiKey
	}
}

// WithCredentialsFile sets the path to a credentials file
func WithCredentialsFile(path string) ServiceOption {
	return func(c *ServiceConfig) {
		c.CredentialsFile = path
	}
}

// WithUnaryInterceptor adds a unary client interceptor
func WithUnaryInterceptor(interceptor grpc.UnaryClientInterceptor) ServiceOption {
	return func(c *ServiceConfig) {
		c.UnaryInterceptors = append(c.UnaryInterceptors, interceptor)
	}
}

// WithInsecure is a convenience option to disable TLS (for development)
func WithInsecure() ServiceOption {
	return WithTLS(false)
}

// WithSecure is a convenience option to enable TLS (for production)
func WithSecure() ServiceOption {
	return WithTLS(true)
}

// discoverCredentials attempts to find API credentials using the standard discovery hierarchy:
//
// 1. PROTOSOL_API_CREDENTIALS environment variable
// 2. Default credential file location:
//   - Linux:   $XDG_CONFIG_HOME/protosol/credentials.json or fallback to $HOME/.config/protosol/credentials.json
//   - macOS:   $HOME/Library/Application Support/protosol/credentials.json
//   - Windows: C:\Users\<user>\AppData\Roaming\protosol\credentials.json
//
// This follows the same pattern as other cloud SDKs for credential discovery.
func discoverCredentials() string {
	// Check environment variable first
	if creds := os.Getenv("PROTOSOL_API_CREDENTIALS"); creds != "" {
		return creds
	}

	// Determine default path based on OS
	var defaultPath string
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return ""
	}

	switch runtime.GOOS {
	case "darwin": // macOS
		defaultPath = filepath.Join(homeDir, "Library", "Application Support", "protosol", "credentials.json")
	case "windows":
		defaultPath = filepath.Join(homeDir, "AppData", "Roaming", "protosol", "credentials.json")
	default: // Linux and others
		// Use XDG_CONFIG_HOME if set, otherwise fallback to ~/.config
		configHome := os.Getenv("XDG_CONFIG_HOME")
		if configHome == "" {
			configHome = filepath.Join(homeDir, ".config")
		}
		defaultPath = filepath.Join(configHome, "protosol", "credentials.json")
	}

	// Check if the default path exists
	if _, err := os.Stat(defaultPath); err == nil {
		return defaultPath
	}

	return ""
}

// WithDefaultCredentials attempts to discover and use default credentials
func WithDefaultCredentials() ServiceOption {
	return func(c *ServiceConfig) {
		if path := discoverCredentials(); path != "" {
			c.CredentialsFile = path
		}
	}
}

// WithProductionDefaults configures the client with production-ready defaults
func WithProductionDefaults() ServiceOption {
	return func(c *ServiceConfig) {
		c.TLS = true
		c.Timeout = 30 * time.Second
		// Attempt to discover default credentials
		if path := discoverCredentials(); path != "" {
			c.CredentialsFile = path
		}
	}
}

// WithDevelopmentDefaults configures the client with development-friendly defaults
func WithDevelopmentDefaults() ServiceOption {
	return func(c *ServiceConfig) {
		c.TLS = false
		c.Timeout = 10 * time.Second
		c.URL = "localhost:9090"
	}
}
