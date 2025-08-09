package config

import (
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestLoadConfig(t *testing.T) {
	t.Run("load_config_with_env_file", func(t *testing.T) {
		// Create a temporary .env file
		envContent := `FRONTEND=http://test:3001
MONGO_URI=mongodb://test:27017
DB_NAME=test_db
PORT=8080
APP_MODE=production
JWT_SECRET=test_secret`

		err := os.WriteFile(".env", []byte(envContent), 0644)
		assert.NoError(t, err)
		defer os.Remove(".env")

		config, err := LoadConfig()
		assert.NoError(t, err)
		assert.NotNil(t, config)
		assert.Equal(t, "http://test:3001", config.Frontend)
		assert.Equal(t, "mongodb://test:27017", config.MongoURI)
		assert.Equal(t, "test_db", config.DBName)
		assert.Equal(t, "8080", config.Port)
		assert.Equal(t, "production", config.AppMode)
		assert.Equal(t, "test_secret", config.JWTSecret)
	})

	t.Run("load_config_without_env_file", func(t *testing.T) {
		// Ensure no .env file exists
		os.Remove(".env")

		// Clear environment variables
		originalVars := map[string]string{
			"FRONTEND":   os.Getenv("FRONTEND"),
			"MONGO_URI":  os.Getenv("MONGO_URI"),
			"DB_NAME":    os.Getenv("DB_NAME"),
			"PORT":       os.Getenv("PORT"),
			"APP_MODE":   os.Getenv("APP_MODE"),
			"JWT_SECRET": os.Getenv("JWT_SECRET"),
		}

		for key := range originalVars {
			os.Unsetenv(key)
		}

		defer func() {
			for key, val := range originalVars {
				if val != "" {
					os.Setenv(key, val)
				}
			}
		}()

		config, err := LoadConfig()
		// When .env file doesn't exist, LoadConfig will return an error
		assert.Error(t, err)
		assert.Nil(t, config)
	})

	t.Run("load_config_with_env_variables", func(t *testing.T) {
		// Set environment variables
		os.Setenv("FRONTEND", "http://env:3001")
		os.Setenv("MONGO_URI", "mongodb://env:27017")
		os.Setenv("DB_NAME", "env_db")
		os.Setenv("PORT", "9000")
		os.Setenv("APP_MODE", "release")
		os.Setenv("JWT_SECRET", "env_secret")

		defer func() {
			os.Unsetenv("FRONTEND")
			os.Unsetenv("MONGO_URI")
			os.Unsetenv("DB_NAME")
			os.Unsetenv("PORT")
			os.Unsetenv("APP_MODE")
			os.Unsetenv("JWT_SECRET")
		}()

		// Remove .env file to ensure env vars take precedence
		os.Remove(".env")

		config, err := LoadConfig()
		assert.Error(t, err) // No .env file will cause error
		assert.Nil(t, config)
	})
}

func TestGetEnv(t *testing.T) {
	t.Run("get_existing_env_var", func(t *testing.T) {
		os.Setenv("TEST_VAR", "test_value")
		defer os.Unsetenv("TEST_VAR")

		result := getEnv("TEST_VAR", "default")
		assert.Equal(t, "test_value", result)
	})

	t.Run("get_non_existing_env_var", func(t *testing.T) {
		os.Unsetenv("NON_EXISTING_VAR")

		result := getEnv("NON_EXISTING_VAR", "default_value")
		assert.Equal(t, "default_value", result)
	})

	t.Run("get_empty_env_var", func(t *testing.T) {
		os.Setenv("EMPTY_VAR", "")
		defer os.Unsetenv("EMPTY_VAR")

		result := getEnv("EMPTY_VAR", "default_value")
		assert.Equal(t, "default_value", result)
	})
}
