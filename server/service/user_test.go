package service

import (
	"context"
	"server/model"
	"testing"

	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

func TestUserService_CreateUser(t *testing.T) {
	mockRepo := model.NewMockUserRepo()
	userService := NewUserService(mockRepo, "test_secret")
	ctx := context.Background()

	t.Run("successful_user_creation", func(t *testing.T) {
		username := "testuser"
		password := "testpassword"

		user, err := userService.CreateUser(ctx, username, password)

		assert.NoError(t, err)
		assert.NotNil(t, user)
		assert.Equal(t, username, user.Username)
	})

	t.Run("username_already_taken", func(t *testing.T) {
		username := "existinguser"
		password := "testpassword"

		// First create a user
		mockRepo.Users = append(mockRepo.Users, &model.User{
			ID:       primitive.NewObjectID(),
			Username: username,
			Password: "hashedpassword",
		})

		user, err := userService.CreateUser(ctx, username, password)

		if assert.Error(t, err) {
			assert.Equal(t, ErrUsernameTaken, err.Error())
		}
		assert.Nil(t, user)
	})

	t.Run("password_too_long", func(t *testing.T) {
		username := "testuser"
		password := "a" + string(make([]byte, 256)) // Simulate a long password

		_, err := userService.CreateUser(ctx, username, password)
		assert.Error(t, err)
	})
}

func TestUserService_AuthenticateUser(t *testing.T) {
	// Setup
	mockRepo := model.NewMockUserRepo()
	userService := NewUserService(mockRepo, "test_secret")
	ctx := context.Background()

	// First create a user for testing
	username := "testuser"
	password := "testpassword"

	// Manually create user (simulate existing user)
	_, _ = userService.CreateUser(ctx, username, password)

	t.Run("successful_authentication", func(t *testing.T) {
		token, err := userService.AuthenticateUser(ctx, username, password)
		assert.NoError(t, err)
		assert.NotNil(t, token)
		assert.NotEmpty(t, *token)
	})

	t.Run("user_not_found", func(t *testing.T) {
		token, err := userService.AuthenticateUser(ctx, "nonexistent", "password")
		if assert.Error(t, err) {
			assert.Equal(t, ErrUserNotFound, err.Error())
		}
		assert.Nil(t, token)
	})

	t.Run("invalid_password", func(t *testing.T) {
		token, err := userService.AuthenticateUser(ctx, username, "wrongpassword")

		if assert.Error(t, err) {
			assert.Equal(t, ErrInvalidPassword, err.Error())
		}
		assert.Nil(t, token)
	})
}
