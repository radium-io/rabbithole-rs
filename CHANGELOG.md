# Changelog

## [Unreleased]

- `rabbithole::model::query::Query` for 
  - [Inclusion of Related Resources](https://jsonapi.org/format/#fetching-includes)
  - [Sparse Fields Set](https://jsonapi.org/format/#fetching-sparse-fieldsets)
  - [Pagination](https://jsonapi.org/format/#fetching-pagination)
  - [Filtering](https://jsonapi.org/format/#fetching-filtering)

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
