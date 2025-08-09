package handler

import (
	"fmt"
	"testing"
	"time"
)

func TestDebugWebSocketMessage(t *testing.T) {
	timestamp := time.Now().Unix()
	msg := WebSocketMessage{
		Type:      "compile",
		Data:      map[string]interface{}{"code": "test code"},
		Timestamp: timestamp,
	}

	response := handleWebSocketMessage(msg)
	fmt.Printf("Response.Data type: %T\n", response.Data)
	fmt.Printf("Response.Data value: %v\n", response.Data)
}
