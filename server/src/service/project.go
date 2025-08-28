package service

import (
	"context"
	"errors"
	"server/src/model"
)

const (
	MsgProjectTypeInvalid = "project type invalid"
	MsgTeamNotFound       = "team not found"
)

var (
	ErrProjectTypeInvalid = errors.New(MsgProjectTypeInvalid)
	ErrTeamNotFound       = errors.New(MsgTeamNotFound)
)

type ProjectService struct {
	projectRepo model.ProjectRepository
	userRepo    model.UserRepository
	teamRepo    model.TeamRepository
}

func NewProjectService(projectRepo model.ProjectRepository, userRepo model.UserRepository, teamRepo model.TeamRepository) *ProjectService {
	return &ProjectService{
		projectRepo: projectRepo,
		userRepo:    userRepo,
		teamRepo:    teamRepo,
	}
}

func (s *ProjectService) CreateProject(ctx context.Context, name string, ownerID string, ownerType string) (*model.Project, error) {
	switch ownerType {
	case "USER":
		return s.createProjectByUser(ctx, name, ownerID)
	case "TEAM":
		return s.createProjectByTeam(ctx, name, ownerID)
	default:
		return nil, ErrProjectTypeInvalid
	}
}

func (s *ProjectService) createProjectByUser(ctx context.Context, name string, userId string) (*model.Project, error) {
	_, err := s.userRepo.GetUserByID(ctx, userId)
	if err != nil {
		return nil, err
	}

	return nil, nil
}

func (s *ProjectService) createProjectByTeam(ctx context.Context, name string, teamId string) (*model.Project, error) {
	_, err := s.teamRepo.GetTeamByID(ctx, teamId)
	if err != nil {
		return nil, err
	}

	return nil, nil
}
