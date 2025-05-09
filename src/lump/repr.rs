use crate::error;
use crate::lump::kind;
use crate::slice_to_cstring;
use crate::Palette;
use std::boxed::Box;
use std::ffi::{CString, IntoStringError};
use std::mem::size_of;
use std::string::{String, ToString};

/// Enum w/ variants for each known lump kind
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Lump {
    Palette(Box<Palette>),
    StatusBar(Image),
    MipTexture(MipTexture),
    Flat(Box<[u8]>),
}

impl Lump {
    /// Single-byte lump kind identifier as used in WAD entries
    pub fn kind(&self) -> u8 {
        match self {
            Self::Palette(_) => kind::PALETTE,
            Self::StatusBar(_) => kind::SBAR,
            Self::MipTexture(_) => kind::MIPTEX,
            _ => kind::FLAT,
        }
    }
}

/// Image stored as palette indices (0..256) in row-major order
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Box<[u8]>,
}

impl Image {
    /// Create an image from a width and list of pixels.  Height is calculated
    /// from number of pixels and width, or 0 if pixels are empty.
    ///
    /// # Panics
    ///
    /// Panics if pixel count does not fit within a `u32`, pixels cannot fit
    /// within an integer number of row, or there are a non-zero number of
    /// pixels and width is 0.
    pub fn from_pixels(width: u32, pixels: Box<[u8]>) -> Self {
        let pixel_ct: u32 = pixels.len().try_into().expect("Too many pixels");

        if pixels.is_empty() {
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

    /// Slice of all the pixels
    pub fn pixels(&self) -> &[u8] {
        &self.pixels[..]
    }
}

/// Mip-mapped texture.  Contains exactly 4 mips (including the full resolution
/// image).
///
/// Textures mips are guaranteed to be valid, meaning that width and height of
/// a mip is half that of the width and height of the previous mip.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MipTexture {
    name: [u8; 16],
    mips: [Image; 4],
}

impl MipTexture {
    pub const MIP_COUNT: usize = 4;

    /// Assemble a texture from provided mips with the given name.  Useful if
    /// name is already given by a WAD entry as a block of 16 bytes.
    ///
    /// # Panic
    ///
    /// Will panic if mips are not valid.
    pub fn from_parts(name: [u8; 16], mips: [Image; Self::MIP_COUNT]) -> Self {
        Self::validate_mips(&mips);
        MipTexture { name, mips }
    }

    /// Assemble a texture from provided mips with `name` converted to a block
    /// of 16 bytes.
    ///
    /// @ Panic
    ///
    /// Will panic if mips are not valid or `name` does not fit within 16 bytes.
    pub fn new(name: String, mips: [Image; Self::MIP_COUNT]) -> Self {
        let mut name_field = [0u8; 16];
        let name_bytes = &name.into_bytes();
        name_field[..name_bytes.len()].copy_from_slice(name_bytes);
        let name = name_field;
        Self::validate_mips(&mips);

        MipTexture { name, mips }
    }

    fn validate_mips(mips: &[Image; Self::MIP_COUNT]) {
        for l in 0..(Self::MIP_COUNT - 1) {
            let r = l + 1;

            if Some(mips[l].width) != mips[r].width.checked_mul(2) {
                panic!("Bad mipmaps");
            }

            if Some(mips[l].height) != mips[r].height.checked_mul(2) {
                panic!("Bad mipmaps");
            }
        }
    }

    /// Obtain the name as a C string.  If the name is not already
    /// null-terminated (in which case the entry is not well-formed) a null byte
    /// is appended to make a valid C string.
    pub fn name_to_cstring(&self) -> CString {
        slice_to_cstring(&self.name)
    }

    /// Attempt to interpret the name as UTF-8 encoded string
    pub fn name_to_string(&self) -> Result<String, IntoStringError> {
        self.name_to_cstring().into_string()
    }

    pub fn name(&self) -> [u8; 16] {
        self.name
    }

    /// Get the texture mip as an image at the specified index.
    ///
    /// # Panic
    ///
    /// Panics if index is > 3
    pub fn mip(&self, index: usize) -> &Image {
        if index < Self::MIP_COUNT {
            &self.mips[index]
        } else {
            panic!("Outside mip bounds ([0..{}])", Self::MIP_COUNT);
        }
    }

    /// Get the texture mips as a slice of images
    pub fn mips(&self) -> &[Image] {
        &self.mips[..]
    }
}

/// Lump header for mip-mapped textures
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

    /// Obtain header from a block of bytes as found in a miptex WAD lump.
    ///
    /// # Panic
    ///
    /// Will panic if width or height are not each divisible by 8, in which case
    /// valid mips cannot be generated.  Will panic if number of pixels in mip
    /// 0 cannot fit within a `u32`.
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
