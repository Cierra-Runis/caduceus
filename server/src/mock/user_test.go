package mock_test

import (
	"context"
	"server/src/mock"
	"server/src/model"
	"testing"

	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

func TestNewMockUserRepo(t *testing.T) {
	repo := mock.NewMockUserRepo()
	assert.NotNil(t, repo)
	assert.NotNil(t, repo.Users)
	assert.Empty(t, repo.Users)
}

func TestMockUserRepo_GetUserByUsername(t *testing.T) {
	repo := mock.NewMockUserRepo()
	ctx := context.Background()

	user1 := &model.User{
		ID:       primitive.NewObjectID(),
		Username: "user1",
		Password: "password1",
	}
	user2 := &model.User{
		ID:       primitive.NewObjectID(),
		Username: "user2",
		Password: "password2",
	}

	repo.Users = append(repo.Users, user1, user2)

	t.Run("existing_user", func(t *testing.T) {
		foundUser, err := repo.GetUserByUsername(ctx, "user1")
		assert.NoError(t, err)
		assert.NotNil(t, foundUser)
		assert.Equal(t, "user1", foundUser.Username)
		assert.Equal(t, user1.ID, foundUser.ID)
	})

	t.Run("non_existing_user", func(t *testing.T) {
		foundUser, err := repo.GetUserByUsername(ctx, "nonexistent")
		assert.Error(t, err)
		assert.Nil(t, foundUser)
		assert.Equal(t, "user not found", err.Error())
	})
}

func TestMockUserRepo_CreateUser(t *testing.T) {
	repo := mock.NewMockUserRepo()
	ctx := context.Background()

	t.Run("successful_creation", func(t *testing.T) {
		user := &model.User{
			Username: "new_user",
			Password: "password",
		}

		createdUser, err := repo.CreateUser(ctx, user)
		assert.NoError(t, err)
		assert.NotNil(t, createdUser)
		assert.Equal(t, "new_user", createdUser.Username)
		assert.NotEqual(t, primitive.NilObjectID, createdUser.ID)
		assert.Len(t, repo.Users, 1)
	})

	t.Run("mock_error_scenario", func(t *testing.T) {
		user := &model.User{
			Username: "fail",
			Password: "password",
		}

		createdUser, err := repo.CreateUser(ctx, user)
		assert.Error(t, err)
		assert.Nil(t, createdUser)
		assert.Equal(t, "mock create error", err.Error())
	})
}

