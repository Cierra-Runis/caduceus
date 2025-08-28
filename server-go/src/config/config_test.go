package config_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"server/src/config"

	"github.com/stretchr/testify/assert"
)

func TestLoadConfig(t *testing.T) {
	t.Run("load_config_success", func(t *testing.T) {
		env := "test"
		config, err := config.LoadConfig(env, "../../config")
		assert.NoError(t, err)
		assert.NotNil(t, config)
	})

	t.Run("load_config_file_not_found", func(t *testing.T) {
		env := "nonexistent"
		config, err := config.LoadConfig(env, "../../config")
		assert.Error(t, err)
		assert.Nil(t, config)
	})

	t.Run("load_config_validate_error", func(t *testing.T) {
		tmpDir := os.TempDir()
		tmpFile := filepath.Join(tmpDir, "missing_config.yaml")

		content := []byte("allowOrigins: []\nmongoUri: ''\ndbName: ''\naddress: ''\njwtSecret: ''\n")
		err := os.WriteFile(tmpFile, content, 0644)
		if err != nil {
			t.Fatalf("failed to write temp file: %v", err)
		}
		defer func() {
			if os.Remove(tmpFile) != nil {
				t.Logf("failed to remove temp file: %v", err)
			}
		}()

		_, err = config.LoadConfig("missing_config", tmpDir)
		if err == nil || !strings.Contains(err.Error(), "config validation failed") {
			t.Errorf("should return validation error, got %v", err)
		}
	})
}

func TestConfig_Validate(t *testing.T) {
	t.Run("valid_config", func(t *testing.T) {
		config := &config.Config{
			AllowOrigins: []string{"http://localhost:3000"},
			MongoURI:     "mongodb://localhost:27017",
			DBName:       "testdb",
			Address:      "http://localhost:8080",
			JWTSecret:    "secret",
		}
		err := config.Validate()
		assert.NoError(t, err)
	})

	t.Run("invalid_config_missing_fields", func(t *testing.T) {
		config := &config.Config{}
		err := config.Validate()
		assert.Error(t, err)
	})
}
