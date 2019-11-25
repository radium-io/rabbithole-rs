#[derive(Error, Debug)]
pub enum EntityDecoratorError {
    #[error(
        "`EntityDecorator` macro can only be used on Named Structs with `id` decorator, just like \
         `#[entity(type = \"foo_type\")]`"
    )]
    InvalidEntityType,
    #[error(
        "`EntityDecorator` needs a Data Service to handle the CRUD operations, using \
         `#[entity(service(FooService))]` to register one"
    )]
    LackOfService,
    #[error("Duplicated Id fields detected")]
    DuplicatedId,
    #[error("Invalid unit decorator {0}, the valid ones: [id, to_one, to_many]")]
    InvalidUnitDecorator(String),
    #[error("Field without name")]
    FieldWithoutName,
}
