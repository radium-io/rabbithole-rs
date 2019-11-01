#[derive(Error, Debug)]
pub enum EntityDecoratorError {
    #[error(
        "`to_one` decorator can only be used on `Option<T>`, `Box<T>`, `Option<Box<T>>` or bare \
         T, you can use `#[entity(to_one(T))]` to define the type manually"
    )]
    InvalidToOneType,
    #[error(
        "`to_many` decorator can only be used on `Vec<T>` or `HashSet<T>`, you can use \
         `#[entity(to_many(T))]` to define the type manually"
    )]
    InvalidToManyType,
    #[error("`id` decorator can only be used on the type with `ToString` trait")]
    InvalidIdType,
    #[error(
        "`EntityDecorator` macro can only be used on Named Structs with `id` decorator, just like \
         `#[entity(type = \"foo_type\")]`"
    )]
    InvalidEntityType,
    #[error("Duplicated Id fields detected")]
    DuplicatedId,
    #[error("Invalid unit decorator {0}, the valid ones: [id, to_one, to_many]")]
    InvalidUnitDecorator(String),
    #[error("Invalid parameterized decorator {0}, the valid ones: [to_one(T), to_many(T)]")]
    InvalidParamDecorator(String),
    #[error("Field without name")]
    FieldWithoutName,
}
