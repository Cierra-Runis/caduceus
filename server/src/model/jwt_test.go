package model_test

import (
	"server/src/model"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

func TestGenerateToken(t *testing.T) {
	user := &model.User{
		ID:       primitive.NewObjectID(),
		Username: "test_user",
	}
	secret := "test_secret"
	issuedAt := time.Now()

	t.Run("successful_token_generation", func(t *testing.T) {
		token, _, err := model.GenerateToken(user, secret, issuedAt, 24*time.Hour)
		assert.NoError(t, err)
		assert.NotEmpty(t, token)

		parsedToken, err := jwt.ParseWithClaims(token, &model.JwtCustomClaims{}, func(token *jwt.Token) (interface{}, error) {
			return []byte(secret), nil
		})
		assert.NoError(t, err)
		assert.True(t, parsedToken.Valid)

		claims := parsedToken.Claims.(*model.JwtCustomClaims)
		assert.Equal(t, user.ID, claims.ID)
		assert.Equal(t, user.Username, claims.Username)
		assert.Equal(t, "Token", claims.Subject)
	})

	t.Run("token_expiration", func(t *testing.T) {
		token, _, err := model.GenerateToken(user, secret, issuedAt, 24*time.Hour)
		assert.NoError(t, err)

		parsedToken, err := jwt.ParseWithClaims(token, &model.JwtCustomClaims{}, func(token *jwt.Token) (interface{}, error) {
			return []byte(secret), nil
		})
		assert.NoError(t, err)

		claims := parsedToken.Claims.(*model.JwtCustomClaims)
		expectedExpiry := time.Now().Add(24 * time.Hour)
		assert.WithinDuration(t, expectedExpiry, claims.ExpiresAt.Time, time.Minute)
	})
}

func TestParseToken(t *testing.T) {
	user := &model.User{
		ID:       primitive.NewObjectID(),
		Username: "test_user",
	}
	secret := "test_secret"
	issuedAt := time.Now()
	token, originalClaims, err := model.GenerateToken(user, secret, issuedAt, 24*time.Hour)
	assert.NoError(t, err)
	assert.NotEmpty(t, token)

	t.Run("successful_token_parsing", func(t *testing.T) {
		claims, err := model.ParseToken(token, secret, issuedAt)
		assert.NoError(t, err)
		assert.Equal(t, originalClaims.ID, claims.ID)
		assert.Equal(t, originalClaims.Username, claims.Username)
		assert.Equal(t, originalClaims.Subject, claims.Subject)
	})

	t.Run("invalid_token", func(t *testing.T) {
		_, err := model.ParseToken("invalid_token", secret, issuedAt)
		assert.Error(t, err)
	})

	t.Run("token_expiration", func(t *testing.T) {
		token, _, err := model.GenerateToken(user, secret, issuedAt, 24*time.Hour)
		assert.NoError(t, err)

		_, err = model.ParseToken(token, secret, issuedAt.Add(48*time.Hour))
		assert.Error(t, err)
	})
}
