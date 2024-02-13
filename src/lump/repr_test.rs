use super::kind;
use super::{Image, Lump, MipTexture, MipTextureHead};
use crate::error;
use std::ffi::CString;
use std::mem::size_of;
use std::string::String;

use std::boxed::Box;

#[test]
fn lump_kind() {
    assert_eq!(Lump::Palette(Box::new([[0; 3]; 256])).kind(), kind::PALETTE);
    assert_eq!(
        Lump::StatusBar(Image::from_pixels(0, Box::new([]))).kind(),
        kind::SBAR
    );
    assert_eq!(
        Lump::MipTexture(MipTexture::from_parts(
            [0u8; 16],
            [
                Image::from_pixels(16, Box::new([0u8; 256])),
                Image::from_pixels(8, Box::new([0u8; 64])),
                Image::from_pixels(4, Box::new([0u8; 16])),
                Image::from_pixels(2, Box::new([0u8; 4])),
            ]
        ))
        .kind(),
        kind::MIPTEX
    );
    assert_eq!(Lump::Flat(Box::new([1, 2, 3])).kind(), kind::FLAT);
}

#[test]
fn image_from_pixels() {
    let image = Image::from_pixels(40, Box::new([0u8; 1280]));
    assert_eq!(image.width(), 40);
    assert_eq!(image.height(), 32);
    assert_eq!(image.pixels().len(), 1280);
}

#[test]
#[should_panic]
fn zero_width_image() {
    Image::from_pixels(0, Box::new([0u8; 1]));
}

#[test]
#[should_panic]
fn malformed_image() {
    Image::from_pixels(128, Box::new([0u8; 666]));
}

#[test]
#[should_panic]
fn bad_mip_width() {
    MipTexture::from_parts(
        [0u8; 16],
        [
            Image::from_pixels(16, Box::new([0u8; 256])),
            Image::from_pixels(8, Box::new([0u8; 64])),
            Image::from_pixels(4, Box::new([0u8; 16])),
            Image::from_pixels(8, Box::new([0u8; 16])),
        ],
    );
}

#[test]
#[should_panic]
fn bad_mip_height() {
    MipTexture::from_parts(
        [0u8; 16],
        [
            Image::from_pixels(16, Box::new([0u8; 256])),
            Image::from_pixels(8, Box::new([0u8; 64])),
            Image::from_pixels(4, Box::new([0u8; 16])),
            Image::from_pixels(2, Box::new([0u8; 2])),
        ],
    );
}

#[test]
fn access_mip_levels() {
    let images = [
        Image::from_pixels(256, Box::new([0u8; 32768])),
        Image::from_pixels(128, Box::new([0u8; 8192])),
        Image::from_pixels(64, Box::new([0u8; 2048])),
        Image::from_pixels(32, Box::new([0u8; 512])),
    ];
    let miptex = MipTexture::from_parts([0u8; 16], images.clone());

    for i in 0..4 {
        assert_eq!(miptex.mip(i), &images[i]);
    }
}

#[test]
#[should_panic]
fn access_bad_mip_level() {
    let miptex = MipTexture::from_parts(
        [0u8; 16],
        [
            Image::from_pixels(128, Box::new([0u8; 32768])),
            Image::from_pixels(64, Box::new([0u8; 8192])),
            Image::from_pixels(32, Box::new([0u8; 2048])),
            Image::from_pixels(16, Box::new([0u8; 512])),
        ],
    );
    miptex.mip(4);
}

#[test]
fn miptex_mips() {
    let images = [
        Image::from_pixels(384, Box::new([0u8; 3072 * 16])),
        Image::from_pixels(192, Box::new([0u8; 768 * 16])),
        Image::from_pixels(96, Box::new([0u8; 192 * 16])),
        Image::from_pixels(48, Box::new([0u8; 48 * 16])),
    ];
    let miptex = MipTexture::from_parts([0u8; 16], images.clone());

    for (idx, mip) in miptex.mips().iter().enumerate() {
        assert_eq!(mip, &images[idx]);
    }
}

const NAME: [u8; 16] = *b"SomeOldNameGame\0";

fn good_miptex_head_bytes() -> [u8; size_of::<MipTextureHead>()] {
    let mut bytes = [0u8; size_of::<MipTextureHead>()];
    bytes[..16].copy_from_slice(&NAME);
    bytes[16..20].copy_from_slice(&(384_u32).to_le_bytes());
    bytes[20..24].copy_from_slice(&(192_u32).to_le_bytes());
    bytes[24..28].copy_from_slice(&(1_111_u32).to_le_bytes());
    bytes[28..32].copy_from_slice(&(2_222_u32).to_le_bytes());
    bytes[32..36].copy_from_slice(&(3_333_u32).to_le_bytes());
    bytes[36..].copy_from_slice(&(4_444_u32).to_le_bytes());
    bytes
}

#[test]
fn miptex_head_from_bytes() {
    let bytes = good_miptex_head_bytes();
    let head: MipTextureHead = bytes.try_into().unwrap();
    let name = head.name;
    let width = head.width;
    let height = head.height;
    let offsets = head.offsets;

    assert_eq!(name, NAME);
    assert_eq!(width, 384);
    assert_eq!(height, 192);
    assert_eq!(offsets, [1_111, 2_222, 3_333, 4_444]);
}

#[test]
fn miptex_head_bad_width() {
    let mut bytes = good_miptex_head_bytes();
    bytes[16..20].copy_from_slice(&(69_u32).to_le_bytes());
    let e = <MipTextureHead>::try_from(bytes).unwrap_err();
    assert!(matches!(e, error::BinParse::Parse(_)));
}

#[test]
fn miptex_head_bad_height() {
    let mut bytes = good_miptex_head_bytes();
    bytes[20..24].copy_from_slice(&(60_u32).to_le_bytes());
    let e = <MipTextureHead>::try_from(bytes).unwrap_err();
    assert!(matches!(e, error::BinParse::Parse(_)));
}

#[test]
fn miptex_head_too_large() {
    let mut bytes = good_miptex_head_bytes();
    bytes[16..20].copy_from_slice(&(65_536_u32).to_le_bytes());
    bytes[20..24].copy_from_slice(&(65_536_u32).to_le_bytes());
    let e = <MipTextureHead>::try_from(bytes).unwrap_err();
    assert!(matches!(e, error::BinParse::Parse(_)));
}

fn good_mips() -> [Image; 4] {
    [
        Image::from_pixels(16, Box::new([0u8; 256])),
        Image::from_pixels(8, Box::new([0u8; 64])),
        Image::from_pixels(4, Box::new([0u8; 16])),
        Image::from_pixels(2, Box::new([0u8; 4])),
    ]
}

#[test]
fn miptex_new_short_name() {
    let miptex = MipTexture::new(String::from("hi"), good_mips());
    assert_eq!(miptex.name(), *b"hi\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    assert_eq!(miptex.name_to_string().unwrap(), String::from("hi"));
    assert_eq!(miptex.name_to_cstring(), CString::new("hi").unwrap());
}

#[test]
fn miptex_new_long_name() {
    let miptex = MipTexture::new(String::from("in_16_characters"), good_mips());
    assert_eq!(miptex.name(), *b"in_16_characters");
}

#[test]
#[should_panic]
fn miptex_name_too_long() {
    MipTexture::new(String::from("this_string_is_too_long"), good_mips());
}
