package handler

import (
	"server/src/model"
	"server/src/service"

	"github.com/gofiber/fiber/v3"
)

type UserHandler struct {
	userService *service.UserService
}

func NewUserHandler(userService *service.UserService) *UserHandler {
	return &UserHandler{userService: userService}
}

type CreateUserRequest struct {
	Username string `json:"username" validate:"required"`
	Password string `json:"password" validate:"required"`
}

type CreateUserResponse = model.Response[model.User]

func (h *UserHandler) CreateUser(c fiber.Ctx) error {
	req := new(CreateUserRequest)

	if err := c.Bind().JSON(req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(CreateUserResponse{Message: service.ErrInvalidRequestBody})
	}

	user, err := h.userService.CreateUser(c, req.Username, req.Password)

	if err != nil {
		switch err.Error() {
		case service.ErrUsernameTaken:
			return c.Status(fiber.StatusConflict).JSON(CreateUserResponse{Message: err.Error()})
		case service.ErrInvalidPassword:
			return c.Status(fiber.StatusBadRequest).JSON(CreateUserResponse{Message: err.Error()})
		default:
			return c.Status(fiber.StatusInternalServerError).JSON(CreateUserResponse{Message: err.Error()})
		}
	}

	return c.Status(fiber.StatusOK).JSON(CreateUserResponse{
		Message: "User created successfully",
		Data:    user,
	})
}

// type LoginRequest struct {
// 	Username string `json:"username" binding:"required"`
// 	Password string `json:"password" binding:"required"`
// }

// type LoginResponse struct {
// 	Error string `json:"error,omitempty"`
// 	Token string `json:"token"`
// }

// func (h *UserHandler) Login(c fiber.Ctx) error {
// 	req := new(LoginRequest)

// 	if err := c.Bind().All(req); err != nil {
// 		return err
// 	}

// 	token, err := h.userService.AuthenticateUser(c, req.Username, req.Password)

// 	if err != nil {
// 		switch err.Error() {
// 		case service.ErrUserNotFound:
// 			return c.Status(fiber.StatusNotFound).JSON(model.Response[LoginResponse]{Message: err.Error()})
// 		case service.ErrInvalidPassword:
// 			return c.Status(fiber.StatusUnauthorized).JSON(model.Response[LoginResponse]{Message: err.Error()})
// 		default:
// 			return c.Status(fiber.StatusInternalServerError).JSON(model.Response[LoginResponse]{Message: err.Error()})
// 		}
// 	}

// 	return c.Status(fiber.StatusOK).JSON(model.Response[LoginResponse]{
// 		Message: "Login successful",
// 		Data:    &LoginResponse{Token: *token},
// 	})
// }
