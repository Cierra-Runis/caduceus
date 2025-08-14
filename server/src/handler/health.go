package handler

import "github.com/gofiber/fiber/v3"

func GetHealth(c fiber.Ctx) error {
	return c.Status(fiber.StatusOK).JSON(fiber.Map{
		"status":    "ok",
		"timestamp": c.RequestCtx().Time(),
	})
}
