use std::boxed::Box;
use std::ffi::CString;
use std::mem::{size_of, size_of_val, transmute};
use std::string::{String, ToString};
use std::vec::Vec;

use crate::common::{Junk, Palette};

pub const MAGIC: [u8; 4] = *b"WAD2";
pub const PAL_LUMP_ID: u8 = 0x40;
pub const SBAR_LUMP_ID: u8 = 0x42;
pub const MIPTEX_LUMP_ID: u8 = 0x44;
pub const FLAT_LUMP_ID: u8 = 0x45;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct WadHead {
    magic: [u8; 4],
    entry_count: u32,
    directory_offset: u32,
}

impl WadHead {
    pub fn new(entry_count: u32, directory_offset: u32) -> Self {
        WadHead {
            magic: MAGIC,
            entry_count,
            directory_offset,
        }
    }

    pub fn entry_count(&self) -> u32 {
        self.entry_count
    }

    pub fn directory_offset(&self) -> u32 {
        self.directory_offset
    }
}

impl TryFrom<[u8; size_of::<WadHead>()]> for WadHead {
    type Error = String;

    fn try_from(
        bytes: [u8; size_of::<WadHead>()],
    ) -> Result<Self, Self::Error> {
        if &bytes[0..4] != &MAGIC[..] {
            let magic_str: String =
                MAGIC.iter().copied().map(char::from).collect();

            return Err(format!("Magic number does not match `{magic_str}`"));
        }

        let mut header: WadHead = unsafe { transmute(bytes) };

        header.entry_count = header.entry_count.to_le();
        header.directory_offset = header.directory_offset.to_le();

        Ok(header)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct WadEntry {
    offset: u32,
    length: u32,
    uncompressed_length: u32, // unused?
    lump_kind: u8,
    compression: u8, // 0 - uncompressed, other values unused?
    _padding: Junk<u16>,
    name: [u8; 16],
}

impl WadEntry {
    pub fn new(config: WadEntryConfig) -> Result<WadEntry, String> {
        let name_sz = config.name;
        let name_bytes = name_sz.as_bytes();

        let name_bytes = if name_bytes.len() <= 16 {
            name_bytes
        } else {
            return Err(format!("Lump name `{:#?}` too long", name_sz));
        };

        let mut name = [0u8; 16];
        (&mut name[..name_bytes.len()]).copy_from_slice(name_bytes);

        Ok(WadEntry {
            offset: config.offset,
            length: config.length,
            uncompressed_length: config.length,
            lump_kind: config.lump_kind,
            compression: 0u8,
            _padding: Junk::default(),
            name,
        })
    }

    pub fn name_as_cstring(&self) -> CString {
        let mut len = 0;

        while len < size_of_val(&self.name) {
            if self.name[len] == 0u8 {
                break;
            }

            len += 1;
        }

        CString::new(&self.name[0..len]).unwrap()
    }

    pub fn name(&self) -> [u8; 16] {
        self.name
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn lump_kind(&self) -> u8 {
        self.lump_kind
    }
}

impl TryFrom<[u8; size_of::<WadEntry>()]> for WadEntry {
    type Error = String;

    fn try_from(
        bytes: [u8; size_of::<WadEntry>()],
    ) -> Result<Self, Self::Error> {
        let mut entry: WadEntry = unsafe { transmute(bytes) };

        entry.offset = entry.offset.to_le();
        entry.length = entry.length.to_le();
        entry.uncompressed_length = entry.uncompressed_length.to_le();

        if entry.compression != 0u8 {
            return Err("Compression is unsupported".to_string());
        }

        let lump_kind = entry.lump_kind;

        if !expected_lump_kind(lump_kind) {
            return Err(format!("Unexpected lump type `{lump_kind}`"));
        }

        Ok(entry)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct WadEntryConfig {
    offset: u32,
    length: u32,
    lump_kind: u8,
    name: CString,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Wad {
    pub lumps: Vec<(CString, Lump)>,
}

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
            Self::Palette(_) => PAL_LUMP_ID,
            Self::StatusBar(_) => PAL_LUMP_ID,
            Self::MipTexture(_) => PAL_LUMP_ID,
            Self::Flat(_) => PAL_LUMP_ID,
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
            pixels: pixels.into(),
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

    pub fn new(mips: [Image; Self::LEN]) -> Result<Self, String> {
        for l in 0..(Self::LEN - 1) {
            let r = l + 1;

            if Some(mips[l].width) != mips[r].width.checked_mul(2) {
                return Err("Bad mipmaps".to_string());
            }

            if Some(mips[l].height) != mips[r].height.checked_mul(2) {
                return Err("Bad mipmaps".to_string());
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
        (&self.mips[..]).into_iter()
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
    type Error = String;

    fn try_from(
        bytes: [u8; size_of::<MipTextureHead>()],
    ) -> Result<Self, Self::Error> {
        let mut head: MipTextureHead = unsafe { transmute(bytes) };

        head.width = head.width.to_le();
        let width = head.width;

        if width % 8 != 0 {
            return Err(format!("Invalid width {}", width));
        }

        head.height = head.height.to_le();
        let height = head.height;

        if height % 8 != 0 {
            return Err(format!("Invalid height {}", height));
        }

        width
            .checked_mul(height)
            .ok_or("Texture too large".to_string())?;

        for i in 0..4 {
            head.offsets[i] = head.offsets[i].to_le();
        }

        Ok(head)
    }
}

fn expected_lump_kind(lump_kind: u8) -> bool {
    [PAL_LUMP_ID, SBAR_LUMP_ID, MIPTEX_LUMP_ID, FLAT_LUMP_ID]
        .contains(&lump_kind)
}
