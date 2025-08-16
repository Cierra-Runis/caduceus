package config

import (
	"fmt"
	"server/src/handler"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/cors"
	"github.com/spf13/viper"
)

type RouterConfig struct {
	FiberConfig      fiber.Config
	CorsConfig       cors.Config
	HealthHandler    handler.HealthHandler
	UserHandler      handler.UserHandler
	WebSocketHandler handler.WebSocketHandler
}

type Config struct {
	AllowOrigins []string `mapstructure:"allowOrigins"`
	MongoURI     string   `mapstructure:"mongoUri"`
	DBName       string   `mapstructure:"dbName"`
	Address      string   `mapstructure:"address"`
	JWTSecret    string   `mapstructure:"jwtSecret"`
}

func LoadConfig(env string, configPath string) (*Config, error) {
	v := viper.New()

	v.SetConfigName(env)
	v.SetConfigType("yaml")
	v.AddConfigPath(".")
	v.AddConfigPath(configPath)

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
	if len(c.AllowOrigins) == 0 || c.MongoURI == "" || c.DBName == "" || c.Address == "" || c.JWTSecret == "" {
		return fmt.Errorf("missing required configuration fields")
	}
	return nil
}
