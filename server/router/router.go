package router

import (
	"server/handler"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/cors"
	"github.com/kiuber/gofiber3-contrib/websocket"
)

func Setup(userHandler *handler.UserHandler) *fiber.App {
	app := fiber.New()

	app.Use(cors.New(cors.Config{
		AllowOrigins:     []string{"http://localhost:3000"},
		AllowHeaders:     []string{"Origin", "Content-Type", "Accept", "Authorization"},
		AllowMethods:     []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowCredentials: true,
	}))

	api := app.Group("/api")
	api.Get("/health", handler.GetHealth)
	api.Post("/register", userHandler.CreateUser)

	ws := app.Group("/ws")
	ws.Use("/ws", func(c fiber.Ctx) error {
		if websocket.IsWebSocketUpgrade(c) {
			return c.Next()
		}
		return fiber.ErrUpgradeRequired
	})
	ws.Get("/", websocket.New(handler.WebSocket))

	return app
}
