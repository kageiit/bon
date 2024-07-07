use std::mem::MaybeUninit;

/// [`MaybeUninit`] is used to make the memory layout of this struct be equal
/// to `T` such that the compiler may optimize away moving data between it and
/// [`Set<T>`].
struct Unset<T>(MaybeUninit<T>);

impl<T> Default for Unset<T> {
    fn default() -> Self {
        Self(MaybeUninit::uninit())
    }
}

pub struct Required<T>(Unset<Option<T>>);

impl<T> Default for Required<T> {
    fn default() -> Self {
        Self(Unset::default())
    }
}

pub struct Optional<T>(Unset<T>);

impl<T> Default for Optional<T> {
    fn default() -> Self {
        Self(Unset::default())
    }
}

impl<T: Default> From<Optional<T>> for Set<Option<T>> {
    fn from(_: Optional<T>) -> Self {
        Set::new(None)
    }
}

#[repr(transparent)]
pub struct Set<T>(T);

impl<T> Set<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}
