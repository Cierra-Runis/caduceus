package router

import (
	"log"
	"server/src/config"
	"server/src/handler"
	"server/src/middleware"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/cors"
	"github.com/kiuber/gofiber3-contrib/websocket"
)

func Setup(config config.RouterConfig) *fiber.App {
	app := fiber.New(config.FiberConfig)

	app.Use(cors.New(config.CorsConfig))

	api := app.Group("/api")
	api.Get("/health", config.HealthHandler.GetHealth)
	api.Post("/register", config.UserHandler.CreateUser)
	api.Post("/login", config.UserHandler.LoginUser)

	api.Post("/project", config.JWTMiddleware, config.ProjectHandler.CreateProject)

	ws := app.Group("/ws")
	ws.Use(func(c fiber.Ctx) error {
		if !websocket.IsWebSocketUpgrade(c) {
			return fiber.ErrUpgradeRequired
		}
		return c.Next()
	})
	ws.Get("/", websocket.New(func(c *websocket.Conn) {
		defer func() {
			if err := c.Close(); err != nil {
				log.Printf("Error closing WebSocket connection: %v", err)
			}
		}()

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

func SetupTestRouter() *fiber.App {
	app := fiber.New(fiber.Config{
		StructValidator: middleware.NewStructValidator(),
	})

	return app
}
