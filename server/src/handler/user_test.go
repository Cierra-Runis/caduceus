package handler_test

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http/httptest"
	"server/src/handler"
	"server/src/model"
	"server/src/router"
	"server/src/service"
	"testing"

	"github.com/gofiber/fiber/v3"
	"github.com/stretchr/testify/assert"
)

func TestUserHandler_CreateUser(t *testing.T) {
	mockRepo := model.NewMockUserRepo()
	userService := service.NewUserService(mockRepo, "test_secret")
	userHandler := handler.NewUserHandler(userService)

	app := router.SetupTestRouter()
	app.Post("/register", userHandler.CreateUser)

	t.Run("successful_user_creation", func(t *testing.T) {
		reqBody := handler.CreateUserRequest{
			Username: "test_user",
			Password: "test_password",
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest(fiber.MethodPost, "/register", bytes.NewReader(jsonBody))
		req.Header.Set(fiber.HeaderContentType, fiber.MIMEApplicationJSON)

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusOK, resp.StatusCode)

		var response handler.CreateUserResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, "test_user", response.Data.Username)
	})

	t.Run("username_already_taken", func(t *testing.T) {
		reqBody := handler.CreateUserRequest{
			Username: "test_user", // Same username as above
			Password: "test_password",
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest(fiber.MethodPost, "/register", bytes.NewReader(jsonBody))
		req.Header.Set(fiber.HeaderContentType, fiber.MIMEApplicationJSON)

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusConflict, resp.StatusCode)

		var response handler.CreateUserResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, service.ErrUsernameTaken, response.Message)
	})

	t.Run("invalid_password", func(t *testing.T) {
		reqBody := handler.CreateUserRequest{
			Username: "test_user2",
			Password: string(make([]byte, 256)), // Too long password
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest(fiber.MethodPost, "/register", bytes.NewReader(jsonBody))
		req.Header.Set(fiber.HeaderContentType, fiber.MIMEApplicationJSON)

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusBadRequest, resp.StatusCode)

		var response handler.CreateUserResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, service.ErrInvalidPassword, response.Message)
	})

	t.Run("invalid_request_body", func(t *testing.T) {
		reqBody := map[string]interface{}{
			"invalid_field": "value",
		}
		jsonBody, _ := json.Marshal(reqBody)
		req := httptest.NewRequest(fiber.MethodPost, "/register", bytes.NewReader(jsonBody))
		req.Header.Set(fiber.HeaderContentType, fiber.MIMEApplicationJSON)

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusBadRequest, resp.StatusCode)

		var response handler.CreateUserResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, service.ErrInvalidRequestBody, response.Message)
	})

	t.Run("mock_create_error", func(t *testing.T) {
		reqBody := handler.CreateUserRequest{
			Username: "fail", // This triggers mock error
			Password: "test_password",
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest(fiber.MethodPost, "/register", bytes.NewReader(jsonBody))
		req.Header.Set(fiber.HeaderContentType, fiber.MIMEApplicationJSON)

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusInternalServerError, resp.StatusCode)
	})
}

func TestUserHandler_Login(t *testing.T) {
	mockRepo := model.NewMockUserRepo()
	userService := service.NewUserService(mockRepo, "test_secret")
	userService.CreateUser(context.Background(), "test_user", "test_password")
	userHandler := handler.NewUserHandler(userService)

	app := router.SetupTestRouter()
	app.Post("/login", userHandler.Login)

	t.Run("successful_login", func(t *testing.T) {
		reqBody := handler.LoginRequest{
			Username: "test_user",
			Password: "test_password",
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest(fiber.MethodPost, "/login", bytes.NewReader(jsonBody))
		req.Header.Set(fiber.HeaderContentType, fiber.MIMEApplicationJSON)

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusOK, resp.StatusCode)

		var response handler.LoginResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.NotEmpty(t, response.Data.Token)
	})

	t.Run("invalid_credentials", func(t *testing.T) {
		reqBody := handler.LoginRequest{
			Username: "test_user",
			Password: "wrong_password",
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest(fiber.MethodPost, "/login", bytes.NewReader(jsonBody))
		req.Header.Set(fiber.HeaderContentType, fiber.MIMEApplicationJSON)

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusUnauthorized, resp.StatusCode)

		var response handler.LoginResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, service.ErrInvalidPassword, response.Message)
	})

	t.Run("user_not_found", func(t *testing.T) {
		reqBody := handler.LoginRequest{
			Username: "non_existent_user",
			Password: "test_password",
		}
		jsonBody, _ := json.Marshal(reqBody)

		req := httptest.NewRequest(fiber.MethodPost, "/login", bytes.NewReader(jsonBody))
		req.Header.Set(fiber.HeaderContentType, fiber.MIMEApplicationJSON)

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusNotFound, resp.StatusCode)

		var response handler.LoginResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, service.ErrUserNotFound, response.Message)
	})

	t.Run("invalid_request_body", func(t *testing.T) {
		reqBody := map[string]interface{}{
			"invalid_field": "value",
		}
		jsonBody, _ := json.Marshal(reqBody)
		req := httptest.NewRequest(fiber.MethodPost, "/login", bytes.NewReader(jsonBody))
		req.Header.Set(fiber.HeaderContentType, fiber.MIMEApplicationJSON)

		resp, err := app.Test(req)
		assert.NoError(t, err)
		assert.Equal(t, fiber.StatusBadRequest, resp.StatusCode)

		var response handler.LoginResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, service.ErrInvalidRequestBody, response.Message)
	})

}
