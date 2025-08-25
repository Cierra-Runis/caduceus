package router_test

import (
	"net/http/httptest"
	"server/src/config"
	"server/src/handler"
	"server/src/model"
	"server/src/router"
	"server/src/service"
	"testing"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/cors"
	"github.com/stretchr/testify/assert"
)

func TestSetup(t *testing.T) {
	mockRepo := model.NewMockUserRepo()
	userService := service.NewUserService(mockRepo, "test_secret", 24*time.Hour, false)
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

	app := router.Setup(routerConfig)

	assert.NotNil(t, app)

	t.Run("health_endpoint", func(t *testing.T) {
		req := httptest.NewRequest(fiber.MethodGet, "/api/health", nil)
		resp, err := app.Test(req)

		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusOK, resp.StatusCode)
	})

	t.Run("register_endpoint_exists", func(t *testing.T) {
		req := httptest.NewRequest(fiber.MethodPost, "/api/register", nil)
		resp, err := app.Test(req)

		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusBadRequest, resp.StatusCode)
	})

	t.Run("websocket_upgrade_required", func(t *testing.T) {
		req := httptest.NewRequest(fiber.MethodGet, "/ws/", nil)
		resp, err := app.Test(req)

		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusUpgradeRequired, resp.StatusCode)
	})

	t.Run("websocket_with_upgrade_headers", func(t *testing.T) {
		req := httptest.NewRequest(fiber.MethodGet, "/ws/", nil)
		req.Header.Set(fiber.HeaderConnection, "Upgrade")
		req.Header.Set(fiber.HeaderUpgrade, "websocket")
		req.Header.Set(fiber.HeaderSecWebSocketKey, "dGhlIHNhbXBsZSBub25jZQ==")
		req.Header.Set(fiber.HeaderSecWebSocketVersion, "13")

		resp, err := app.Test(req)

		assert.NoError(t, err)
		// This should pass the middleware check (c.Next() gets called)
		// but will fail at the WebSocket handler level since we're not doing a real WebSocket handshake
		// The important thing is that we test the c.Next() path in the middleware
		assert.NotEqual(t, fiber.StatusUpgradeRequired, resp.StatusCode)
	})

	t.Run("cors_preflight", func(t *testing.T) {
		req := httptest.NewRequest(fiber.MethodOptions, "/api/health", nil)
		req.Header.Set(fiber.HeaderOrigin, "http://localhost:3000")
		req.Header.Set(fiber.HeaderAccessControlRequestMethod, fiber.MethodGet)

		resp, err := app.Test(req)

		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusNoContent, resp.StatusCode)
		assert.Equal(t, "http://localhost:3000", resp.Header.Get("Access-Control-Allow-Origin"))
	})

	t.Run("invalid_route", func(t *testing.T) {
		req := httptest.NewRequest(fiber.MethodGet, "/invalid", nil)
		resp, err := app.Test(req)

		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusNotFound, resp.StatusCode)
	})
}

func TestSetupTestRouter(t *testing.T) {
	app := router.SetupTestRouter()
	assert.NotNil(t, app)
}
