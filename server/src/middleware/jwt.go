package middleware

import (
	"server/src/model"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/keyauth"
)

type JWTMiddlewarePayload struct{}
type JWTMiddlewareResponse = model.Response[JWTMiddlewarePayload]

func NewJWTMiddleware(secret string) fiber.Handler {
	return keyauth.New(keyauth.Config{
		KeyLookup: "cookie:jwt",
		Validator: func(c fiber.Ctx, token string) (bool, error) {
			_, err := model.ParseStringToToken(token, secret, time.Now())
			if err != nil {
				return false, err
			}
			return true, nil
		},
		ErrorHandler: func(c fiber.Ctx, err error) error {
			return c.Status(fiber.StatusUnauthorized).JSON(JWTMiddlewareResponse{
				Message: "Unauthorized",
			})
		},
	})
}
