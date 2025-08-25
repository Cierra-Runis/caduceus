package model

import (
	"errors"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

const (
	ErrTokenExpired = "token has expired"
)

type JwtCustomClaims struct {
	ID       primitive.ObjectID `json:"id"`
	Username string             `json:"username"`
	jwt.RegisteredClaims
}

func GenerateToken(u *User, secret string, issuedAt time.Time, ttl time.Duration) (string, JwtCustomClaims, error) {
	claims := JwtCustomClaims{
		ID:       u.ID,
		Username: u.Username,
		RegisteredClaims: jwt.RegisteredClaims{
			IssuedAt:  jwt.NewNumericDate(issuedAt),
			ExpiresAt: jwt.NewNumericDate(issuedAt.Add(ttl)),
			Subject:   "Token",
		},
	}

	jwt := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	token, err := jwt.SignedString([]byte(secret))

	return token, claims, err
}

func ParseToken(token string, secret string, verifiedAt time.Time) (*JwtCustomClaims, error) {
	jwt, err := jwt.ParseWithClaims(token, &JwtCustomClaims{}, func(token *jwt.Token) (interface{}, error) {
		return []byte(secret), nil
	})

	if err != nil {
		return nil, err
	}

	claims := jwt.Claims.(*JwtCustomClaims)

	if claims.ExpiresAt.Before(verifiedAt) {
		return nil, errors.New(ErrTokenExpired)
	}

	return claims, nil
}
