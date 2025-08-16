package handler

import "github.com/gofiber/fiber/v3"

type HealthHandler struct{}

func NewHealthHandler() *HealthHandler {
	return &HealthHandler{}
}

func (h *HealthHandler) GetHealth(c fiber.Ctx) error {
	return c.Status(fiber.StatusOK).JSON(fiber.Map{
		"status":    "ok",
		"timestamp": c.RequestCtx().Time(),
	})
}
