package middleware

import "github.com/go-playground/validator/v10"

type StructValidator struct {
	InnerValidate *validator.Validate
}

func NewStructValidator() *StructValidator {
	return &StructValidator{
		InnerValidate: validator.New(),
	}
}

// Validator needs to implement the Validate method
func (v *StructValidator) Validate(out any) error {
	return v.InnerValidate.Struct(out)
}
