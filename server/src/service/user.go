package service

import (
	"context"
	"errors"
	"server/src/model"
	"time"

	"golang.org/x/crypto/bcrypt"
)

const (
	ErrInvalidRequestBody = "invalid request body"
	ErrUsernameTaken      = "username already taken"
	ErrInvalidPassword    = "invalid password"
	ErrUserNotFound       = "user not found"
)

type UserService struct {
	Repo         model.UserRepository
	JwtSecret    string
	JwtTTL       time.Duration
	CookieSecure bool
}

func NewUserService(
	repo model.UserRepository,
	jwtSecret string,
	jwtTTL time.Duration,
	cookieSecure bool,
) *UserService {
	return &UserService{
		Repo:         repo,
		JwtSecret:    jwtSecret,
		JwtTTL:       jwtTTL,
		CookieSecure: cookieSecure,
	}
}

func (s *UserService) CreateUser(ctx context.Context, username string, password string) (*model.User, error) {
	_, err := s.Repo.GetUserByUsername(ctx, username)
	if err == nil {
		return nil, errors.New(ErrUsernameTaken)
	}

	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		return nil, errors.New(ErrInvalidPassword)
	}

	user := &model.User{
		Username:  username,
		Nickname:  username, // Default nickname is the same as username
		Password:  string(hashedPassword),
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}

	return s.Repo.CreateUser(ctx, user)
}

func (s *UserService) AuthenticateUser(ctx context.Context, username string, password string) (*string, *model.JwtCustomClaims, error) {
	user, err := s.Repo.GetUserByUsername(ctx, username)
	if err != nil {
		return nil, nil, errors.New(ErrUserNotFound)
	}

	if err := bcrypt.CompareHashAndPassword([]byte(user.Password), []byte(password)); err != nil {
		return nil, nil, errors.New(ErrInvalidPassword)
	}

	token, claims, err := model.GenerateToken(user, s.JwtSecret, time.Now(), s.JwtTTL)

	if err != nil {
		return nil, nil, err
	}
	return &token, &claims, nil
}
