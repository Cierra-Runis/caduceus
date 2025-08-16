package config_test

import (
	"server/src/config"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestLoadConfig(t *testing.T) {
	t.Run("load_config_success", func(t *testing.T) {
		env := "test"
		config, err := config.LoadConfig(env, "../../config")
		assert.NoError(t, err)
		assert.NotNil(t, config)
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
