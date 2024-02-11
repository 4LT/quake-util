use crate::error;
use crate::lump::{kind, Lump};
use crate::wad;
use std::io::Cursor;
use std::iter::repeat;
use std::mem::{size_of, size_of_val};
use std::vec::Vec;
use wad::repr::Head;

fn image_bytes() -> Vec<u8> {
    let width: u32 = 64;
    let height: u32 = 128;
    let pix_ct: usize = (width * height).try_into().unwrap();

    let mut image = Vec::new();
    image.extend(width.to_le_bytes());
    image.extend(height.to_le_bytes());
    image.extend(repeat(0u8).take(pix_ct));

    image
}

fn miptex_bytes(name: [u8; 16]) -> Vec<u8> {
    let width: u32 = 512;
    let height: u32 = 32;
    let mip0_sz = width * height;
    let mut miptex = Vec::new();

    miptex.extend(name);
    miptex.extend(width.to_le_bytes());
    miptex.extend(height.to_le_bytes());

    let mut offset: u32 = miptex.len().try_into().unwrap();

    for i in 0..4 {
        miptex.extend(offset.to_le_bytes());
        let mip_sz = mip0_sz >> (2 * i);
        offset += mip_sz;
    }

    for i in 0..4 {
        let mip_sz = mip0_sz >> (2 * i);
        miptex.extend(repeat(0u8).take(mip_sz.try_into().unwrap()));
    }

    miptex
}

fn palette_bytes() -> Vec<u8> {
    let mut palette = Vec::new();

    for i in 0..768i32 {
        palette.push((i & 0xff).try_into().unwrap());
    }

    palette
}

fn entry_bytes(offset: u32, length: u32, kind: u8, name: [u8; 16]) -> Vec<u8> {
    let mut entry = Vec::new();

    entry.extend(offset.to_le_bytes());
    entry.extend(length.to_le_bytes());
    entry.extend(length.to_le_bytes());
    entry.push(kind);
    entry.push(0u8);
    entry.extend([0; 2]);
    entry.extend(name);

    entry
}

fn compressed_entry_bytes(offset: u32) -> Vec<u8> {
    let mut entry = Vec::new();

    entry.extend(offset.to_le_bytes());
    entry.extend([0; 8]);
    entry.push(kind::FLAT);
    entry.push(1u8);
    entry.extend([0; 2]);
    entry.extend(b"compressed\0\0\0\0\0\0");

    entry
}

fn duplicate_entry_wad_bytes() -> Vec<u8> {
    let mut wad = Vec::new();
    let entry_count = 2u32;
    let directory_offset = 12u32;
    let name = *b"same_name\0\0\0\0\0\0\0";

    wad.extend(b"WAD2");
    wad.extend(entry_count.to_le_bytes());
    wad.extend(directory_offset.to_le_bytes());

    let entry1 =
        entry_bytes(wad.len().try_into().unwrap(), 0, kind::FLAT, name);
    wad.extend(entry1);

    let entry2 =
        entry_bytes(wad.len().try_into().unwrap(), 0, kind::FLAT, name);
    wad.extend(entry2);

    wad
}

fn good_wad_bytes() -> Vec<u8> {
    let mut wad = Vec::new();
    let image_name = *b"image\0\0\0\0\0\0\0\0\0\0\0";
    let miptex_name = *b"miptex\0\0\0\0\0\0\0\0\0\0";
    let palette_name = *b"palette\0\0\0\0\0\0\0\0\0";
    let flat_name = *b"flat\0\0\0\0\0\0\0\0\0\0\0\0";
    let conchars_name = *b"CONCHARS\0\0\0\0\0\0\0\0";
    let entry_count: u32 = 5;
    let image = image_bytes();
    let miptex = miptex_bytes(miptex_name);
    let palette = palette_bytes();
    let flat = [0u8; 123];
    let conchars = [0u8; 128 * 128];
    let directory_offset: u32 = (image.len()
        + miptex.len()
        + palette.len()
        + flat.len()
        + conchars.len()
        + size_of::<Head>())
    .try_into()
    .unwrap();

    wad.extend(b"WAD2");
    wad.extend(entry_count.to_le_bytes());
    wad.extend(directory_offset.to_le_bytes());

    let image_offset: u32 = wad.len().try_into().unwrap();
    wad.extend(&image);

    let miptex_offset: u32 = wad.len().try_into().unwrap();
    wad.extend(&miptex);

    let palette_offset: u32 = wad.len().try_into().unwrap();
    wad.extend(&palette);

    let flat_offset: u32 = wad.len().try_into().unwrap();
    wad.extend(&flat);

    let conchars_offset: u32 = wad.len().try_into().unwrap();
    wad.extend(&conchars);

    wad.extend(entry_bytes(
        image_offset,
        image.len().try_into().unwrap(),
        kind::SBAR,
        image_name,
    ));
    wad.extend(entry_bytes(
        miptex_offset,
        miptex.len().try_into().unwrap(),
        kind::MIPTEX,
        miptex_name,
    ));
    wad.extend(entry_bytes(
        palette_offset,
        palette.len().try_into().unwrap(),
        kind::PALETTE,
        palette_name,
    ));
    wad.extend(entry_bytes(
        flat_offset,
        flat.len().try_into().unwrap(),
        kind::FLAT,
        flat_name,
    ));
    wad.extend(entry_bytes(
        conchars_offset,
        conchars.len().try_into().unwrap(),
        kind::MIPTEX,
        conchars_name,
    ));

    wad
}

