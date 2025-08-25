package model

type Response[T any] struct {
	Message string `json:"message"`
	Payload *T     `json:"payload,omitempty"`
}
