package handler_test

import (
	"encoding/json"
	"net/http/httptest"
	"testing"

	"server/src/handler"
	"server/src/router"

	"github.com/gofiber/fiber/v3"
	"github.com/stretchr/testify/assert"
)

func TestGetHealth(t *testing.T) {
	app := router.SetupTestRouter()
	healthHandler := handler.NewHealthHandler()
	app.Get("/health", healthHandler.GetHealth)

	req := httptest.NewRequest(fiber.MethodGet, "/health", nil)
	resp, err := app.Test(req)

	assert.NoError(t, err)
	assert.Equal(t, fiber.StatusOK, resp.StatusCode)

	var response map[string]interface{}
	err = json.NewDecoder(resp.Body).Decode(&response)
	assert.NoError(t, err)
	assert.Equal(t, "ok", response["status"])
	assert.NotNil(t, response["timestamp"])
}
