package handler

import (
	"server/src/model"
	"time"

	"github.com/gofiber/fiber/v3"
)

type HealthHandler struct{}

func NewHealthHandler() *HealthHandler {
	return &HealthHandler{}
}

type HealthPayload struct {
	Status    string    `json:"status"`
	Timestamp time.Time `json:"timestamp"`
}

type HealthResponse = model.Response[HealthPayload]

func (h *HealthHandler) GetHealth(c fiber.Ctx) error {
	return c.Status(fiber.StatusOK).JSON(HealthResponse{
		Message: "Health check successful",
		Payload: &HealthPayload{
			Status:    "ok",
			Timestamp: c.RequestCtx().Time(),
		},
	})
}
