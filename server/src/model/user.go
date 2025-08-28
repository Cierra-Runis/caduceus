package model

import (
	"context"
	"time"

	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
)

type User struct {
	ID        primitive.ObjectID `bson:"_id,omitempty" json:"id"`
	Username  string             `bson:"username" json:"username"`
	Nickname  string             `bson:"nickname" json:"nickname"`
	Password  string             `bson:"password" json:"password"`
	CreatedAt time.Time          `bson:"created_at" json:"created_at"`
	UpdatedAt time.Time          `bson:"updated_at" json:"updated_at"`
}

type UserRepository interface {
	GetUserByUsername(ctx context.Context, username string) (*User, error)
	CreateUser(ctx context.Context, user *User) (*User, error)
}

type MongoUserRepo struct {
	Collection *mongo.Collection
}

func NewMongoUserRepo(db *mongo.Database) *MongoUserRepo {
	return &MongoUserRepo{
		Collection: db.Collection("users"),
	}
}



func (r *MongoUserRepo) GetUserByUsername(ctx context.Context, username string) (*User, error) {
	var user User
	err := r.Collection.FindOne(ctx, bson.M{"username": username}).Decode(&user)
	if err != nil {
		return nil, err
	}
	return &user, nil
}

func (r *MongoUserRepo) CreateUser(ctx context.Context, user *User) (*User, error) {
	res, err := r.Collection.InsertOne(ctx, user)
	if err != nil {
		return nil, err
	}
	user.ID = res.InsertedID.(primitive.ObjectID)
	return user, nil
}

