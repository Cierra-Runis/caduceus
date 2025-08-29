# Server

[At the beginning](https://github.com/Cierra-Runis/caduceus/commit/094c20e29bb3f4949d8e1674323eb16e939fa137), I tried to use [Rust](https://www.rust-lang.org) as backend language and [Axum](https://) as the web framework for the server, because what I developed here, Caduceus, is a [Typst](https://typst.app) document editor, and Typst itself is written in Rust, so I thought it would be easier to integrate Typst with a Rust backend.

However, I found that it was too difficult to use Rust, its language features just drove me crazy, and [MongoDB's Rust driver](https://github.com/mongodb/mongo-rust-driver) lacks some features that I will mention later. So I [dropped](https://github.com/Cierra-Runis/caduceus/commit/916269342b79b2765a992aae45423217da230730) Rust, and switched to [Go](https://go.dev) and [Fiber](https://gofiber.io) as the backend language and web framework respectively.

Things went well after the switch, Go's simplicity made development much easier, and Fiber is a very good web framework; I like it a lot. They have great documentation and a large community, I can easily find solutions to problems I encounter during development.

At [this commit](https://github.com/Cierra-Runis/caduceus/commit/e2bf832cf8bf833724ff49b90156cc1800abddaa), I first met a feature that I needed but was not supported by MongoDB's Rust driver, or Rust itself, or I just couldn't figure out how to do it:

```go
type User struct {
  ID        primitive.ObjectID `bson:"_id,omitempty" json:"id"`
  Username  string             `bson:"username" json:"username"`
  Nickname  string             `bson:"nickname" json:"nickname"`
  Password  string             `bson:"password" json:"-"`
  CreatedAt time.Time          `bson:"created_at" json:"created_at"`
  UpdatedAt time.Time          `bson:"updated_at" json:"updated_at"`
}
```

Note that the `Password` field has a `json:"-"` tag, which means that when this struct is serialized to JSON, the `Password` field will be omitted, and `bson:"password"` means that when this struct is serialized to BSON (the format used by MongoDB), it will be stored in the `password` field.

This is a very useful feature, because when I return a `User` object to the client, I don't want to expose the password hash to the client, which is a security risk. In Rust, I had to create a separate struct for the response, which is cumbersome and error-prone. In Go, I can just use the same struct and add the `json:"-"` tag to the `Password` field.

Additionally, in Go, I can easily create a new struct that omits the `Password` field in another way:

```go
type UserPayload struct {
  User
  Password *string `json:"omitempty"`
}
```

Here, `UserPayload` embeds the `User` struct, and `Password` field will hide the `Password` field from the embedded `User` struct, and be omitted when serialized to JSON if it is `nil` (yes, if we **_do_** want to include the password in the payload, we can set it to a non-nil value), if we don't set it at all, it will be `nil` by default, which is the Go zero value for pointers.

I believed that I could just use Go and Fiber to develop the backend without any problem, but I was wrong. With development going on, I couldn't bear some of Go's shortcomings.

The first thing started to bother me was the zero-value initialization of structs. In Go, when we create a struct, all its fields are initialized to their zero values, which is usually not what I want. As mentioned above, the zero value of a pointer is `nil`, which reminds me of the `NullPointerException` in Java, and I hate it a lot. If I add a new field to a struct, I have to make sure that I initialize it properly everywhere, otherwise it will be `nil` by default, and if I forget to check for `nil` before using it, it will cause a runtime panic - maybe we could use a constructor function to create a struct, but I also hate writing [boilerplate code](https://en.wikipedia.org/wiki/Boilerplate_code).

Oh, I should mention more Go's benefits before I complain about it. Its compiling speed is amazing. With [air](https://github.com/air-verse/air), I can have a live-reloading development environment that re-compiles and restarts the server in less than a second after I save a file, which is great for productivity - Actually, I met hot-reloading in Dart/Flutter first. In another hand, Rust's compiling speed is terrible, it runs all my CPU cores at 100% for minutes. Does the compiler suck? No, It does run all CPU cores, but it is slow.

Later, I realized that Go lacks true enums, which are very useful for defining errors, statuses, and types, and are a powerful feature in Rust. In Go, we can use `const` and `iota` to define a set of related constants, but they are not type-safe, exhaustive, or pattern-matchable. I can use `switch` statements to mimic pattern matching, but it is nowhere near as elegant as Rust’s `match` expressions. Another option is to combine `struct` and `interface` to build an enum-like, type-safe structure, but this approach is cumbersome, error-prone, and still not exhaustive - plus, it introduces even more boilerplate code.

So I [back to Rust](https://github.com/Cierra-Runis/caduceus/commit/74894aec8aac7354664bb8f7bc43cd60e31ddc12) again, and [replaced the Go/Fiber](https://github.com/Cierra-Runis/caduceus/commit/0946d94b15b92c903abccc4d9c0534991db630dd).
