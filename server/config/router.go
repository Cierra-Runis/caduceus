package config

import (
	"server/handler"

	"github.com/gofiber/fiber/v3/middleware/cors"
)

type RouterConfig struct {
	CorsConfig  cors.Config
	UserHandler handler.UserHandler
}
