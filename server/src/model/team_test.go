package model_test

import (
	"context"
	"server/src/model"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo/integration/mtest"
)

func TestNewMongoTeamRepo(t *testing.T) {
	mt := mtest.New(t, mtest.NewOptions().ClientType(mtest.Mock))

	mt.Run("creates_mongo_team_repo", func(mt *mtest.T) {
		repo := model.NewMongoTeamRepo(mt.DB)

		assert.NotNil(t, repo)
		assert.NotNil(t, repo.Collection)
		assert.Equal(t, "teams", repo.Collection.Name())
	})
}

func TestMongoTeamRepo_CreateTeam(t *testing.T) {
	mt := mtest.New(t, mtest.NewOptions().ClientType(mtest.Mock))

	user := model.User{
		ID:        primitive.NewObjectID(),
	}

	mt.Run("successful_create_team", func(mt *mtest.T) {
		repo := model.NewMongoTeamRepo(mt.DB)
		ctx := context.Background()


		team := &model.Team{
			Name:      "Test Team",
			CreatorID:   user.ID,
			MemberIDs: []primitive.ObjectID{user.ID},
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		}

		mt.AddMockResponses(mtest.CreateSuccessResponse(bson.E{Key: "insertedId", Value: primitive.NewObjectID()}))

		createdTeam, err := repo.CreateTeam(ctx, team)

		assert.NoError(t, err)
		assert.NotNil(t, createdTeam)
		assert.NotEqual(t, primitive.NilObjectID, createdTeam.ID)
		assert.Equal(t, team.Name, createdTeam.Name)
		assert.Equal(t, team.CreatorID, createdTeam.CreatorID)
		assert.Contains(t, createdTeam.MemberIDs, user.ID)
	})

	mt.Run("failed_create_team", func(mt *mtest.T) {
		repo := model.NewMongoTeamRepo(mt.DB)
		ctx := context.Background()

		team := &model.Team{
			Name:      "Test Team",
			CreatorID:   user.ID,
			MemberIDs: []primitive.ObjectID{user.ID},
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		}

		mt.AddMockResponses(mtest.CreateWriteErrorsResponse(mtest.WriteError{
			Code:    11000,
			Message: "duplicate key error",
		}))

		createdTeam, err := repo.CreateTeam(ctx, team)

		assert.Error(t, err)
		assert.Nil(t, createdTeam)
	})
}