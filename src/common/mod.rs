use std::ffi::CString;

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

pub fn slice_to_cstring(slice: &[u8]) -> std::ffi::CString {
    let mut len = 0;

    while len < slice.len() {
        if slice[len] == 0u8 {
            break;
        }

        len += 1;
    }

    CString::new(&slice[..len]).unwrap()
}
