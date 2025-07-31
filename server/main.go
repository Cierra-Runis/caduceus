package main

import (
	"fmt"
	"log"
	"server/config"
	"server/router"

	"github.com/gofiber/fiber/v3"
)

func main() {
	config := config.LoadConfig()
	app := router.Setup(config)

	port := fmt.Sprintf(":%s", config.Port)

	log.Fatal(app.Listen(port, fiber.ListenConfig{
		EnablePrefork: true,
		// TIPS: When prefork is set to true, only "tcp4" and "tcp6" can be chosen.
		// ListenerNetwork: fiber.NetworkTCP6,
	}))
}
