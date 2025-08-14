package main

import (
	"context"
	"log"
	"os"
	"server/config"
	"server/database"
	"server/handler"
	"server/model"
	"server/router"
	"server/service"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/cors"
)

func main() {
	env := os.Getenv("APP_ENV")
	if env == "" {
		env = "dev"
	}
	appConfig, err := config.LoadConfig(env)
	if err != nil {
		log.Fatal("Failed to load configuration: ", err)
	}

	client, err := database.NewMongoClient(
		appConfig.MongoURI,
		appConfig.DBName,
		10*time.Second,
	)
	if err != nil {
		log.Fatal("Failed to connect to MongoDB: ", err)
	}
	defer client.Client.Disconnect(context.Background())

	userHandler := handler.NewUserHandler(
		service.NewUserService(
			model.NewMongoUserRepo(client.DB),
			appConfig.JWTSecret,
		),
	)

	app := router.Setup(config.RouterConfig{
		CorsConfig: cors.Config{
			AllowOrigins: appConfig.AllowOrigins,
			AllowHeaders: []string{
				fiber.HeaderOrigin,
				fiber.HeaderContentType,
				fiber.HeaderAccept,
				fiber.HeaderAuthorization,
			},
			AllowMethods: []string{
				fiber.MethodGet,
				fiber.MethodPost,
				fiber.MethodPut,
				fiber.MethodDelete,
				fiber.MethodOptions,
			},
			AllowCredentials: true,
		},
		UserHandler: *userHandler,
	})

	log.Fatal(app.Listen(appConfig.Address, fiber.ListenConfig{
		EnablePrefork: true,
		// TIPS: When prefork is set to true, only "tcp4" and "tcp6" can be chosen.
		// ListenerNetwork: fiber.NetworkTCP6,
	}))
}
