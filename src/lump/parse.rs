use crate::common::Palette;
use crate::error;
use crate::lump::{Image, MipTexture, MipTextureHead};
use crate::wad;
use error::BinParseResult;
use std::boxed::Box;
use std::io::{Read, Seek, SeekFrom};
use std::mem::{size_of, transmute, MaybeUninit};
use std::string::ToString;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ParseInferenceInfo<'a> {
    None,
    Entry(&'a wad::Entry),
    Length(u32),
}

impl ParseInferenceInfo<'_> {
    pub fn length(&self) -> Option<u32> {
        match self {
            ParseInferenceInfo::None => None,
            ParseInferenceInfo::Entry(entry) => Some(entry.length()),
            ParseInferenceInfo::Length(length) => Some(*length),
        }
    }
}

pub fn parse_mip_texture(
    cursor: &mut (impl Seek + Read),
) -> BinParseResult<MipTexture> {
    const LUMP_SIZE: usize = size_of::<MipTextureHead>();
    let mut head_bytes = [0u8; LUMP_SIZE];
    let lump_start = cursor.stream_position()?;

    cursor.read_exact(&mut head_bytes)?;

    let head: MipTextureHead = head_bytes.try_into()?;
    let mip0_length = u64::from(head.width) * u64::from(head.height);
    const UNINIT_IMAGE: MaybeUninit<Image> = MaybeUninit::uninit();
    let mut mips = [UNINIT_IMAGE; 4];

    for i in 0u32..4u32 {
        let pix_start: u64 = head.offsets[i as usize].into();
        let length: usize = (mip0_length >> (i * 2)).try_into().unwrap();

        cursor.seek(SeekFrom::Start(
            lump_start
                .checked_add(pix_start)
                .ok_or(error::BinParse::Parse("Bad offset".to_string()))?,
        ))?;

        let mut pixels = vec![0u8; length].into_boxed_slice();
        cursor.read_exact(&mut pixels)?;

        mips[i as usize].write(Image::from_pixels(head.width >> i, pixels));
    }

    MipTexture::new(unsafe { mips.map(|elem| elem.assume_init()) })
}

pub fn parse_palette(reader: &mut impl Read) -> BinParseResult<Box<Palette>> {
    let mut bytes = [0u8; size_of::<Palette>()];
    reader.read_exact(&mut bytes[..])?;
    Ok(Box::from(unsafe { transmute::<_, Palette>(bytes) }))
}

pub fn parse_image(reader: &mut impl Read) -> BinParseResult<Image> {
    let mut u32_buf = [0u8; size_of::<u32>()];
    reader.read_exact(&mut u32_buf[..])?;
    let width = u32::from_le_bytes(u32_buf);
    reader.read_exact(&mut u32_buf[..])?;
    let height = u32::from_le_bytes(u32_buf);

    let pixel_ct = width
        .checked_mul(height)
        .ok_or(error::BinParse::Parse("Image too large".to_string()))?;

    let mut pixels = vec![0u8; pixel_ct as usize].into_boxed_slice();
    reader.read_exact(&mut pixels)?;

    Ok(Image::from_pixels(width, pixels))
}

pub fn read_raw(
    cursor: &mut (impl Seek + Read),
    length: usize,
) -> BinParseResult<Box<[u8]>> {
    let mut bytes = vec![0u8; length].into_boxed_slice();
    cursor.read_exact(&mut bytes)?;
    Ok(bytes)
}
