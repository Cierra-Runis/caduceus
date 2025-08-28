package mock

import (
	"context"
	"errors"
	"server/src/model"

	"go.mongodb.org/mongo-driver/bson/primitive"
)

type MockProjectRepo struct {
	Projects []*model.Project
}

func NewMockProjectRepo() *MockProjectRepo {
	return &MockProjectRepo{
		Projects: make([]*model.Project, 0),
	}
}

func (m *MockProjectRepo) CreateProject(ctx context.Context, project *model.Project) (*model.Project, error) {
	if project.Name == "fail" {
		return nil, errors.New("mock create error")
	}
	project.ID = primitive.NewObjectID()
	m.Projects = append(m.Projects, project)
	return project, nil
}