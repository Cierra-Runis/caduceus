package handler

import (
	"server/src/service"

	"github.com/gofiber/fiber/v3"
)

// import (
// 	"log"
// 	"net/http"
// 	"server/service"
// )

type UserHandler struct {
	userService *service.UserService
}

func NewUserHandler(userService *service.UserService) *UserHandler {
	return &UserHandler{userService: userService}
}

type CreateUserRequest struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

func (h *UserHandler) CreateUser(c fiber.Ctx) error {
	req := new(CreateUserRequest)

	if err := c.Bind().All(req); err != nil {
		return err
	}

	user, err := h.userService.CreateUser(c, req.Username, req.Password)

	if err != nil {
		switch err.Error() {
		case service.ErrUsernameTaken:
			return c.Status(fiber.StatusConflict).JSON(fiber.Map{"error": err.Error()})
		case service.ErrInvalidPassword:
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{"error": err.Error()})
		default:
			return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": err.Error()})
		}
	}

	return c.Status(fiber.StatusOK).JSON(user)
}

// 	var req CreateUserRequest
// 	if err := c.ShouldBindJSON(&req); err != nil {
// 		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request data: " + err.Error()})
// 		return
// 	}

// 	user, err := h.userService.CreateUser(c.Request.Context(), req.Username, req.Password)

// 	if err != nil {
// 		switch err.Error() {
// 		case service.ErrUsernameTaken:
// 			c.JSON(http.StatusConflict, gin.H{"error": err.Error()}) /// TODO: Add I18N support
// 		case service.ErrInvalidPassword:
// 			c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()}) /// TODO: Add I18N support
// 		default:
// 			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
// 			log.Printf("[Error] %v", err)
// 		}
// 		return
// 	}

// 	c.JSON(http.StatusOK, user)
// }

// type LoginUserRequest struct {
// 	Username string `json:"username" binding:"required"`
// 	Password string `json:"password" binding:"required"`
// }

// type LoginUserResponse struct {
// 	JWT string `json:"jwt"`
// }

// func (h *UserHandler) LoginUser(c *gin.Context) {
// 	var req LoginUserRequest
// 	if err := c.ShouldBindJSON(&req); err != nil {
// 		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request data: " + err.Error()})
// 		return
// 	}

// 	jwt, err := h.userService.AuthenticateUser(c.Request.Context(), req.Username, req.Password)
// 	if err != nil {
// 		switch err.Error() {
// 		case service.ErrUserNotFound:
// 			c.JSON(http.StatusNotFound, gin.H{"error": err.Error()}) /// TODO: Add I18N support
// 		case service.ErrInvalidPassword:
// 			c.JSON(http.StatusUnauthorized, gin.H{"error": err.Error()}) /// TODO: Add I18N support
// 		default:
// 			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
// 			log.Printf("[Error] %v", err)
// 		}
// 		return
// 	}

// 	c.JSON(http.StatusOK, LoginUserResponse{JWT: *jwt})
// }
