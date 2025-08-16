package middleware_test

import (
	"server/src/middleware"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestStructValidator_Validate(t *testing.T) {
	mockValidator := middleware.NewStructValidator()

	t.Run("valid_struct", func(t *testing.T) {
		type TestStruct struct {
			Name string `validate:"required"`
		}

		testData := TestStruct{Name: "Valid Name"}
		err := mockValidator.Validate(testData)
		assert.NoError(t, err)
	})

	t.Run("invalid_struct", func(t *testing.T) {
		type TestStruct struct {
			Name string `validate:"required"`
		}

		testData := TestStruct{}
		err := mockValidator.Validate(testData)
		assert.Error(t, err)
	})
}
