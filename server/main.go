package main

import (
	"fmt"
	"log"
	"server/config"
	"server/database"
	"server/handler"
	"server/model"
	"server/router"
	"server/service"

	"github.com/gofiber/fiber/v3"
)

func main() {
	appConfig, err := config.LoadConfig()
	if err != nil {
		log.Fatal("Failed to load configuration: ", err)
	}

	client, err := database.NewMongoClient(appConfig.MongoURI, appConfig.DBName)
	if err != nil {
		log.Fatal("Failed to connect to MongoDB: ", err)
	}

	userHandler := handler.NewUserHandler(
		service.NewUserService(
			model.NewMongoUserRepo(client.DB),
			appConfig.JWTSecret,
		),
	)

	app := router.Setup(config.RouterConfig{
		UserHandler: userHandler,
	})

	port := fmt.Sprintf(":%s", appConfig.Port)

	log.Fatal(app.Listen(port, fiber.ListenConfig{
		EnablePrefork: true,
		// TIPS: When prefork is set to true, only "tcp4" and "tcp6" can be chosen.
		// ListenerNetwork: fiber.NetworkTCP6,
	}))
}
