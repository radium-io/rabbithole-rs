# Changelog

## [Unreleased]

## [0.2.2] - 2019-11-10

## [0.2.1] - 2019-11-10

## Fixed

- Publish error: all dependencies must have a version specified when publishing.

## [0.2.0] - 2019-11-10

The most important and bothering thing is that [`attributes` cannot be sorted](https://github.com/UkonnRa/rabbithole-rs/issues/1)

### Added

- `rabbithole::model::query::Query` model for 
  - [Inclusion of Related Resources](https://jsonapi.org/format/#fetching-includes)
  - [Sparse Fields Set](https://jsonapi.org/format/#fetching-sparse-fieldsets)
  - [Pagination](https://jsonapi.org/format/#fetching-pagination)
  - [Filtering](https://jsonapi.org/format/#fetching-filtering)
- `My final goal to this project` in `README.md`
- Basic actix_web support (see `examples/mock_gen.rs` for more details)
- An actix-web based web server
  - A example project showing the basic features
- A lot more tests
- A Rule System ready for the different operations when the jsonapi version changes

### Changed

- Mark the `impl<T: Entity> Entity for Vec<T>` as `unstable-vec-to-document`
- Change `to_document` to longer `to_document_automatically` to encourage users using their own version, rather than the auto version for better performance
- Split `Entity` into `Entity` and `SingleEntity`, where:
  - `SingleEntity` has all of the params like `ty()`, `id()` where only the single entity will have
  - `Entity` defines the more general functions like `included` and `to_document_automatically`, where `Vec<T>` or `[T]` will also have. But of course, they have no the param like `id`
- Error System now is much better now

## [0.1.1] - 2019-11-02

### Fixed

- [Wildcard (`*`) dependency constraints are not allowed on crates.io](https://doc.rust-lang.org/cargo/faq.html#can-libraries-use--as-a-version-for-their-dependencies)

## [0.1.0] - 2019-11-02

### Added

- JSON:API Data Structure
- A user-friendly macro system and **Entity System** (`rabbithole::entity::Entity`)
- Basic tests
- TravisCI support
- README/CHANGELOG docs
