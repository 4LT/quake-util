use core::cell::Cell;

pub trait CellOptionExt<T> {
    fn steal(&self) -> T;

    fn into_unwrapped(self) -> T;
}

impl<T> CellOptionExt<T> for Cell<Option<T>> {
    fn steal(&self) -> T {
        self.take().expect("Empty cell option")
    }

    fn into_unwrapped(self) -> T {
        self.into_inner().expect("Empty cell option")
    }
}
