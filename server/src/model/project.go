package model

import (
	"context"
	"time"

	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
)

type Project struct {
	ID        primitive.ObjectID `bson:"_id,omitempty" json:"id"`
	Name      string             `bson:"name" json:"name"`
	OwnerID   primitive.ObjectID `bson:"owner_id" json:"owner_id"`
	OwnerType string             `bson:"owner_type" json:"owner_type"` // "USER" or "TEAM"
	CreatedAt time.Time          `bson:"created_at" json:"created_at"`
	UpdatedAt time.Time          `bson:"updated_at" json:"updated_at"`
}

type ProjectRepository interface {
	CreateProject(ctx context.Context, project *Project) (*Project, error)
}

type MongoProjectRepo struct {
	Collection *mongo.Collection
}

func NewMongoProjectRepo(db *mongo.Database) *MongoProjectRepo {
	return &MongoProjectRepo{
		Collection: db.Collection("projects"),
	}
}

func (r *MongoProjectRepo) CreateProject(ctx context.Context, project *Project) (*Project, error) {
	res, err := r.Collection.InsertOne(ctx, project)
	if err != nil {
		return nil, err
	}
	project.ID = res.InsertedID.(primitive.ObjectID)
	return project, nil
}
