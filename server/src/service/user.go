package service

import (
	"context"
	"errors"
	"server/src/model"
	"time"

	"github.com/gofiber/fiber/v3"
	"golang.org/x/crypto/bcrypt"
)

const (
	MsgUsernameTaken   = "username already taken"
	MsgInvalidPassword = "invalid password"
	MsgUserNotFound    = "user not found"
)

var (
	ErrUsernameTaken   = errors.New(MsgUsernameTaken)
	ErrInvalidPassword = errors.New(MsgInvalidPassword)
	ErrUserNotFound    = errors.New(MsgUserNotFound)
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
		return nil, ErrUsernameTaken
	}

	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		return nil, ErrInvalidPassword
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

func (s *UserService) LoginUser(ctx context.Context, username string, password string) (*fiber.Cookie, error) {
	user, err := s.Repo.GetUserByUsername(ctx, username)
	if err != nil {
		return nil, ErrUserNotFound
	}

	if err := bcrypt.CompareHashAndPassword([]byte(user.Password), []byte(password)); err != nil {
		return nil, ErrInvalidPassword
	}

	claims := user.GenerateClaimsWith(time.Now(), s.JwtTTL)
	token, err := claims.GenerateSignedString(s.JwtSecret)

	if err != nil {
		return nil, err
	}

	return &fiber.Cookie{
		Name:     "jwt",
		Value:    token,
		HTTPOnly: true,
		Secure:   s.CookieSecure,
		SameSite: fiber.CookieSameSiteLaxMode,
		Expires:  claims.ExpiresAt.Time,
	}, nil
}
