use crate::common::Palette;
use crate::wad::repr::{
    Entry, Head, Image, Lump, MipTexture, MipTextureHead, FLAT_LUMP_ID,
    MIPTEX_LUMP_ID, PAL_LUMP_ID, SBAR_LUMP_ID,
};
use crate::wad::{ReadError, ReadResult};
use std::boxed::Box;
use std::io::{Read, Seek, SeekFrom};
use std::mem::{size_of, size_of_val, MaybeUninit};
use std::string::{String, ToString};
use std::vec::Vec;

pub fn parse_directory(mut cursor: impl Seek + Read) -> ReadResult<Vec<Entry>> {
    cursor.rewind()?;
    let mut header_bytes = [0u8; size_of::<Head>()];
    cursor.read_exact(&mut header_bytes[..])?;
    let header: Head = header_bytes.try_into().map_err(ReadError::Parse)?;
    let entry_ct = header.entry_count();
    let dir_offset = header.directory_offset();
    cursor.seek(SeekFrom::Start(dir_offset.into()))?;

    let mut entries = Vec::<Entry>::with_capacity(entry_ct.try_into().unwrap());

    for _ in 0..entry_ct {
        const WAD_ENTRY_SIZE: usize = size_of::<Entry>();
        let mut entry_bytes = [0u8; WAD_ENTRY_SIZE];
        cursor.read_exact(&mut entry_bytes[0..WAD_ENTRY_SIZE])?;
        let entry: Entry = entry_bytes.try_into().map_err(ReadError::Parse)?;
        entries.push(entry);
    }

    Ok(entries)
}

pub fn parse_lump(
    entry: &Entry,
    mut cursor: impl Seek + Read,
) -> Result<Lump, String> {
    const CONCHARS: [u8; 9] = *b"CONCHARS\0";

    let byte_ct = entry.length().try_into().unwrap();

    cursor
        .seek(SeekFrom::Start(entry.offset().into()))
        .map_err(|e| e.to_string())?;

    let bytes = cursor
        .bytes()
        .take(byte_ct)
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| e.to_string())?;

    if bytes.len() != byte_ct {
        return Err("Reached EOF before end of lump".to_string());
    }

    let mut kind = entry.lump_kind();

    // Stupid stupid stupid...
    if &entry.name()[..size_of_val(&CONCHARS)] == &CONCHARS[..] {
        kind = FLAT_LUMP_ID;
    }

    match kind {
        PAL_LUMP_ID => Ok(Lump::Palette(parse_palette_lump(&bytes)?)),
        SBAR_LUMP_ID => Ok(Lump::StatusBar(parse_image_lump(&bytes)?)),
        MIPTEX_LUMP_ID => Ok(Lump::MipTexture(parse_mip_texture_lump(&bytes)?)),
        FLAT_LUMP_ID => Ok(Lump::Flat(Box::from(bytes))),
        b => Err(format!("Unknown lump type `{}`", char::from(b))),
    }
}

fn parse_mip_texture_lump(bytes: &[u8]) -> Result<MipTexture, String> {
    const LUMP_SIZE: usize = size_of::<MipTextureHead>();
    let head_bytes = <[u8; LUMP_SIZE]>::try_from(&bytes[..LUMP_SIZE])
        .map_err(|e| e.to_string())?;
    let head: MipTextureHead = head_bytes.try_into()?;
    let mip0_length = head.width as usize * head.height as usize;
    const UNINIT_IMAGE: MaybeUninit<Image> = MaybeUninit::uninit();
    let mut mips = [UNINIT_IMAGE; 4];

    for i in 0..4 {
        let pix_start = head.offsets[i] as usize;
        let length = mip0_length >> (i * 2);
        let pix_end = pix_start + length as usize;
        let pixels: Box<[u8]> = (&bytes[pix_start..pix_end]).into();
        mips[i].write(Image::from_pixels(head.width >> i, pixels.into()));
    }

    Ok(MipTexture::new(unsafe {
        mips.map(|elem| elem.assume_init())
    })?)
}

fn parse_palette_lump(bytes: &[u8]) -> Result<Box<Palette>, String> {
    if bytes.len() != size_of::<Palette>() {
        return Err(format!(
            "Palette must be {} bytes long",
            size_of::<Palette>()
        ));
    }

    let mut palette: Box<Palette> = Box::from([[0u8; 3]; 256]);
    let mut idx = 0;

    for color in &mut palette.iter_mut() {
        for channel in &mut color.iter_mut() {
            *channel = bytes[idx];
            idx += 1;
        }
    }

    Ok(palette)
}

fn parse_image_lump(bytes: &[u8]) -> Result<Image, String> {
    let width = u32::from_le_bytes(
        <[u8; 4]>::try_from(&bytes[0..4]).map_err(|e| e.to_string())?,
    );

    let height = u32::from_le_bytes(
        <[u8; 4]>::try_from(&bytes[4..8]).map_err(|e| e.to_string())?,
    );

    let pixel_ct = width
        .checked_mul(height)
        .ok_or("Image too large".to_string())?;

    let pixel_ct = usize::try_from(pixel_ct).map_err(|e| e.to_string())?;
    let pixel_end = pixel_ct + 8;

    if pixel_end != bytes.len() {
        return Err(
            "Status bar image size does not match width * height".to_string()
        );
    }

    Ok(Image::from_pixels(
        width,
        Box::<[u8]>::try_from(&bytes[8..pixel_end])
            .map_err(|e| e.to_string())?,
    ))
}
