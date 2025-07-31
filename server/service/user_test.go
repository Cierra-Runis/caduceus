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

		if err == nil {
			t.Fatal("expected error, got nil")
		}

		if err.Error() != ErrUsernameTaken {
			t.Errorf("expected error %s, got %s", ErrUsernameTaken, err.Error())
		}

		if user != nil {
			t.Error("expected user to be nil")
		}
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

		if err != nil {
			t.Fatalf("expected no error, got: %v", err)
		}

		if token == nil {
			t.Fatal("expected token not to be nil")
		}

		if *token == "" {
			t.Error("expected token not to be empty string")
		}
	})

	t.Run("user_not_found", func(t *testing.T) {
		token, err := userService.AuthenticateUser(ctx, "nonexistent", "password")

		if err == nil {
			t.Fatal("expected error, got nil")
		}

		if err.Error() != ErrUserNotFound {
			t.Errorf("expected error %s, got %s", ErrUserNotFound, err.Error())
		}

		if token != nil {
			t.Error("expected token to be nil")
		}
	})

	t.Run("invalid_password", func(t *testing.T) {
		token, err := userService.AuthenticateUser(ctx, username, "wrongpassword")

		if err == nil {
			t.Fatal("expected error, got nil")
		}

		if err.Error() != ErrInvalidPassword {
			t.Errorf("expected error %s, got %s", ErrInvalidPassword, err.Error())
		}

		if token != nil {
			t.Error("expected token to be nil")
		}
	})
}
