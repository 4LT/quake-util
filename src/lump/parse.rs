use crate::common::Palette;
use crate::lump::{Image, MipTexture, MipTextureHead};
use crate::wad::Entry;
use std::boxed::Box;
use std::io::{Read, Seek, SeekFrom};
use std::mem::{size_of, MaybeUninit};
use std::string::{String, ToString};
use std::vec::Vec;

pub fn parse_mip_texture(
    mut cursor: impl Seek + Read,
) -> Result<MipTexture, String> {
    cursor.rewind().map_err(|e| e.to_string())?;
    const LUMP_SIZE: usize = size_of::<MipTextureHead>();
    let mut head_bytes = [0u8; LUMP_SIZE];
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
            .seek(SeekFrom::Start(pix_start))
            .map_err(|e| e.to_string())?;
        let mut pixels = Vec::<u8>::new();
        pixels.resize(length as usize, 0u8);
        let mut pixels = pixels.into_boxed_slice();
        cursor.read_exact(&mut pixels).map_err(|e| e.to_string())?;
        mips[i as usize]
            .write(Image::from_pixels(head.width >> i, pixels.into()));
    }

    Ok(MipTexture::new(unsafe {
        mips.map(|elem| elem.assume_init())
    })?)
}

pub fn parse_palette(bytes: &[u8]) -> Result<Box<Palette>, String> {
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

pub fn parse_image(bytes: &[u8]) -> Result<Image, String> {
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

pub fn read_raw(
    mut cursor: impl Seek + Read,
    entry: &Entry,
) -> Result<Vec<u8>, String> {
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

    Ok(bytes)
}
