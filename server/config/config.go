package config

import (
	"fmt"
	"server/handler"

	"github.com/gofiber/fiber/v3/middleware/cors"
	"github.com/spf13/viper"
)

type RouterConfig struct {
	CorsConfig  cors.Config
	UserHandler handler.UserHandler
}

type Config struct {
	AllowOrigins []string `mapstructure:"allowOrigins"`
	MongoURI     string   `mapstructure:"mongoUri"`
	DBName       string   `mapstructure:"dbName"`
	Port         string   `mapstructure:"port"`
	JWTSecret    string   `mapstructure:"jwtSecret"`
}

func LoadConfig(env string) (*Config, error) {
	v := viper.New()

	v.SetConfigName(env)
	v.SetConfigType("yaml")
	v.AddConfigPath(".")
	v.AddConfigPath("config")

	if err := v.ReadInConfig(); err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	var config Config
	if err := v.Unmarshal(&config); err != nil {
		return nil, fmt.Errorf("failed to unmarshal config: %w", err)
	}

	if err := config.Validate(); err != nil {
		return nil, fmt.Errorf("config validation failed: %w", err)
	}

	return &config, nil
}

func (c *Config) Validate() error {
	if len(c.AllowOrigins) == 0 || c.MongoURI == "" || c.DBName == "" || c.Port == "" || c.JWTSecret == "" {
		return fmt.Errorf("missing required configuration fields")
	}
	return nil
}
