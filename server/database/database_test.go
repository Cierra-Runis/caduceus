package database

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestNewMongoClient_InvalidURI(t *testing.T) {
	t.Run("invalid_mongo_uri", func(t *testing.T) {
		client, err := NewMongoClient("invalid://uri", "testdb", time.Second)
		assert.Error(t, err)
		assert.Nil(t, client)
	})

	t.Run("empty_uri", func(t *testing.T) {
		client, err := NewMongoClient("", "testdb", time.Second)
		assert.Error(t, err)
		assert.Nil(t, client)
	})

	t.Run("empty_db_name", func(t *testing.T) {
		// This should still work as MongoDB allows empty database names (though not recommended)
		client, err := NewMongoClient("mongodb://invalid:27017", "", time.Second)
		assert.Error(t, err) // Will fail due to connection, not empty DB name
		assert.Nil(t, client)
	})
}

func TestMongoClient_Struct(t *testing.T) {
	// Test the MongoClient struct definition
	client := &MongoClient{
		Client: nil,
		DB:     nil,
	}

	assert.NotNil(t, client)
	assert.Nil(t, client.Client)
	assert.Nil(t, client.DB)
}
