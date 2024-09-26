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

pub trait AsOptStrExt {
    fn as_str(&self) -> Option<&str>;
}

impl AsOptStrExt for Option<&String> {
    fn as_str(&self) -> Option<&str> {
        self.map(String::as_str)
    }
}

impl AsOptStrExt for Option<String> {
    fn as_str(&self) -> Option<&str> {
        self.as_ref().map(String::as_str)
    }
}

pub trait AsStrUnwrappedOrEmptyExt: AsOptStrExt {
    fn as_str_unwrapped_or_empty(&self) -> &str;
}

impl AsStrUnwrappedOrEmptyExt for Option<&String> {
    fn as_str_unwrapped_or_empty(&self) -> &str {
        self.as_str().unwrap_or_default()
    }
}

impl AsStrUnwrappedOrEmptyExt for Option<String> {
    fn as_str_unwrapped_or_empty(&self) -> &str {
        self.as_str().unwrap_or_default()
    }
}
