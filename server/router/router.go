package router

import (
	"server/handler"

	"github.com/gofiber/fiber/v3"
)

func Setup(userHandler *handler.UserHandler) *fiber.App {
	app := fiber.New()
	api := app.Group("/api")
	api.Get("/health", handler.GetHealth)
	api.Post("/register", userHandler.CreateUser)
	return app
}
