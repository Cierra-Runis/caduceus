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
	appConfig := config.LoadConfig()

	client, err := database.NewMongoClient(appConfig.MongoURI, appConfig.DBName)
	if err != nil {
		panic("Failed to connect to MongoDB: " + err.Error())
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
