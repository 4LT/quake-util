use crate::error;
use crate::lump::kind;
use crate::Palette;
use error::BinParseResult;
use std::boxed::Box;
use std::mem::size_of;
use std::string::ToString;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Lump {
    Palette(Box<Palette>),
    StatusBar(Image),
    MipTexture(MipTexture),
    Flat(Box<[u8]>),
    Unknown(Box<[u8]>),
}

impl Lump {
    pub fn kind(&self) -> Option<u8> {
        match self {
            Self::Palette(_) => Some(kind::PALETTE),
            Self::StatusBar(_) => Some(kind::SBAR),
            Self::MipTexture(_) => Some(kind::MIPTEX),
            Self::Flat(_) => Some(kind::FLAT),
            _ => None,
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

        if pixel_ct % width != 0 {
            panic!("Pixel count != width * height");
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
    mips: [Image; 4],
}

impl MipTexture {
    pub const LEN: usize = 4;

    pub fn new(mips: [Image; Self::LEN]) -> BinParseResult<Self> {
        for l in 0..(Self::LEN - 1) {
            let r = l + 1;
            let err = || Err(error::BinParse::Parse("Bad mipmaps".to_string()));

            if Some(mips[l].width) != mips[r].width.checked_mul(2) {
                return err();
            }

            if Some(mips[l].height) != mips[r].height.checked_mul(2) {
                return err();
            }
        }

        Ok(MipTexture { mips })
    }

    pub fn mip(&self, index: usize) -> &Image {
        if index < Self::LEN {
            &self.mips[index]
        } else {
            panic!("Outside mip bounds ([0..{}])", Self::LEN);
        }
    }
}

impl<'a> IntoIterator for &'a MipTexture {
    type Item = &'a Image;
    type IntoIter = std::slice::Iter<'a, Image>;

    fn into_iter(self) -> Self::IntoIter {
        self.mips[..].iter()
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
