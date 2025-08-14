package model

import (
	"context"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

func TestGenerateToken(t *testing.T) {
	user := &User{
		ID:       primitive.NewObjectID(),
		Username: "testuser",
	}
	secret := "test_secret"

	t.Run("successful_token_generation", func(t *testing.T) {
		token, err := GenerateToken(user, secret)
		assert.NoError(t, err)
		assert.NotEmpty(t, token)

		// Verify token can be parsed
		parsedToken, err := jwt.ParseWithClaims(token, &JwtCustomClaims{}, func(token *jwt.Token) (interface{}, error) {
			return []byte(secret), nil
		})
		assert.NoError(t, err)
		assert.True(t, parsedToken.Valid)

		claims := parsedToken.Claims.(*JwtCustomClaims)
		assert.Equal(t, user.ID, claims.ID)
		assert.Equal(t, user.Username, claims.Username)
		assert.Equal(t, "Token", claims.Subject)
	})

	t.Run("token_expiration", func(t *testing.T) {
		token, err := GenerateToken(user, secret)
		assert.NoError(t, err)

		parsedToken, err := jwt.ParseWithClaims(token, &JwtCustomClaims{}, func(token *jwt.Token) (interface{}, error) {
			return []byte(secret), nil
		})
		assert.NoError(t, err)

		claims := parsedToken.Claims.(*JwtCustomClaims)
		expectedExpiry := time.Now().Add(24 * time.Hour)
		assert.WithinDuration(t, expectedExpiry, claims.ExpiresAt.Time, time.Minute)
	})
}

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
			Username: "newuser",
			Password: "password",
		}

		createdUser, err := repo.CreateUser(ctx, user)
		assert.NoError(t, err)
		assert.NotNil(t, createdUser)
		assert.Equal(t, "newuser", createdUser.Username)
		assert.NotEqual(t, primitive.NilObjectID, createdUser.ID)
		assert.Len(t, repo.Users, 1)
	})

	t.Run("mock_error_scenario", func(t *testing.T) {
		user := &User{
			Username: "fail", // This triggers mock error
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
		Username:  "testuser",
		Nickname:  "Test User",
		Password:  "hashedpassword",
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}

	assert.NotEqual(t, primitive.NilObjectID, user.ID)
	assert.Equal(t, "testuser", user.Username)
	assert.Equal(t, "Test User", user.Nickname)
	assert.Equal(t, "hashedpassword", user.Password)
	assert.WithinDuration(t, time.Now(), user.CreatedAt, time.Second)
	assert.WithinDuration(t, time.Now(), user.UpdatedAt, time.Second)
}
