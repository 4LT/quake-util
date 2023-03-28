pub type Palette = [[u8; 3]; 256];

pub const QUAKE_PALETTE: Palette =
    unsafe { std::mem::transmute(*include_bytes!("palette.lmp")) };

#[derive(Debug, Clone, Copy)]
pub struct Junk<T: Copy + Default + Sized> {
    _value: T,
}

impl<T: Copy + Default> PartialEq<Junk<T>> for Junk<T> {
    fn eq(&self, _: &Junk<T>) -> bool {
        true
    }
}

impl<T: Copy + Default> Eq for Junk<T> {}

impl<T: Copy + Default> Default for Junk<T> {
    fn default() -> Self {
        Self {
            _value: T::default(),
        }
    }
}
