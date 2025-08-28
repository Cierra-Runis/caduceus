package handler

import (
	"server/src/model"
	"server/src/service"

	"github.com/gofiber/fiber/v3"
)

type ProjectHandler struct {
	projectService *service.ProjectService
}

func NewProjectHandler(projectService *service.ProjectService) *ProjectHandler {
	return &ProjectHandler{projectService: projectService}
}

type CreateProjectRequest struct {
	Name      string `json:"name" validate:"required"`
	OwnerID   string `json:"owner_id" validate:"required"`
	OwnerType string `json:"owner_type" validate:"required"`
}

type CreateProjectPayload struct {
	*model.Project
}

type CreateProjectResponse = model.Response[CreateProjectPayload]

func (h *ProjectHandler) CreateProject(c fiber.Ctx) error {
	req := new(CreateProjectRequest)

	if err := c.Bind().JSON(req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(CreateProjectResponse{Message: service.MsgInvalidRequestBody})
	}

	project, err := h.projectService.CreateProject(c, req.Name, req.OwnerID, req.OwnerType)

	if err != nil {
		switch err.Error() {
		case service.MsgInvalidRequestBody:
			return c.Status(fiber.StatusBadRequest).JSON(CreateProjectResponse{Message: err.Error()})
		case service.MsgProjectTypeInvalid:
			return c.Status(fiber.StatusBadRequest).JSON(CreateProjectResponse{Message: err.Error()})
		default:
			return c.Status(fiber.StatusInternalServerError).JSON(CreateProjectResponse{Message: err.Error()})
		}
	}

	return c.Status(fiber.StatusOK).JSON(CreateProjectResponse{
		Message: "Project created successfully",
		Payload: &CreateProjectPayload{Project: project},
	})
}
