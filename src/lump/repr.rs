use crate::error;
use crate::lump::kind;
use crate::slice_to_cstring;
use crate::Palette;
use std::boxed::Box;
use std::ffi::{CString, IntoStringError};
use std::mem::size_of;
use std::string::{String, ToString};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Lump {
    Palette(Box<Palette>),
    StatusBar(Image),
    MipTexture(MipTexture),
    Flat(Box<[u8]>),
}

impl Lump {
    pub fn kind(&self) -> u8 {
        match self {
            Self::Palette(_) => kind::PALETTE,
            Self::StatusBar(_) => kind::SBAR,
            Self::MipTexture(_) => kind::MIPTEX,
            _ => kind::FLAT,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Box<[u8]>,
}

impl Image {
    pub fn from_pixels(width: u32, pixels: Box<[u8]>) -> Self {
        let pixel_ct: u32 = pixels.len().try_into().expect("Too many pixels");

        if pixels.len() == 0 {
            return Image {
                width: 0,
                height: 0,
                pixels,
            };
        }

        if width == 0 {
            panic!("Image with pixels must have width > 0");
        }

        if pixel_ct % width != 0 {
            panic!("Incomplete pixel row");
        }

        Image {
            width,
            height: pixel_ct / width,
            pixels,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels[..]
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MipTexture {
    name: [u8; 16],
    mips: [Image; 4],
}

impl MipTexture {
    pub const LEN: usize = 4;

    pub fn from_parts(name: [u8; 16], mips: [Image; Self::LEN]) -> Self {
        Self::validate_mips(&mips);
        MipTexture { name, mips }
    }

    pub fn new(name: String, mips: [Image; Self::LEN]) -> Self {
        let mut name_field = [0u8; 16];
        let name_bytes = &name.into_bytes();
        (&mut name_field[..name_bytes.len()]).copy_from_slice(name_bytes);
        let name = name_field;
        Self::validate_mips(&mips);

        MipTexture { name, mips }
    }

    fn validate_mips(mips: &[Image; Self::LEN]) {
        for l in 0..(Self::LEN - 1) {
            let r = l + 1;

            if Some(mips[l].width) != mips[r].width.checked_mul(2) {
                panic!("Bad mipmaps");
            }

            if Some(mips[l].height) != mips[r].height.checked_mul(2) {
                panic!("Bad mipmaps");
            }
        }
    }

    pub fn name_to_cstring(&self) -> CString {
        slice_to_cstring(&self.name)
    }

    pub fn name_to_string(&self) -> Result<String, IntoStringError> {
        self.name_to_cstring().into_string()
    }

    pub fn name(&self) -> [u8; 16] {
        self.name
    }

    pub fn mip(&self, index: usize) -> &Image {
        if index < Self::LEN {
            &self.mips[index]
        } else {
            panic!("Outside mip bounds ([0..{}])", Self::LEN);
        }
    }

    pub fn mips(&self) -> &[Image] {
        &self.mips[..]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C, packed)]
pub struct MipTextureHead {
    pub(crate) name: [u8; 16],
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) offsets: [u32; 4],
}

impl TryFrom<[u8; size_of::<MipTextureHead>()]> for MipTextureHead {
    type Error = error::BinParse;

    fn try_from(
        bytes: [u8; size_of::<MipTextureHead>()],
    ) -> Result<Self, Self::Error> {
        let name = <[u8; 16]>::try_from(&bytes[..16]).unwrap();

        let bytes = &bytes[16..];

        let width =
            u32::from_le_bytes(<[u8; 4]>::try_from(&bytes[..4]).unwrap());

        let bytes = &bytes[4..];

        let height =
            u32::from_le_bytes(<[u8; 4]>::try_from(&bytes[..4]).unwrap());

        if width % 8 != 0 {
            return Err(error::BinParse::Parse(format!(
                "Invalid width {}",
                width
            )));
        }

        if height % 8 != 0 {
            return Err(error::BinParse::Parse(format!(
                "Invalid height {}",
                height
            )));
        }

        width
            .checked_mul(height)
            .ok_or(error::BinParse::Parse("Texture too large".to_string()))?;

        let bytes = &bytes[4..];

        let mut offsets = [0u32; 4];

        for i in 0..4 {
            offsets[i] = u32::from_le_bytes(
                <[u8; 4]>::try_from(&bytes[(4 * i)..(4 * i + 4)]).unwrap(),
            );
        }

        Ok(MipTextureHead {
            name,
            width,
            height,
            offsets,
        })
    }
}
