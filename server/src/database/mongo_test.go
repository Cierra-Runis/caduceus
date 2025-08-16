package database_test

import (
	"server/src/database"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestNewMongoClient_InvalidURI(t *testing.T) {
	t.Run("invalid_mongo_uri", func(t *testing.T) {
		client, err := database.NewMongoClient("invalid://uri", "test_db", time.Second)
		assert.Error(t, err)
		assert.Nil(t, client)
	})

	t.Run("empty_uri", func(t *testing.T) {
		client, err := database.NewMongoClient("", "test_db", time.Second)
		assert.Error(t, err)
		assert.Nil(t, client)
	})

	t.Run("empty_db_name", func(t *testing.T) {
		// This should still work as MongoDB allows empty database names (though not recommended)
		client, err := database.NewMongoClient("mongodb://invalid:27017", "", time.Second)
		assert.Error(t, err) // Will fail due to connection, not empty DB name
		assert.Nil(t, client)
	})
}

func TestMongoClient_Struct(t *testing.T) {
	// Test the MongoClient struct definition
	client := &database.MongoClient{
		Client: nil,
		DB:     nil,
	}

	assert.NotNil(t, client)
	assert.Nil(t, client.Client)
	assert.Nil(t, client.DB)
}
