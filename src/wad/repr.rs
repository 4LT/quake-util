use std::boxed::Box;
use std::ffi::CString;
use std::mem::size_of;
use std::string::{String, ToString};

use crate::{lump, slice_to_cstring, Junk};

pub const MAGIC: [u8; 4] = *b"WAD2";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C, packed)]
pub struct Head {
    magic: [u8; 4],
    entry_count: u32,
    directory_offset: u32,
}

impl Head {
    pub fn new(entry_count: u32, directory_offset: u32) -> Self {
        Head {
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

impl TryFrom<[u8; size_of::<Head>()]> for Head {
    type Error = String;

    fn try_from(bytes: [u8; size_of::<Head>()]) -> Result<Self, Self::Error> {
        let mut chunks = bytes.chunks_exact(4usize);

        if chunks.next().unwrap() != &MAGIC[..] {
            let magic_str: String =
                MAGIC.iter().copied().map(char::from).collect();

            return Err(format!("Magic number does not match `{magic_str}`"));
        }

        let entry_count = u32::from_le_bytes(
            <[u8; 4]>::try_from(chunks.next().unwrap())
                .map_err(|e| e.to_string())?,
        );

        let directory_offset = u32::from_le_bytes(
            <[u8; 4]>::try_from(chunks.next().unwrap())
                .map_err(|e| e.to_string())?,
        );

        Ok(Head::new(entry_count, directory_offset))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C, packed)]
pub struct Entry {
    offset: u32,
    length: u32,
    uncompressed_length: u32, // unused?
    lump_kind: u8,
    compression: u8, // 0 - uncompressed, other values unused?
    _padding: Junk<u16>,
    name: [u8; 16],
}

impl Entry {
    pub fn new(config: EntryConfig) -> Entry {
        Entry {
            offset: config.offset,
            length: config.length,
            uncompressed_length: config.length,
            lump_kind: config.lump_kind,
            compression: 0u8,
            _padding: Junk::default(),
            name: config.name,
        }
    }

    pub fn name_as_cstring(&self) -> CString {
        slice_to_cstring(&self.name)
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

    pub fn kind(&self) -> u8 {
        self.lump_kind
    }
}

impl TryFrom<[u8; size_of::<Entry>()]> for Entry {
    type Error = String;

    fn try_from(bytes: [u8; size_of::<Entry>()]) -> Result<Self, Self::Error> {
        let (offset_bytes, rest) = bytes.split_at(4);

        let offset =
            u32::from_le_bytes(<[u8; 4]>::try_from(offset_bytes).unwrap());

        let (length_bytes, rest) = rest.split_at(4);

        let length =
            u32::from_le_bytes(<[u8; 4]>::try_from(length_bytes).unwrap());

        let (uc_length_bytes, rest) = rest.split_at(4);

        let _uc_length =
            u32::from_le_bytes(<[u8; 4]>::try_from(uc_length_bytes).unwrap());

        let (&[lump_kind], rest) = rest.split_at(1) else {
            unreachable!()
        };

        let (&[compression], rest) = rest.split_at(1) else {
            unreachable!()
        };

        if compression != 0 {
            return Err("Compression is unsupported".to_string());
        }

        if !expected_lump_kind(lump_kind) {
            return Err(format!("Unexpected lump type `{lump_kind}`"));
        }

        let name: [u8; 16] = rest[2..].try_into().unwrap();

        Ok(Entry::new(EntryConfig {
            offset,
            length,
            lump_kind,
            name,
        }))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct EntryConfig {
    offset: u32,
    length: u32,
    lump_kind: u8,
    name: [u8; 16],
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
        let name = <[u8; 16]>::try_from(&bytes[..16]).unwrap();

        let bytes = &bytes[16..];

        let width =
            u32::from_le_bytes(<[u8; 4]>::try_from(&bytes[..4]).unwrap());

        let bytes = &bytes[4..];

        let height =
            u32::from_le_bytes(<[u8; 4]>::try_from(&bytes[..4]).unwrap());

        if width % 8 != 0 {
            return Err(format!("Invalid width {}", width));
        }

        if height % 8 != 0 {
            return Err(format!("Invalid height {}", height));
        }

        width
            .checked_mul(height)
            .ok_or("Texture too large".to_string())?;

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

fn expected_lump_kind(lump_kind: u8) -> bool {
    [
        lump::kind::PALETTE,
        lump::kind::SBAR,
        lump::kind::MIPTEX,
        lump::kind::FLAT,
    ]
    .contains(&lump_kind)
}
