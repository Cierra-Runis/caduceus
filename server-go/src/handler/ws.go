package handler

import (
	"github.com/gofiber/fiber/v3"
)

type WebSocketHandler struct{}

func NewWebSocketHandler() *WebSocketHandler {
	return &WebSocketHandler{}
}

type WebSocketMessage struct {
	Type      string      `json:"type"`
	Payload   interface{} `json:"data,omitempty"`
	Timestamp int64       `json:"timestamp"`
}

func (h *WebSocketHandler) HandleWebSocketMessage(msg WebSocketMessage) WebSocketMessage {
	switch msg.Type {
	case "compile":
		return WebSocketMessage{
			Type: "compile_result",
			Payload: fiber.Map{
				"status":  "processing",
				"message": "Compilation started",
			},
			Timestamp: msg.Timestamp,
		}
	case "ping":
		return WebSocketMessage{
			Type: "pong",
			Payload: fiber.Map{
				"message": "Server is alive",
			},
			Timestamp: msg.Timestamp,
		}
	default:
		return WebSocketMessage{
			Type: "error",
			Payload: fiber.Map{
				"message": "Unknown message type",
			},
			Timestamp: msg.Timestamp,
		}
	}
}
