package router

import (
	"server/config"
	"server/database"
	"server/handler"
	"server/model"
	"server/service"

	"github.com/gofiber/fiber/v3"
)

func Setup(config config.Config) *fiber.App {

	app := fiber.New()

	client, err := database.NewMongoClient(config.MongoURI, config.DBName)
	if err != nil {
		panic("Failed to connect to MongoDB: " + err.Error())
	}

	userHandler := handler.NewUserHandler(
		service.NewUserService(
			model.NewMongoUserRepo(client.DB),
			config.JWTSecret,
		),
	)

	app.Post("/register", userHandler.CreateUser)

	return app
}
