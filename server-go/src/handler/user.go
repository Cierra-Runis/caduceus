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

type CreateUserPayload struct {
	*model.User
	Password string `json:"password,omitempty"`
}

type CreateUserResponse = model.Response[CreateUserPayload]

func (h *UserHandler) CreateUser(c fiber.Ctx) error {
	req := new(CreateUserRequest)

	if err := c.Bind().JSON(req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(CreateUserResponse{Message: service.MsgInvalidRequestBody})
	}

	user, err := h.userService.CreateUser(c, req.Username, req.Password)

	if err != nil {
		switch err.Error() {
		case service.MsgUsernameTaken:
			return c.Status(fiber.StatusConflict).JSON(CreateUserResponse{Message: err.Error()})
		case service.MsgInvalidPassword:
			return c.Status(fiber.StatusBadRequest).JSON(CreateUserResponse{Message: err.Error()})
		default:
			return c.Status(fiber.StatusInternalServerError).JSON(CreateUserResponse{Message: err.Error()})
		}
	}

	return c.Status(fiber.StatusOK).JSON(CreateUserResponse{
		Message: "User created successfully",
		Payload: &CreateUserPayload{User: user},
	})
}

type LoginRequest struct {
	Username string `json:"username" validate:"required"`
	Password string `json:"password" validate:"required"`
}

type LoginPayload struct {
	Token string `json:"token"`
}

type LoginResponse = model.Response[LoginPayload]

func (h *UserHandler) LoginUser(c fiber.Ctx) error {
	req := new(LoginRequest)

	if err := c.Bind().JSON(req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(LoginResponse{Message: service.MsgInvalidRequestBody})
	}

	cookie, err := h.userService.LoginUser(c, req.Username, req.Password)

	if err != nil {
		switch err.Error() {
		case service.MsgUserNotFound:
			return c.Status(fiber.StatusNotFound).JSON(LoginResponse{Message: err.Error()})
		case service.MsgInvalidPassword:
			return c.Status(fiber.StatusUnauthorized).JSON(LoginResponse{Message: err.Error()})
		default:
			return c.Status(fiber.StatusInternalServerError).JSON(LoginResponse{Message: err.Error()})
		}
	}

	c.Status(fiber.StatusOK).Cookie(cookie)

	return c.JSON(LoginResponse{
		Message: "Login successful",
		Payload: &LoginPayload{
			Token: cookie.Value,
		},
	})
}
