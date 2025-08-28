package model_test

import (
	"context"
	"server/src/model"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/integration/mtest"
)


func TestNewMongoUserRepo(t *testing.T) {
	mt := mtest.New(t, mtest.NewOptions().ClientType(mtest.Mock))

	mt.Run("creates_mongo_user_repo", func(mt *mtest.T) {
		repo := model.NewMongoUserRepo(mt.DB)

		assert.NotNil(t, repo)
		assert.NotNil(t, repo.Collection)
		assert.Equal(t, "users", repo.Collection.Name())
	})
}

func TestMongoUserRepo_GetUserByUsername(t *testing.T) {
	mt := mtest.New(t, mtest.NewOptions().ClientType(mtest.Mock))

	mt.Run("successful_get_user", func(mt *mtest.T) {
		repo := model.NewMongoUserRepo(mt.DB)
		ctx := context.Background()

		expectedUser := model.User{
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
		repo := model.NewMongoUserRepo(mt.DB)
		ctx := context.Background()

		mt.AddMockResponses(mtest.CreateCursorResponse(0, "caduceus_test.users", mtest.FirstBatch))

		user, err := repo.GetUserByUsername(ctx, "nonexistent")

		assert.Error(t, err)
		assert.Nil(t, user)
		assert.Equal(t, mongo.ErrNoDocuments, err)
	})

	mt.Run("database_error", func(mt *mtest.T) {
		repo := model.NewMongoUserRepo(mt.DB)
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
		repo := model.NewMongoUserRepo(mt.DB)
		ctx := context.Background()

		user := &model.User{
			Username:  "new_user",
			Nickname:  "New User",
			Password:  "hashed_password",
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		}

		mt.AddMockResponses(mtest.CreateSuccessResponse(bson.E{Key: "insertedId", Value: primitive.NewObjectID()}))

		createdUser, err := repo.CreateUser(ctx, user)

		assert.NoError(t, err)
		assert.NotNil(t, createdUser)
		assert.NotEqual(t, primitive.NilObjectID, createdUser.ID)
		assert.Equal(t, "new_user", createdUser.Username)
		assert.Equal(t, "New User", createdUser.Nickname)
	})

	mt.Run("database_error_on_create", func(mt *mtest.T) {
		repo := model.NewMongoUserRepo(mt.DB)
		ctx := context.Background()

		user := &model.User{
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
}
