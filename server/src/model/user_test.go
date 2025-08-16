package model

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/integration/mtest"
)

func TestNewMockUserRepo(t *testing.T) {
	repo := NewMockUserRepo()
	assert.NotNil(t, repo)
	assert.NotNil(t, repo.Users)
	assert.Empty(t, repo.Users)
}

func TestMockUserRepo_GetUserByUsername(t *testing.T) {
	repo := NewMockUserRepo()
	ctx := context.Background()

	user1 := &User{
		ID:       primitive.NewObjectID(),
		Username: "user1",
		Password: "password1",
	}
	user2 := &User{
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
	repo := NewMockUserRepo()
	ctx := context.Background()

	t.Run("successful_creation", func(t *testing.T) {
		user := &User{
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
		user := &User{
			Username: "fail",
			Password: "password",
		}

		createdUser, err := repo.CreateUser(ctx, user)
		assert.Error(t, err)
		assert.Nil(t, createdUser)
		assert.Equal(t, "mock create error", err.Error())
	})
}

func TestUser_StructFields(t *testing.T) {
	user := User{
		ID:        primitive.NewObjectID(),
		Username:  "test_user",
		Nickname:  "Test User",
		Password:  "hashed_password",
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}

	assert.NotEqual(t, primitive.NilObjectID, user.ID)
	assert.Equal(t, "test_user", user.Username)
	assert.Equal(t, "Test User", user.Nickname)
	assert.Equal(t, "hashed_password", user.Password)
	assert.WithinDuration(t, time.Now(), user.CreatedAt, time.Second)
	assert.WithinDuration(t, time.Now(), user.UpdatedAt, time.Second)
}

func TestNewMongoUserRepo(t *testing.T) {
	mt := mtest.New(t, mtest.NewOptions().ClientType(mtest.Mock))

	mt.Run("creates_mongo_user_repo", func(mt *mtest.T) {
		repo := NewMongoUserRepo(mt.DB)

		assert.NotNil(t, repo)
		assert.NotNil(t, repo.collection)
		assert.Equal(t, "users", repo.collection.Name())
	})
}

func TestMongoUserRepo_GetUserByUsername(t *testing.T) {
	mt := mtest.New(t, mtest.NewOptions().ClientType(mtest.Mock))

	mt.Run("successful_get_user", func(mt *mtest.T) {
		repo := NewMongoUserRepo(mt.DB)
		ctx := context.Background()

		expectedUser := User{
			ID:        primitive.NewObjectID(),
			Username:  "test_user",
			Nickname:  "Test User",
			Password:  "hashed_password",
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		}

		mt.AddMockResponses(mtest.CreateCursorResponse(1, "caduceus_test.users", mtest.FirstBatch, bson.D{
			{Key: "_id", Value: expectedUser.ID},
			{Key: "username", Value: expectedUser.Username},
			{Key: "nickname", Value: expectedUser.Nickname},
			{Key: "password", Value: expectedUser.Password},
			{Key: "created_at", Value: expectedUser.CreatedAt},
			{Key: "updated_at", Value: expectedUser.UpdatedAt},
		}))

		user, err := repo.GetUserByUsername(ctx, "test_user")

		assert.NoError(t, err)
		assert.NotNil(t, user)
		assert.Equal(t, expectedUser.Username, user.Username)
		assert.Equal(t, expectedUser.Nickname, user.Nickname)
	})

	mt.Run("user_not_found", func(mt *mtest.T) {
		repo := NewMongoUserRepo(mt.DB)
		ctx := context.Background()

		mt.AddMockResponses(mtest.CreateCursorResponse(0, "caduceus_test.users", mtest.FirstBatch))

		user, err := repo.GetUserByUsername(ctx, "nonexistent")

		assert.Error(t, err)
		assert.Nil(t, user)
		assert.Equal(t, mongo.ErrNoDocuments, err)
	})

	mt.Run("database_error", func(mt *mtest.T) {
		repo := NewMongoUserRepo(mt.DB)
		ctx := context.Background()

		mt.AddMockResponses(mtest.CreateCommandErrorResponse(mtest.CommandError{
			Code:    1,
			Message: "database error",
		}))

		user, err := repo.GetUserByUsername(ctx, "test_user")

		assert.Error(t, err)
		assert.Nil(t, user)
	})
}

func TestMongoUserRepo_CreateUser(t *testing.T) {
	mt := mtest.New(t, mtest.NewOptions().ClientType(mtest.Mock))

	mt.Run("successful_create_user", func(mt *mtest.T) {
		repo := NewMongoUserRepo(mt.DB)
		ctx := context.Background()

		newID := primitive.NewObjectID()
		user := &User{
			Username:  "new_user",
			Nickname:  "New User",
			Password:  "hashed_password",
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		}

		mt.AddMockResponses(mtest.CreateSuccessResponse(bson.E{Key: "insertedId", Value: newID}))

		createdUser, err := repo.CreateUser(ctx, user)

		assert.NoError(t, err)
		assert.NotNil(t, createdUser)
		assert.NotEqual(t, primitive.NilObjectID, createdUser.ID)
		assert.Equal(t, "new_user", createdUser.Username)
		assert.Equal(t, "New User", createdUser.Nickname)
	})

	mt.Run("database_error_on_create", func(mt *mtest.T) {
		repo := NewMongoUserRepo(mt.DB)
		ctx := context.Background()

		user := &User{
			Username:  "new_user",
			Password:  "hashed_password",
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		}

		mt.AddMockResponses(mtest.CreateCommandErrorResponse(mtest.CommandError{
			Code:    11000,
			Message: "duplicate key error",
		}))

		createdUser, err := repo.CreateUser(ctx, user)

		assert.Error(t, err)
		assert.Nil(t, createdUser)
	})

	mt.Run("timeout_context", func(mt *mtest.T) {
		repo := NewMongoUserRepo(mt.DB)

		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		user := &User{
			Username:  "new_user",
			Password:  "hashed_password",
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		}

		createdUser, err := repo.CreateUser(ctx, user)

		assert.Error(t, err)
		assert.Nil(t, createdUser)
		assert.Contains(t, err.Error(), "context canceled")
	})
}
