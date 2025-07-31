package config

import (
	"log"
	"os"

	"github.com/joho/godotenv"
)

type Config struct {
	MongoURI  string
	DBName    string
	Port      string
	AppMode   string
	JWTSecret string
}

func LoadConfig() Config {
	if err := godotenv.Load(); err != nil {
		log.Printf("Warning: No .env file found or error loading it: %v", err)
	}

	return Config{
		MongoURI:  getEnv("MONGO_URI", "mongodb://localhost:27017"),
		DBName:    getEnv("DB_NAME", "caduceus_dev"),
		Port:      getEnv("PORT", "3000"),
		AppMode:   getEnv("APP_MODE", "debug"),
		JWTSecret: getEnv("JWT_SECRET", "default_secret_change_me"),
	}

}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}
