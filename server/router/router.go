package router

import (
	"server/handler"
	"server/model"
	"server/service"

	"github.com/gofiber/fiber/v3"
)

func Setup() *fiber.App {
	app := fiber.New(fiber.Config{
		AppName: "Caduceus",
	})

	userHandler := handler.NewUserHandler(
		service.NewUserService(
			model.NewMockUserRepo(),
			"your_jwt_secret",
		),
	)

	app.Post("/register", userHandler.CreateUser)

	return app
}
