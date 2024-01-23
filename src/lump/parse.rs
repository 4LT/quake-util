use crate::common::Palette;
use crate::lump::{kind, Image, Lump, MipTexture, MipTextureHead};
use crate::wad;
use std::boxed::Box;
use std::io::{Read, Seek, SeekFrom};
use std::mem::{size_of, size_of_val, transmute, MaybeUninit};
use std::string::{String, ToString};

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
) -> Result<MipTexture, String> {
    const LUMP_SIZE: usize = size_of::<MipTextureHead>();
    let mut head_bytes = [0u8; LUMP_SIZE];
    let lump_start = cursor.stream_position().map_err(|e| e.to_string())?;

    cursor
        .read_exact(&mut head_bytes)
        .map_err(|e| e.to_string())?;

    let head: MipTextureHead = head_bytes.try_into()?;
    let mip0_length = head.width as u64 * head.height as u64;
    const UNINIT_IMAGE: MaybeUninit<Image> = MaybeUninit::uninit();
    let mut mips = [UNINIT_IMAGE; 4];

    for i in 0u32..4u32 {
        let pix_start = head.offsets[i as usize] as u64;
        let length = mip0_length >> (i * 2);

        cursor
            .seek(SeekFrom::Start(
                lump_start
                    .checked_add(pix_start)
                    .ok_or("Bad offset".to_string())?,
            ))
            .map_err(|e| e.to_string())?;

        let mut pixels = vec![0u8; length as usize].into_boxed_slice();
        cursor.read_exact(&mut pixels).map_err(|e| e.to_string())?;

        mips[i as usize].write(Image::from_pixels(head.width >> i, pixels));
    }

    MipTexture::new(unsafe { mips.map(|elem| elem.assume_init()) })
}

pub fn parse_palette(reader: &mut impl Read) -> Result<Box<Palette>, String> {
    let mut bytes = [0u8; size_of::<Palette>()];
    reader
        .read_exact(&mut bytes[..])
        .map_err(|e| e.to_string())?;
    Ok(Box::from(unsafe { transmute::<_, Palette>(bytes) }))
}

pub fn parse_image(reader: &mut impl Read) -> Result<Image, String> {
    let mut u32_buf = [0u8; size_of::<u32>()];
    reader
        .read_exact(&mut u32_buf[..])
        .map_err(|e| e.to_string())?;
    let width = u32::from_le_bytes(u32_buf);
    reader
        .read_exact(&mut u32_buf[..])
        .map_err(|e| e.to_string())?;
    let height = u32::from_le_bytes(u32_buf);

    let pixel_ct = width
        .checked_mul(height)
        .ok_or("Image too large".to_string())?;

    let mut pixels = vec![0u8; pixel_ct as usize].into_boxed_slice();
    reader.read_exact(&mut pixels).map_err(|e| e.to_string())?;

    Ok(Image::from_pixels(width, pixels))
}

pub fn parse_inferred(
    cursor: &mut (impl Seek + Read),
    info: ParseInferenceInfo<'_>,
) -> Result<Lump, String> {
    const CONCHARS_NAME: &[u8; 9] = b"CONCHARS\0";

    let mut attempt_order =
        [kind::MIPTEX, kind::SBAR, kind::PALETTE, kind::FLAT];

    // Some paranoid nonsense because not even Id can be trusted to tag their
    // lumps correctly
    let mut prioritize = |first_kind| {
        let mut index = 0;

        for (i, kind) in attempt_order.into_iter().enumerate() {
            if kind == first_kind {
                index = i;
            }
        }

        while index > 0 {
            attempt_order[index] = attempt_order[index - 1];
            attempt_order[index - 1] = first_kind;
            index -= 1;
        }
    };

    if let ParseInferenceInfo::Entry(entry) = info {
        seek_to_entry(cursor, entry)?;

        prioritize(entry.kind());

        // It's *improbable* that a palette-sized lump could be a valid status
        // bar image OR miptex, though it's possibly just 768 rando bytes.  So
        // if the explicit type is FLAT and it's 768 bytes, we can't know for
        // sure that it
        if entry.length() as usize == size_of::<Palette>()
            && entry.kind() != kind::FLAT
        {
            prioritize(kind::PALETTE);
        }

        // Quake's gfx.wad has CONCHARS's type set explicitly to MIPTEX,
        // event though it's a FLAT (128x128 pixels)
        if entry.name()[..size_of_val(CONCHARS_NAME)] == CONCHARS_NAME[..] {
            prioritize(kind::FLAT);
        }
    }

    if let ParseInferenceInfo::Length(length) = info {
        if length as usize == size_of::<Palette>() {
            prioritize(kind::PALETTE);
        }
    }

    let lump_start = cursor.stream_position().map_err(|e| e.to_string())?;

    for attempt_kind in attempt_order {
        match attempt_kind {
            kind::MIPTEX => {
                if let Ok(miptex) = parse_mip_texture(cursor) {
                    return Ok(Lump::MipTexture(miptex));
                }
            }
            kind::SBAR => {
                if let Ok(img) = parse_image(cursor) {
                    return Ok(Lump::StatusBar(img));
                }
            }
            kind::PALETTE => {
                if let Ok(pal) = parse_palette(cursor) {
                    return Ok(Lump::Palette(pal));
                }
            }
            kind::FLAT => {
                let length = match info {
                    ParseInferenceInfo::Entry(entry) => Some(entry.length()),
                    ParseInferenceInfo::Length(length) => Some(length),
                    _ => None,
                };

                if let Some(length) = length {
                    if let Ok(bytes) = read_raw_priv(cursor, length as usize) {
                        return Ok(Lump::Flat(bytes));
                    }
                }
            }
            _ => unreachable!(),
        }

        cursor
            .seek(SeekFrom::Start(lump_start))
            .map_err(|e| e.to_string())?;
    }

    Err("Failed to parse lump".to_string())
}

pub fn seek_to_entry(
    cursor: &mut (impl Seek + Read),
    entry: &wad::Entry,
) -> Result<(), String> {
    cursor
        .seek(SeekFrom::Start(entry.offset().into()))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn read_raw(
    cursor: &mut (impl Seek + Read),
    entry: &wad::Entry,
) -> Result<Box<[u8]>, String> {
    seek_to_entry(cursor, entry)?;
    read_raw_priv(cursor, entry.length() as usize)
}

fn read_raw_priv(
    cursor: &mut (impl Seek + Read),
    length: usize,
) -> Result<Box<[u8]>, String> {
    let offset = cursor.stream_position().map_err(|e| e.to_string())?;
    let mut bytes = vec![0u8; length].into_boxed_slice();

    cursor.read_exact(&mut bytes).map_err(|e| {
        format!("{} (offset = {}, length = {})", e, offset, length,)
    })?;

    Ok(bytes)
}
