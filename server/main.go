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
	config := config.LoadConfig()

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

	app := router.Setup(userHandler)

	port := fmt.Sprintf(":%s", config.Port)

	log.Fatal(app.Listen(port, fiber.ListenConfig{
		EnablePrefork: true,
		// TIPS: When prefork is set to true, only "tcp4" and "tcp6" can be chosen.
		// ListenerNetwork: fiber.NetworkTCP6,
	}))
}