#[test]
fn parse_good_wad() {
    let mut wad_file = Cursor::new(good_wad_bytes());
    let (mut parser, warnings) = wad::Parser::new(&mut wad_file).unwrap();
    let dir = parser.directory();
    let panic_dir = || panic!("{:?}", dir);
    let image_entry = dir.get("image").unwrap_or_else(panic_dir);
    let miptex_entry = dir.get("miptex").unwrap_or_else(panic_dir);
    let palette_entry = dir.get("palette").unwrap_or_else(panic_dir);
    let flat_entry = dir.get("flat").unwrap_or_else(panic_dir);
    let conchars_entry = dir.get("CONCHARS").unwrap_or_else(panic_dir);

    assert_eq!(warnings.len(), 0);
    assert_eq!(image_entry.kind(), kind::SBAR);
    assert_eq!(miptex_entry.kind(), kind::MIPTEX);
    assert_eq!(palette_entry.kind(), kind::PALETTE);
    assert_eq!(flat_entry.kind(), kind::FLAT);
    assert_eq!(conchars_entry.kind(), kind::MIPTEX);

    {
        let image_lump = parser.parse_image(image_entry).unwrap();
        let (width, height) = (image_lump.width(), image_lump.height());
        assert_eq!(width, 64);
        assert_eq!(height, 128);
        assert_eq!(
            width * height,
            image_lump.pixels().len().try_into().unwrap()
        );
    }

    {
        let miptex_lump = parser.parse_mip_texture(miptex_entry).unwrap();
        assert_eq!(miptex_lump.mip(0).width(), 512);
        assert_eq!(miptex_lump.mip(1).width(), 256);
        assert_eq!(miptex_lump.mip(2).width(), 128);
        assert_eq!(miptex_lump.mip(3).width(), 64);
        assert_eq!(miptex_lump.mip(0).height(), 32);
        assert_eq!(miptex_lump.mip(1).height(), 16);
        assert_eq!(miptex_lump.mip(2).height(), 8);
        assert_eq!(miptex_lump.mip(3).height(), 4);
    }

    {
        let palette_lump = parser.parse_palette(palette_entry).unwrap();
        let mut val = 0u8;

        for i in 0..256usize {
            assert_eq!(
                (*palette_lump)[i],
                [val, val.wrapping_add(1), val.wrapping_add(2)]
            );
            val = val.wrapping_add(3);
        }
    }

    {
        let flat_lump = parser.read_raw(flat_entry).unwrap();
        assert_eq!(flat_lump.len(), 123);
    }

    for (entry_name, entry) in dir {
        assert!(match &entry_name[..] {
            "image" => matches!(
                parser.parse_inferred(&entry).unwrap(),
                Lump::StatusBar(_)
            ),
            "miptex" => matches!(
                parser.parse_inferred(&entry).unwrap(),
                Lump::MipTexture(_)
            ),
            "palette" => matches!(
                parser.parse_inferred(&entry).unwrap(),
                Lump::Palette(_)
            ),
            "flat" =>
                matches!(parser.parse_inferred(&entry).unwrap(), Lump::Flat(_)),
            "CONCHARS" =>
                matches!(parser.parse_inferred(&entry).unwrap(), Lump::Flat(_)),
            name => panic!("Unexpected entry name `{name}`"),
        });
    }
}

#[test]
fn parse_duplicate_entry() {
    let mut wad_file = Cursor::new(duplicate_entry_wad_bytes());
    let (_, warnings) = wad::Parser::new(&mut wad_file).unwrap();

    assert_eq!(warnings.len(), 1);
}

#[test]
fn parse_bad_magic_wad() {
    let mut wad_file = Cursor::new(b"WART\0\0\0\0\0\0\0\0");
    let e = wad::Parser::new(&mut wad_file).unwrap_err();

    assert!(matches!(e, error::BinParse::Parse(_)));
}

#[test]
fn parse_bad_short_wad() {
    let mut wad_file = Cursor::new(b"WAD2");
    let e = wad::Parser::new(&mut wad_file).unwrap_err();

    assert!(matches!(e, error::BinParse::Io(_)));
}

#[test]
fn parse_bad_directory() {
    let mut wad_file = Cursor::new(b"WAD2\x01\0\0\0\0\0\0\0");
    let e = wad::Parser::new(&mut wad_file).unwrap_err();

    assert!(matches!(e, error::BinParse::Io(_)));
}

#[test]
fn parse_bad_compression_entry() {
    let mut wad = Vec::<u8>::new();
    wad.extend(b"WAD2\x01\0\0\0\x0c\0\0\0");
    wad.extend(compressed_entry_bytes(12));
    let mut wad_file = Cursor::new(wad);
    let e = wad::Parser::new(&mut wad_file).unwrap_err();

    assert!(matches!(e, error::BinParse::Parse(_)));
}

#[test]
fn parse_bad_lumps() {
    for kind in [kind::SBAR, kind::MIPTEX, kind::PALETTE, kind::FLAT] {
        let mut wad = Vec::<u8>::new();
        let bad_lump = b"\xBA\xDB\xAD";
        wad.extend(b"WAD2");
        wad.extend((1u32).to_le_bytes());

        wad.extend(
            <u32>::try_from(size_of::<Head>() + size_of_val(bad_lump))
                .unwrap()
                .to_le_bytes(),
        );

        wad.extend(bad_lump);

        wad.extend(entry_bytes(
            wad.len().try_into().unwrap(),
            666,
            kind,
            *b"BAD_BAD_BAD_BAD\0",
        ));

        let mut wad_file = Cursor::new(wad);
        let (mut parser, _) = wad::Parser::new(&mut wad_file).unwrap();
        let dir = parser.directory();
        let entry = dir.get("BAD_BAD_BAD_BAD").unwrap();

        assert!(matches!(
            parser.parse_inferred(entry),
            Err(error::BinParse::Io(_))
        ));
    }
}
