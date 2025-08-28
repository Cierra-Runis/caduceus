package model

import (
	"context"
	"time"

	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
)

type Team struct {
	ID        primitive.ObjectID   `bson:"_id,omitempty" json:"id"`
	Name      string               `bson:"name" json:"name"`
	CreatorID primitive.ObjectID   `bson:"creator_id" json:"creator_id"`
	MemberIDs []primitive.ObjectID `bson:"member_ids" json:"member_ids"`
	CreatedAt time.Time            `bson:"created_at" json:"created_at"`
	UpdatedAt time.Time            `bson:"updated_at" json:"updated_at"`
}

type TeamRepository interface {
	CreateTeam(ctx context.Context, team *Team) (*Team, error)
	GetTeamByID(ctx context.Context, teamID string) (*Team, error)
}

type MongoTeamRepo struct {
	Collection *mongo.Collection
}

func NewMongoTeamRepo(db *mongo.Database) *MongoTeamRepo {
	return &MongoTeamRepo{
		Collection: db.Collection("teams"),
	}
}

func (r *MongoTeamRepo) CreateTeam(ctx context.Context, team *Team) (*Team, error) {
	res, err := r.Collection.InsertOne(ctx, team)
	if err != nil {
		return nil, err
	}
	team.ID = res.InsertedID.(primitive.ObjectID)
	return team, nil
}

func (r *MongoTeamRepo) GetTeamByID(ctx context.Context, teamID string) (*Team, error) {
	objectID, err := primitive.ObjectIDFromHex(teamID)
	if err != nil {
		return nil, err
	}

	var team Team
	err = r.Collection.FindOne(ctx, bson.M{"_id": objectID}).Decode(&team)
	if err != nil {
		return nil, err
	}
	return &team, nil
}
