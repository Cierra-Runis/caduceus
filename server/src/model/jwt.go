package model

import (
	"time"

	"github.com/golang-jwt/jwt/v5"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

type UserJwtClaims struct {
	ID       primitive.ObjectID `json:"id"`
	Username string             `json:"username"`
	jwt.RegisteredClaims
}

func (u *User) GenerateClaimsWith(issuedAt time.Time, ttl time.Duration) *UserJwtClaims {
	return &UserJwtClaims{
		ID:       u.ID,
		Username: u.Username,
		RegisteredClaims: jwt.RegisteredClaims{
			IssuedAt:  jwt.NewNumericDate(issuedAt),
			ExpiresAt: jwt.NewNumericDate(issuedAt.Add(ttl)),
			Subject:   u.ID.String(),
		},
	}
}

func (c *UserJwtClaims) GenerateSignedString(secret string) (string, error) {
	jwt := jwt.NewWithClaims(jwt.SigningMethodHS256, c)
	token, err := jwt.SignedString([]byte(secret))
	return token, err
}

func ParseStringToToken(token string, secret string, verifiedAt time.Time) (*jwt.Token, error) {
	parser := jwt.NewParser(
		jwt.WithValidMethods([]string{jwt.SigningMethodHS256.Alg()}),
		jwt.WithIssuedAt(),
		jwt.WithExpirationRequired(),
		jwt.WithTimeFunc(func() time.Time {
			return verifiedAt
		}),
	)

	jwtToken, err := parser.ParseWithClaims(token, &UserJwtClaims{}, func(token *jwt.Token) (interface{}, error) {
		return []byte(secret), nil
	})

	if err != nil {
		return nil, err
	}

	return jwtToken, nil
}
