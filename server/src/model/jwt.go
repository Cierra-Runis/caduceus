package model

import (
	"time"

	"github.com/golang-jwt/jwt/v5"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

type JwtCustomClaims struct {
	ID       primitive.ObjectID `json:"id"`
	Username string             `json:"username"`
	jwt.RegisteredClaims
}

func GenerateToken(u *User, secret string) (string, error) {
	claims := JwtCustomClaims{
		ID:       u.ID,
		Username: u.Username,
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Duration(time.Hour) * 24)),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
			Subject:   "Token",
		},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString([]byte(secret))
}
