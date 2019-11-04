#[macro_export]
macro_rules! from_external_error {
    () => {};
    ($head:path) => {
        impl From<$head> for RabbitholeError {
            fn from(err: $head) -> Self { RabbitholeError::Unhandled(Box::new(err)) }
        }
    };
    ($head:path $(, $tail:path)*) => {
        impl From<$head> for RabbitholeError {
            fn from(err: $head) -> Self { RabbitholeError::Unhandled(Box::new(err)) }
        }
        from_external_error!($($tail),*);
    };
}
