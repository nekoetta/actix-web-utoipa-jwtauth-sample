use validator::Validate;

pub trait IntoValidator<T: Validate> {
    fn validator(&self) -> T;
}
