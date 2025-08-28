package service

import (
	"context"
	"errors"
	"server/src/model"
)

const (
	MsgProjectTypeInvalid = "project type invalid"
)

var (
	ErrProjectTypeInvalid = errors.New(MsgProjectTypeInvalid)
)

type ProjectService struct {
	projectRepo model.ProjectRepository
}

func NewProjectService(projectRepo model.ProjectRepository) *ProjectService {
	return &ProjectService{projectRepo: projectRepo}
}

func (s *ProjectService) CreateProject(ctx context.Context, name string, ownerID string, ownerType string) (*model.Project, error) {
	switch ownerType {
	case "USER":
		return s.createProjectByUser(name, ownerID)
	case "TEAM":
		return s.createProjectByTeam(name, ownerID)
	default:
		return nil, ErrProjectTypeInvalid
	}
}

func (s *ProjectService) createProjectByUser(name string, userId string) (*model.Project, error) {
	return nil, nil
}

func (s *ProjectService) createProjectByTeam(name string, teamId string) (*model.Project, error) {
	return nil, nil
}
