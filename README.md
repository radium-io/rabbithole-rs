# rabbithole-rs [![Build Status](https://travis-ci.com/UkonnRa/rabbithole-rs.svg?branch=master)](https://travis-ci.com/UkonnRa/rabbithole-rs) ![crates.io](https://img.shields.io/crates/v/rabbithole.svg)

> The rabbit-hole went straight on like a tunnel for some way, and then dipped suddenly down,
> so suddenly that Alice had not a moment to think about stopping herself before she found herself falling down what seemed to be a very deep well.
>
> -- <cite>Alice's Adventures in Wonderland, by Lewis Carroll</cite>

## Introduction

Rabbithole-rs is a nearly well-typed, user-friendly [JSON:API](https://jsonapi.org/) type system,
with an easy-to-use Macro System to help you modelling the your data.

Inspired a lot by [jsonapi-rust](https://github.com/michiel/jsonapi-rust), in fact, all of the sample data in `tests` are just from this crate. Nice job, michiel!


### So what is [JSON:API](https://jsonapi.org/)?

> If you’ve ever argued with your team about the way your JSON responses should be formatted, JSON:API can be your anti-bikeshedding tool.
> 
> By following shared conventions, you can increase productivity, take advantage of generalized tooling, and focus on what matters: your application.

When you are designing a RESTful API, the most troubling problem is **how to design the Data Structure**, especially how to **design the Error System**. So JSON:API
design a Specification for the people like you to specify some rules to help you handling the design problem and free your days!

Maybe the specification is LONG LONG and boring, like reading a textbook, but believe me, you will learn a lot from it, just like a textbook. :)

### So why yet another JSON:API crate?

#### RSQL support needed

One of the main reason of this crate is that I need to support RSQL/FIQL, for I think it is a well-defined query system for complex querying. The JSON:API does not
given a official Query/Filter solution, but I think RSQL/FIQL is good enough to handle my project.

For more infomation about RSQL/FIQL, see:

- [rsql-rs](https://github.com/UkonnRa/rsql-rs), my project, please have a look and give me a Star, THX!
- [rsql-parser](https://github.com/jirutka/rsql-parser) (Maybe the best RSQL parser implementation)
- [FIQL Specification](https://tools.ietf.org/html/draft-nottingham-atompub-fiql-00)

#### Well Typed System

As a Scala player, I believe a well designed type system can just avoid a lot of problem.
And I want to know if Rust's ADT System can handle the problem with this kind of complexity.
In fact, it can handle it well.

#### An user-friendly Macro and Modelling

As a Java developer, I prefer the annotation system a lot. Thankfully, Rust uses [proc_macro](https://doc.rust-lang.org/reference/procedural-macros.html) system
to give the users "most-exactly-the-same" experience.

For example, instead of:

```rust
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Dog {
    id: String,
    name: String,
    age: i32,
    main_flea: Flea,
    fleas: Vec<Flea>,
}
jsonapi_model!(Dog; "dog"; has one main_flea; has many fleas);
```

I can model my data structure like this:

```rust
#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "dogs")]
pub struct Dog<'a> {
    #[entity(id)]
    pub id: String,
    pub name: String,
    #[entity(to_many)]
    pub fleas: Vec<Flea>,
    #[entity(to_many)]
    pub friends: Vec<Dog<'a>>,
    #[entity(to_one)]
    #[serde(bound(deserialize = "Box<Human<'a>>: Deserialize<'de>"))]
    pub master: Box<Human<'a>>,
    #[entity(to_one)]
    pub best_one: Option<Box<Dog<'a>>>,
}
```
For me, the second one is more beautiful.

## Features

- [x] Basic JSON:API model system
- [x] Basic Macro System
- [x] Basic tests

- [ ] Query/Filter API
  - [x] Query/Filter model
  - [ ] The auto query/filter is impossible now. Need to be finished by User
- [x] [Stricter type checking and error hints](#type-checking-and-error-hints-in-macro-system)
- [ ] [A high performance JSON:API Server](#a-high-performance-server)
  - [x] actix backend

## Future Works

### Type checking and error hints in Marco System

There are lots of type restrictions in rabbithole-rs. for example:

- `[#to_one]` decorator can only be use on a **field** with the type of being:
  - a `rabbithole::entity::Entity`
  - or a *wrapper* class of `rabbithole::entity::Entity`, where `wrapper` class is one of:
    - `Option<T>`
    - `Box<T>`
    - `&T`
    - A wrapper class inside another, like `Option<Box<T>>`

- `#[to_many]` decorator can only use on a **field** with (all of):
  - an iterator,
  - the inner type of the iterator should meet the above restriction
  - no nested List (discussing)

Now because of lacking the Reflection in Rust, the macro now can not check type errors at all, so some solutions may needed.

### A high performance Server

Because the API interface of JSON:API is complex, I think it's a redundant and boring work to write all the API interface following the specification yourself,
so I will do the boring things for you!

## My final goal to this project

The final goal of the project is just like [crnk](https://www.crnk.io/) or [elide](https://elide.io), who can auto generate a bunch of API based on **JUST** the definition of the models (maybe DAOs).
Here I want to just show what will the project look like finally.

### Define the models

The first step is define some *API-friendly* models.

```rust
// This is the derive crate which you can use to generate JSON:API specific traits
extern crate rabbithole_derive as rbh_derive;

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "people")]
pub struct Human {
    #[entity(id)]
    pub id_code: Uuid,
    pub name: String,
    #[entity(to_many)]
    pub dogs: Vec<Dog>,
}

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "dogs")]
pub struct Dog {
    #[entity(id)]
    pub id: Uuid,
    pub name: String,
}
```

### Write your own DAOs

`rabbithole` does not bind with any specific databases, which means you have to write your own DAOs.

See `rabbithole-endpoint-actix/examples/mock_gen.rs` for more details.

#### What is `Fetching` trait

`Fetching` trait is a mapping of ["fetching data" part in JSON:API](https://jsonapi.org/format/#fetching), which define a several operations:

- Fetching Resources
  - Single resource: `GET /articles/1`
  - Multiple resources: `GET /articles`
  - Related resource: `GET /articles/1/author`, which can be both single and multiple
- Fetching relationships
  - Relationship: `GET /articles/1/relationships/comments`
- Fenching with query parameters
  - Inclusion of Related Resources (`include` part)
  - Sparse Fieldsets (`fields[TYPE]` part)
  - Sorting (`sort` part)
  - Pagination (`page` part)

These are all we need to know in `fetching data` part. So these operation are abstracted into `Fenching` trait.

#### What is `vec_to_document` part?

If you want to transform a `Vec<SingleEntity>` into `Document`, it will do a lot of things like
[excluding un-included resources](https://jsonapi.org/format/#fetching-includes),
[retaining sparse fields](https://jsonapi.org/format/#fetching-sparse-fieldsets),
etc. and etc., and of course I can help you in the background (using `Entity::to_document_automatically`).
But more than extracting all the fields from databases and dropping them later, why not just leaving them in databases?
So here is the point. If you don't want to write the `Vec<SingleEntity> to Document` code, just use `Entity::to_document_automatically`,
or, you could assemble the `Document` directly from the database.

#### What is `...`(any other) part?

- `fetch_collection` will be mapped into: `/<ty>?<query>`
- `fetch_single` will be mapped into: `/<ty>/<id>?<query>`
- `fetch_relationship` will be mapped into: `/<ty>/<id>/relationships/<related_field>?<query>`
- `fetch_related` will be mapped into: `/<ty>/<id>/<related_field>?<query>`
- `type Error` will be mapped into the error responses if possible
- `type Item` must be a `SingleEntity`