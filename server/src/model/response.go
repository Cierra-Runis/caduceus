package model

type Response[T any] struct {
	Message string
	Data    *T
}
