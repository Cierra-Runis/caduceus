package handler_test

import (
	"server/src/handler"
	"testing"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/stretchr/testify/assert"
)

func TestHandleWebSocketMessage(t *testing.T) {
	webSocketHandler := handler.NewWebSocketHandler()
	timestamp := time.Now().Unix()

	t.Run("compile_message", func(t *testing.T) {
		msg := handler.WebSocketMessage{
			Type:      "compile",
			Payload:   map[string]interface{}{"code": "test code"},
			Timestamp: timestamp,
		}

		response := webSocketHandler.HandleWebSocketMessage(msg)

		assert.Equal(t, "compile_result", response.Type)
		assert.NotNil(t, response.Payload)
		assert.Equal(t, timestamp, response.Timestamp)

		// fiber.Map is a type alias for map[string]interface{}
		if dataMap, ok := response.Payload.(fiber.Map); ok {
			assert.Equal(t, "processing", dataMap["status"])
			assert.Equal(t, "Compilation started", dataMap["message"])
		} else {
			t.Fatalf("Expected response.Payload to be fiber.Map, got %T", response.Payload)
		}
	})

	t.Run("ping_message", func(t *testing.T) {
		msg := handler.WebSocketMessage{
			Type:      "ping",
			Payload:   nil,
			Timestamp: timestamp,
		}

		response := webSocketHandler.HandleWebSocketMessage(msg)

		assert.Equal(t, "pong", response.Type)
		assert.NotNil(t, response.Payload)
		assert.Equal(t, timestamp, response.Timestamp)

		// fiber.Map is a type alias for map[string]interface{}
		if dataMap, ok := response.Payload.(fiber.Map); ok {
			assert.Equal(t, "Server is alive", dataMap["message"])
		} else {
			t.Fatalf("Expected response.Payload to be fiber.Map, got %T", response.Payload)
		}
	})

	t.Run("unknown_message_type", func(t *testing.T) {
		msg := handler.WebSocketMessage{
			Type:      "unknown",
			Payload:   nil,
			Timestamp: timestamp,
		}

		response := webSocketHandler.HandleWebSocketMessage(msg)

		assert.Equal(t, "error", response.Type)
		assert.NotNil(t, response.Payload)
		assert.Equal(t, timestamp, response.Timestamp)

		// fiber.Map is a type alias for map[string]interface{}
		if dataMap, ok := response.Payload.(fiber.Map); ok {
			assert.Equal(t, "Unknown message type", dataMap["message"])
		} else {
			t.Fatalf("Expected response.Payload to be fiber.Map, got %T", response.Payload)
		}
	})

	t.Run("empty_message_type", func(t *testing.T) {
		msg := handler.WebSocketMessage{
			Type:      "",
			Payload:   nil,
			Timestamp: timestamp,
		}

		response := webSocketHandler.HandleWebSocketMessage(msg)

		assert.Equal(t, "error", response.Type)
		assert.NotNil(t, response.Payload)
		assert.Equal(t, timestamp, response.Timestamp)
	})
}
