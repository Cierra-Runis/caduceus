package handler

import (
	"testing"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/stretchr/testify/assert"
)

func TestHandleWebSocketMessage(t *testing.T) {
	webSocketHandler := NewWebSocketHandler()
	timestamp := time.Now().Unix()

	t.Run("compile_message", func(t *testing.T) {
		msg := WebSocketMessage{
			Type:      "compile",
			Data:      map[string]interface{}{"code": "test code"},
			Timestamp: timestamp,
		}

		response := webSocketHandler.HandleWebSocketMessage(msg)

		assert.Equal(t, "compile_result", response.Type)
		assert.NotNil(t, response.Data)
		assert.Equal(t, timestamp, response.Timestamp)

		// fiber.Map is a type alias for map[string]interface{}
		if dataMap, ok := response.Data.(fiber.Map); ok {
			assert.Equal(t, "processing", dataMap["status"])
			assert.Equal(t, "Compilation started", dataMap["message"])
		} else {
			t.Fatalf("Expected response.Data to be fiber.Map, got %T", response.Data)
		}
	})

	t.Run("ping_message", func(t *testing.T) {
		msg := WebSocketMessage{
			Type:      "ping",
			Data:      nil,
			Timestamp: timestamp,
		}

		response := webSocketHandler.HandleWebSocketMessage(msg)

		assert.Equal(t, "pong", response.Type)
		assert.NotNil(t, response.Data)
		assert.Equal(t, timestamp, response.Timestamp)

		// fiber.Map is a type alias for map[string]interface{}
		if dataMap, ok := response.Data.(fiber.Map); ok {
			assert.Equal(t, "Server is alive", dataMap["message"])
		} else {
			t.Fatalf("Expected response.Data to be fiber.Map, got %T", response.Data)
		}
	})

	t.Run("unknown_message_type", func(t *testing.T) {
		msg := WebSocketMessage{
			Type:      "unknown",
			Data:      nil,
			Timestamp: timestamp,
		}

		response := webSocketHandler.HandleWebSocketMessage(msg)

		assert.Equal(t, "error", response.Type)
		assert.NotNil(t, response.Data)
		assert.Equal(t, timestamp, response.Timestamp)

		// fiber.Map is a type alias for map[string]interface{}
		if dataMap, ok := response.Data.(fiber.Map); ok {
			assert.Equal(t, "Unknown message type", dataMap["message"])
		} else {
			t.Fatalf("Expected response.Data to be fiber.Map, got %T", response.Data)
		}
	})

	t.Run("empty_message_type", func(t *testing.T) {
		msg := WebSocketMessage{
			Type:      "",
			Data:      nil,
			Timestamp: timestamp,
		}

		response := webSocketHandler.HandleWebSocketMessage(msg)

		assert.Equal(t, "error", response.Type)
		assert.NotNil(t, response.Data)
		assert.Equal(t, timestamp, response.Timestamp)
	})
}
