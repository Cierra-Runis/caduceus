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
	api.Get("/health", config.HealthHandler.GetHealth)
	api.Post("/register", config.UserHandler.CreateUser)

	ws := app.Group("/ws")
	ws.Use(func(c fiber.Ctx) error {
		if !websocket.IsWebSocketUpgrade(c) {
			return fiber.ErrUpgradeRequired
		}
		return c.Next()
	})
	ws.Get("/", websocket.New(func(c *websocket.Conn) {
		defer c.Close()

		for {
			var msg handler.WebSocketMessage
			if err := c.ReadJSON(&msg); err != nil {
				break
			}

			response := config.WebSocketHandler.HandleWebSocketMessage(msg)

			if err := c.WriteJSON(response); err != nil {
				break
			}
		}
	}))

	return app
}
