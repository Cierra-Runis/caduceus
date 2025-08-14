package router

import (
	"net/http/httptest"
	"server/src/config"
	"server/src/handler"
	"server/src/model"
	"server/src/service"
	"testing"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/cors"
	"github.com/stretchr/testify/assert"
)

func TestSetup(t *testing.T) {
	// Create dependencies
	mockRepo := model.NewMockUserRepo()
	userService := service.NewUserService(mockRepo, "test_secret")
	userHandler := handler.NewUserHandler(userService)

	routerConfig := config.RouterConfig{
		CorsConfig: cors.Config{
			AllowOrigins: []string{"http://localhost:3000"},
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
	}

	app := Setup(routerConfig)

	assert.NotNil(t, app)

	t.Run("health_endpoint", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/api/health", nil)
		resp, err := app.Test(req)

		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusOK, resp.StatusCode)
	})

	t.Run("register_endpoint_exists", func(t *testing.T) {
		req := httptest.NewRequest("POST", "/api/register", nil)
		resp, err := app.Test(req)

		assert.NoError(t, err)
		// Should get 422 for invalid JSON binding, not 404 for missing route
		assert.Equal(t, fiber.StatusUnprocessableEntity, resp.StatusCode)
	})

	t.Run("websocket_upgrade_required", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/ws/", nil)
		resp, err := app.Test(req)

		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusUpgradeRequired, resp.StatusCode)
	})

	t.Run("cors_preflight", func(t *testing.T) {
		req := httptest.NewRequest("OPTIONS", "/api/health", nil)
		req.Header.Set("Origin", "http://localhost:3000")
		req.Header.Set("Access-Control-Request-Method", "GET")

		resp, err := app.Test(req)

		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusNoContent, resp.StatusCode)
		assert.Equal(t, "http://localhost:3000", resp.Header.Get("Access-Control-Allow-Origin"))
	})

	t.Run("invalid_route", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/invalid", nil)
		resp, err := app.Test(req)

		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusNotFound, resp.StatusCode)
	})
}
