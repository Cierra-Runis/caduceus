package router

import (
	"server/src/config"
	"server/src/handler"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/cors"
	"github.com/kiuber/gofiber3-contrib/websocket"
)

func Setup(config config.RouterConfig) *fiber.App {
	app := fiber.New()

	app.Use(cors.New(config.CorsConfig))

	api := app.Group("/api")
	api.Get("/health", handler.GetHealth)
	api.Post("/register", config.UserHandler.CreateUser)

	ws := app.Group("/ws")
	ws.Use("/ws", func(c fiber.Ctx) error {
		if !websocket.IsWebSocketUpgrade(c) {
			return fiber.ErrUpgradeRequired
		}
		return c.Next()
	})
	ws.Get("/", websocket.New(handler.WebSocket))

	return app
}
