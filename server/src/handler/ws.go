package handler

import (
	"github.com/gofiber/fiber/v3"
	"github.com/kiuber/gofiber3-contrib/websocket"
)

type WebSocketMessage struct {
	Type      string      `json:"type"`
	Data      interface{} `json:"data,omitempty"`
	Timestamp int64       `json:"timestamp"`
}

func WebSocket(c *websocket.Conn) {
	defer c.Close()

	for {
		var msg WebSocketMessage
		if err := c.ReadJSON(&msg); err != nil {
			break
		}

		response := handleWebSocketMessage(msg)

		if err := c.WriteJSON(response); err != nil {
			break
		}
	}

}

func handleWebSocketMessage(msg WebSocketMessage) WebSocketMessage {
	switch msg.Type {
	case "compile":
		return WebSocketMessage{
			Type: "compile_result",
			Data: fiber.Map{
				"status":  "processing",
				"message": "Compilation started",
			},
			Timestamp: msg.Timestamp,
		}
	case "ping":
		return WebSocketMessage{
			Type: "pong",
			Data: fiber.Map{
				"message": "Server is alive",
			},
			Timestamp: msg.Timestamp,
		}
	default:
		return WebSocketMessage{
			Type: "error",
			Data: fiber.Map{
				"message": "Unknown message type",
			},
			Timestamp: msg.Timestamp,
		}
	}
}
