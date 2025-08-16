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

	t.Run("successful_token_generation", func(t *testing.T) {
		token, err := model.GenerateToken(user, secret)
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
		token, err := model.GenerateToken(user, secret)
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
