package service

import "errors"

const (
	MsgInvalidRequestBody = "invalid request body"
)

var (
	ErrInvalidRequestBody = errors.New(MsgInvalidRequestBody)
)
