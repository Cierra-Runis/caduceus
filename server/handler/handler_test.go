package handler

import (
	"bytes"
	"encoding/json"
	"net/http/httptest"
	"server/model"
	"server/service"
	"testing"

	"github.com/gofiber/fiber/v3"
	"github.com/stretchr/testify/assert"
)

func TestGetHealth(t *testing.T) {
	app := fiber.New()
	app.Get("/health", GetHealth)

	req := httptest.NewRequest("GET", "/health", nil)
	resp, err := app.Test(req)

	assert.NoError(t, err)
	assert.Equal(t, fiber.StatusOK, resp.StatusCode)

	var response map[string]interface{}
	err = json.NewDecoder(resp.Body).Decode(&response)
	assert.NoError(t, err)
	assert.Equal(t, "ok", response["status"])
	assert.NotNil(t, response["timestamp"])
}

func TestUserHandler_CreateUser(t *testing.T) {
	mockRepo := model.NewMockUserRepo()
	userService := service.NewUserService(mockRepo, "test_secret")
	userHandler := NewUserHandler(userService)

	app := fiber.New()
	app.Post("/register", userHandler.CreateUser)

	t.Run("successful_user_creation", func(t *testing.T) {
		reqBody := CreateUserRequest{
			Username: "testuser",
			Password: "testpassword",
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest("POST", "/register", bytes.NewReader(jsonBody))
		req.Header.Set("Content-Type", "application/json")

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusOK, resp.StatusCode)

		var response model.User
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, "testuser", response.Username)
	})

	t.Run("username_already_taken", func(t *testing.T) {
		reqBody := CreateUserRequest{
			Username: "testuser", // Same username as above
			Password: "testpassword",
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest("POST", "/register", bytes.NewReader(jsonBody))
		req.Header.Set("Content-Type", "application/json")

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusConflict, resp.StatusCode)

		var response map[string]string
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, service.ErrUsernameTaken, response["error"])
	})

	t.Run("invalid_password", func(t *testing.T) {
		reqBody := CreateUserRequest{
			Username: "testuser2",
			Password: string(make([]byte, 256)), // Too long password
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest("POST", "/register", bytes.NewReader(jsonBody))
		req.Header.Set("Content-Type", "application/json")

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusBadRequest, resp.StatusCode)

		var response map[string]string
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, service.ErrInvalidPassword, response["error"])
	})

	t.Run("invalid_request_body", func(t *testing.T) {
		req := httptest.NewRequest("POST", "/register", bytes.NewReader([]byte("invalid json")))
		req.Header.Set("Content-Type", "application/json")

		resp, err := app.Test(req)
		assert.NoError(t, err)
		// Fiber v3 returns 500 for JSON parsing errors, not 400
		assert.Equal(t, fiber.StatusInternalServerError, resp.StatusCode)
	})

	t.Run("mock_create_error", func(t *testing.T) {
		reqBody := CreateUserRequest{
			Username: "fail", // This triggers mock error
			Password: "testpassword",
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest("POST", "/register", bytes.NewReader(jsonBody))
		req.Header.Set("Content-Type", "application/json")

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusInternalServerError, resp.StatusCode)
	})
}

func TestNewUserHandler(t *testing.T) {
	mockRepo := model.NewMockUserRepo()
	userService := service.NewUserService(mockRepo, "test_secret")
	userHandler := NewUserHandler(userService)

	assert.NotNil(t, userHandler)
	assert.Equal(t, userService, userHandler.userService)
}
