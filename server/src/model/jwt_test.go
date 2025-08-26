package model_test

import (
	"testing"
	"time"

	"server/src/model"

	"github.com/golang-jwt/jwt/v5"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

func Test_Model_GenerateClaimsWith(t *testing.T) {
	user := &model.User{
		ID:       primitive.NewObjectID(),
		Username: "test_user",
	}
	issuedAt := time.Now()
	ttl := time.Hour

	claims := user.GenerateClaimsWith(issuedAt, ttl)

	assert.Equal(t, user.ID, claims.ID)
	assert.Equal(t, user.Username, claims.Username)
	assert.Equal(t, user.ID.String(), claims.Subject)
	assert.WithinDuration(t, issuedAt, claims.IssuedAt.Time, time.Second)
	assert.WithinDuration(t, issuedAt.Add(ttl), claims.ExpiresAt.Time, time.Second)

	t.Run("successful_generate_token", func(t *testing.T) {
		secret := "test_secret"

		token, err := claims.GenerateSignedString(secret)
		assert.NoError(t, err)
		assert.NotEmpty(t, token)

		t.Run("ParseStringToToken", func(t *testing.T) {

			t.Run("successful_parse_token", func(t *testing.T) {
				jwtToken, err := model.ParseStringToToken(token, secret, issuedAt.Add(ttl/2))
				assert.NoError(t, err)

				parsedClaims, ok := jwtToken.Claims.(*model.UserJwtClaims)
				assert.True(t, ok)
				assert.Equal(t, claims, parsedClaims)
			})

			t.Run("invalid_token", func(t *testing.T) {
				_, err := model.ParseStringToToken(token+"invalid", secret, issuedAt.Add(ttl/2))
				assert.ErrorIs(t, err, jwt.ErrTokenSignatureInvalid)
			})

			t.Run("invalid_secret", func(t *testing.T) {
				_, err := model.ParseStringToToken(token, secret+"invalid", issuedAt.Add(ttl/2))
				assert.ErrorIs(t, err, jwt.ErrTokenSignatureInvalid)
			})

			t.Run("expired_token", func(t *testing.T) {
				_, err := model.ParseStringToToken(token, secret, issuedAt.Add(2*ttl))
				assert.ErrorIs(t, err, jwt.ErrTokenExpired)
			})
		})
	})
}
