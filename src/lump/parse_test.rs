use super::{
    parse_image, parse_mip_texture, parse_palette, read_raw, MipTextureHead,
};
use crate::error;
use std::io::Cursor;
use std::mem::size_of;
use std::vec::Vec;

fn good_miptex_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut offset: u32 = size_of::<MipTextureHead>().try_into().unwrap();
    bytes.extend(b"namenamenamenam\0");
    bytes.extend((16_u32).to_le_bytes());
    bytes.extend((16_u32).to_le_bytes());
    bytes.extend((offset).to_le_bytes());
    offset += 256;
    bytes.extend((offset).to_le_bytes());
    offset += 64;
    bytes.extend((offset).to_le_bytes());
    offset += 16;
    bytes.extend((offset).to_le_bytes());

    bytes.extend([0u8; 256]);
    bytes.extend([0u8; 64]);
    bytes.extend([0u8; 16]);
    bytes.extend([0u8; 4]);

    bytes
}

#[test]
fn parse_good_mip_texture() {
    let bytes = good_miptex_bytes();
    let mut cursor = Cursor::new(bytes);
    let miptex = parse_mip_texture(&mut cursor).unwrap();
    assert_eq!(miptex.mip(0).width(), 16);
    assert_eq!(miptex.mip(0).height(), 16);
}

#[test]
fn parse_mip_texture_with_bad_head() {
    let mut bytes = good_miptex_bytes();
    bytes[16..20].copy_from_slice(&(13u32).to_le_bytes());
    let mut cursor = Cursor::new(bytes);
    let e = parse_mip_texture(&mut cursor).unwrap_err();
    assert!(matches!(e, error::BinParse::Parse(_)));
}

#[test]
fn parse_mip_texture_short_head() {
    let bytes = &good_miptex_bytes()[..27];
    let mut cursor = Cursor::new(bytes);
    let e = parse_mip_texture(&mut cursor).unwrap_err();
    assert!(matches!(e, error::BinParse::Io(_)));
}

#[test]
fn parse_mip_texture_with_bad_offset() {
    let mut bytes = good_miptex_bytes();
    bytes[24..28].copy_from_slice(&(360u32).to_le_bytes());
    let mut cursor = Cursor::new(bytes);
    let e = parse_mip_texture(&mut cursor).unwrap_err();
    assert!(matches!(e, error::BinParse::Io(_)));
}

#[test]
fn parse_good_palette() {
    let bytes = [0u8; 768];
    let pal = parse_palette(&mut &bytes[..]).unwrap();
    assert_eq!(pal.len(), 256);
}

#[test]
fn parse_bad_palette() {
    let bytes = [0u8; 3];
    let e = parse_palette(&mut &bytes[..]).unwrap_err();
    assert!(matches!(e, error::BinParse::Io(_)));
}

#[test]
fn parse_good_image() {
    let mut bytes = Vec::<u8>::new();
    bytes.extend((48u32).to_le_bytes());
    bytes.extend((32u32).to_le_bytes());
    bytes.extend([0u8; 32 * 48]);
    let image = parse_image(&mut &bytes[..]).unwrap();
    assert_eq!(image.width(), 48);
    assert_eq!(image.height(), 32);
    assert_eq!(image.pixels().len(), 48 * 32);
}

#[test]
fn parse_image_too_large() {
    let mut bytes = Vec::<u8>::new();
    bytes.extend((65_536_u32).to_le_bytes());
    bytes.extend((65_536_u32).to_le_bytes());
    let e = parse_image(&mut &bytes[..]).unwrap_err();
    assert!(matches!(e, error::BinParse::Parse(_)));
}

#[test]
fn parse_image_cutoff() {
    let mut bytes = Vec::<u8>::new();
    bytes.extend((256u32).to_le_bytes());
    bytes.extend((256u32).to_le_bytes());
    bytes.extend((65_535_u32).to_le_bytes());
    let e = parse_image(&mut &bytes[..]).unwrap_err();
    assert!(matches!(e, error::BinParse::Io(_)));
}

#[test]
fn read_raw_success() {
    let bytes = [0u8; 123];
    let flat = read_raw(&mut &bytes[..], 123).unwrap();
    assert_eq!(flat.len(), 123);
}

#[test]
fn read_raw_io_error() {
    let bytes = [0u8; 123];
    let e = read_raw(&mut &bytes[..], 209).unwrap_err();
    assert!(matches!(e, error::BinParse::Io(_)));
}
