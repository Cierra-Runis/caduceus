package mock

import (
	"context"
	"errors"
	"server/src/model"

	"go.mongodb.org/mongo-driver/bson/primitive"
)

type MockUserRepo struct {
	Users []*model.User
}

func NewMockUserRepo() *MockUserRepo {
	return &MockUserRepo{
		Users: make([]*model.User, 0),
	}
}


func (m *MockUserRepo) GetUserByUsername(ctx context.Context, username string) (*model.User, error) {
	for _, user := range m.Users {
		if user.Username == username {
			return user, nil
		}
	}
	return nil, errors.New("user not found")
}

func (m *MockUserRepo) CreateUser(ctx context.Context, user *model.User) (*model.User, error) {
	if user.Username == "fail" {
		return nil, errors.New("mock create error")
	}
	user.ID = primitive.NewObjectID()
	m.Users = append(m.Users, user)
	return user, nil
}

