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

type HealthResponseData struct {
	Status    string    `json:"status"`
	Timestamp time.Time `json:"timestamp"`
}

type HealthResponse = model.Response[HealthResponseData]

func (h *HealthHandler) GetHealth(c fiber.Ctx) error {
	return c.Status(fiber.StatusOK).JSON(HealthResponse{
		Message: "Health check successful",
		Data: &HealthResponseData{
			Status:    "ok",
			Timestamp: c.RequestCtx().Time(),
		},
	})
}
