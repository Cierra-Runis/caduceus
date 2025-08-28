package service_test

import (
	"context"
	"server/mock"
	"server/src/model"
	"server/src/service"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

func Test_UserService_CreateUser(t *testing.T) {
	mockRepo := mock.NewMockUserRepo()
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

func Test_UserService_LoginUser(t *testing.T) {
	mockRepo := mock.NewMockUserRepo()
	userService := service.NewUserService(mockRepo, "test_secret", 24*time.Hour, false)
	ctx := context.Background()

	username := "test_user"
	password := "test_password"

	_, _ = userService.CreateUser(ctx, username, password)

	t.Run("successful_authentication", func(t *testing.T) {
		cookie, err := userService.LoginUser(ctx, username, password)
		assert.NoError(t, err)
		assert.NotNil(t, cookie)
	})

	t.Run("user_not_found", func(t *testing.T) {
		cookie, err := userService.LoginUser(ctx, "nonexistent", "password")
		if assert.Error(t, err) {
			assert.ErrorIs(t, service.ErrUserNotFound, err)
		}
		assert.Nil(t, cookie)
	})

	t.Run("invalid_password", func(t *testing.T) {
		cookie, err := userService.LoginUser(ctx, username, "wrong_password")

		if assert.Error(t, err) {
			assert.ErrorIs(t, service.ErrInvalidPassword, err)
		}
		assert.Nil(t, cookie)
	})
}

func Test_UserService_NewUserService(t *testing.T) {
	mockRepo := mock.NewMockUserRepo()
	secret := "test_secret"

	userService := service.NewUserService(mockRepo, secret, 24*time.Hour, false)

	assert.NotNil(t, userService)
	assert.Equal(t, mockRepo, userService.Repo)
	assert.Equal(t, secret, userService.JwtSecret)
}
