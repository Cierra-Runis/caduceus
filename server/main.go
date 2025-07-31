package main

import (
	"log"
	"server/router"

	"github.com/gofiber/fiber/v3"
)

func main() {
	app := router.Setup()

	// Start the server on port 3000
	log.Fatal(app.Listen(":3000", fiber.ListenConfig{
		EnablePrefork: true,
		// TIPS: When prefork is set to true, only "tcp4" and "tcp6" can be chosen.
		// ListenerNetwork: fiber.NetworkTCP6,
	}))
}
