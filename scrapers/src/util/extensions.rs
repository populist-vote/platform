pub trait NoneIfEmptyExt: Sized {
    fn none_if_empty(self) -> Option<Self>;
}

impl NoneIfEmptyExt for String {
    fn none_if_empty(self) -> Option<Self> {
        if self.is_empty() {
            return None;
        }
        Some(self)
    }
}

impl<T> NoneIfEmptyExt for Vec<T> {
    fn none_if_empty(self) -> Option<Self> {
        if self.is_empty() {
            return None;
        }
        Some(self)
    }
}
