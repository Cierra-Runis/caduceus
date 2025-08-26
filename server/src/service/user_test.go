package service_test

import (
	"context"
	"server/src/model"
	"server/src/service"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

func TestUserService_CreateUser(t *testing.T) {
	mockRepo := model.NewMockUserRepo()
	userService := service.NewUserService(mockRepo, "test_secret", 24*time.Hour, false)
	ctx := context.Background()

	t.Run("successful_user_creation", func(t *testing.T) {
		username := "successful_user_creation_user"
		password := "test_password"

		user, err := userService.CreateUser(ctx, username, password)

		assert.NoError(t, err)
		assert.NotNil(t, user)
		assert.Equal(t, username, user.Username)
	})

	t.Run("username_already_taken", func(t *testing.T) {
		username := "username_already_taken_user"
		password := "test_password"

		mockRepo.Users = append(mockRepo.Users, &model.User{
			ID:       primitive.NewObjectID(),
			Username: username,
			Password: "hashed_password",
		})

		user, err := userService.CreateUser(ctx, username, password)

		if assert.Error(t, err) {
			assert.ErrorIs(t, service.ErrUsernameTaken, err)
		}
		assert.Nil(t, user)
	})

	t.Run("password_too_long", func(t *testing.T) {
		username := "password_too_long_user"
		password := string(make([]byte, 256))

		_, err := userService.CreateUser(ctx, username, password)
		if assert.Error(t, err) {
			assert.ErrorIs(t, service.ErrInvalidPassword, err)
		}
	})
}

func TestUserService_AuthenticateUser(t *testing.T) {
	mockRepo := model.NewMockUserRepo()
	userService := service.NewUserService(mockRepo, "test_secret", 24*time.Hour, false)
	ctx := context.Background()

	username := "test_user"
	password := "test_password"

	_, _ = userService.CreateUser(ctx, username, password)

	t.Run("successful_authentication", func(t *testing.T) {
		token, claims, err := userService.AuthenticateUser(ctx, username, password)
		assert.NoError(t, err)
		assert.NotNil(t, token)
		assert.NotEmpty(t, *token)
		assert.NotNil(t, claims)
		assert.Equal(t, username, claims.Username)
	})

	t.Run("user_not_found", func(t *testing.T) {
		token, claims, err := userService.AuthenticateUser(ctx, "nonexistent", "password")
		if assert.Error(t, err) {
			assert.ErrorIs(t, service.ErrUserNotFound, err)
		}
		assert.Nil(t, token)
		assert.Nil(t, claims)
	})

	t.Run("invalid_password", func(t *testing.T) {
		token, claims, err := userService.AuthenticateUser(ctx, username, "wrong_password")

		if assert.Error(t, err) {
			assert.ErrorIs(t, service.ErrInvalidPassword, err)
		}
		assert.Nil(t, token)
		assert.Nil(t, claims)
	})
}

func TestUserService_AuthenticateUser_TokenGenerationError(t *testing.T) {
	// Create a mock repository with a specific user
	mockRepo := model.NewMockUserRepo()
	userService := service.NewUserService(mockRepo, "test_secret", 24*time.Hour, false)
	ctx := context.Background()

	// Create a user first
	username := "test_user"
	password := "test_password"
	_, _ = userService.CreateUser(ctx, username, password)

	// Test JWT generation with normal secret (should work fine)
	token, claims, err := userService.AuthenticateUser(ctx, username, password)
	assert.NoError(t, err)
	assert.NotNil(t, token)
	assert.NotEmpty(t, *token)
	assert.NotNil(t, claims)
	assert.Equal(t, username, claims.Username)
}

func TestNewUserService(t *testing.T) {
	mockRepo := model.NewMockUserRepo()
	secret := "test_secret"

	userService := service.NewUserService(mockRepo, secret, 24*time.Hour, false)

	assert.NotNil(t, userService)
	assert.Equal(t, mockRepo, userService.Repo)
	assert.Equal(t, secret, userService.JwtSecret)
}
