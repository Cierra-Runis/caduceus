package model_test

import (
	"context"
	"server/src/model"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/mongo/integration/mtest"
)

func TestNewMongoProjectRepo(t *testing.T) {
	mt := mtest.New(t, mtest.NewOptions().ClientType(mtest.Mock))

	mt.Run("creates_mongo_project_repo", func(mt *mtest.T) {
		repo := model.NewMongoProjectRepo(mt.DB)

		assert.NotNil(t, repo)
		assert.NotNil(t, repo.Collection)
		assert.Equal(t, "projects", repo.Collection.Name())
	})
}

func TestMongoProjectRepo_CreateProject(t *testing.T) {
	mt := mtest.New(t, mtest.NewOptions().ClientType(mtest.Mock))

	mt.Run("creates_project", func(mt *mtest.T) {
		repo := model.NewMongoProjectRepo(mt.DB)
		ctx := context.Background()

		project := &model.Project{
			Name:      "Test Project",
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		}

		mt.AddMockResponses(mtest.CreateSuccessResponse())

		createdProject, err := repo.CreateProject(ctx, project)

		assert.NoError(t, err)
		assert.NotNil(t, createdProject)
		assert.NotEqual(t, "", createdProject.ID)
		assert.Equal(t, project.Name, createdProject.Name)
	})

	mt.Run("failed_create_project", func(mt *mtest.T) {
		repo := model.NewMongoProjectRepo(mt.DB)
		ctx := context.Background()

		project := &model.Project{
			Name:      "Test Project",
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		}

		mt.AddMockResponses(mtest.CreateWriteErrorsResponse(mtest.WriteError{
			Code:    11000,
			Message: "duplicate key error",
		}))

		createdProject, err := repo.CreateProject(ctx, project)

		if assert.Error(t, err) {
			assert.Contains(t, err.Error(), "duplicate key error")
		}
		assert.Nil(t, createdProject)
	})
}
