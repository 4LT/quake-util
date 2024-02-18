use crate::{bsp, BinParseError};
use bsp::EntryOffset;
use std::ffi::CString;
use std::io::Cursor;
use std::mem::size_of;
use std::vec::Vec;

const HEAD_SZ: usize = size_of::<bsp::Head>();
const ENTITIES: &str = r#"
{
    "classname" "func_door"
    "model" "*37"
}"#;
const ENTITIES_LEN: usize = ENTITIES.len() + 1;

#[test]
fn parse_good_bsp() {
    const MODELS_LEN: usize = 123;
    let mut bytes = [0u8; HEAD_SZ + ENTITIES_LEN + MODELS_LEN];
    let entities_offset = HEAD_SZ as u32;
    let models_offset = entities_offset + ENTITIES_LEN as u32;
    bytes[0] = b'B';
    bytes[1] = b'S';
    bytes[2] = b'P';
    bytes[3] = b'2';
    bytes[4..8].copy_from_slice(&(entities_offset).to_le_bytes());
    bytes[8..12].copy_from_slice(&(ENTITIES_LEN as u32).to_le_bytes());
    bytes[(HEAD_SZ - 8)..(HEAD_SZ - 4)]
        .copy_from_slice(&(models_offset).to_le_bytes());
    bytes[(HEAD_SZ - 4)..HEAD_SZ]
        .copy_from_slice(&(MODELS_LEN as u32).to_le_bytes());
    bytes[HEAD_SZ..(HEAD_SZ + ENTITIES_LEN - 1)]
        .copy_from_slice(&ENTITIES.bytes().collect::<Vec<u8>>());

    let mut cursor = Cursor::new(bytes);
    let mut parser = bsp::Parser::new(&mut cursor).unwrap();

    {
        let lump_reader = parser.lump_reader(EntryOffset::Models).unwrap();
        assert_eq!(lump_reader.limit(), MODELS_LEN as u64);
    }

    assert!(!parser.lump_empty(EntryOffset::Entities));
    assert!(parser.lump_empty(EntryOffset::Nodes));

    let qmap = parser.parse_entities().unwrap();

    assert_eq!(
        qmap.entities[0]
            .edict
            .get(&CString::new("classname").unwrap()),
        Some(&CString::new("func_door").unwrap())
    );

    assert_eq!(
        qmap.entities[0].edict.get(&CString::new("model").unwrap()),
        Some(&CString::new("*37").unwrap())
    );
}

#[test]
fn parse_empty_entities() {
    let mut bytes = [0u8; HEAD_SZ];
    bytes[0] = 29;
    bytes[4..8].copy_from_slice(&(HEAD_SZ as u32).to_le_bytes());
    bytes[8..12].copy_from_slice(&(0u32).to_le_bytes());

    let mut cursor = Cursor::new(bytes);
    let mut parser = bsp::Parser::new(&mut cursor).unwrap();

    let quake_map = parser.parse_entities().unwrap();

    assert_eq!(quake_map.entities.len(), 0);
}

#[test]
fn parse_bad_entities() {
    let mut bytes = [0u8; HEAD_SZ + 2];
    bytes[0] = 29;
    bytes[4..8].copy_from_slice(&(HEAD_SZ as u32).to_le_bytes());
    bytes[8..12].copy_from_slice(&(2u32).to_le_bytes());
    bytes[HEAD_SZ] = b'{';
    bytes[HEAD_SZ + 1] = 0;

    let mut cursor = Cursor::new(bytes);
    let mut parser = bsp::Parser::new(&mut cursor).unwrap();

    let error = parser.parse_entities().unwrap_err();

    assert!(matches!(error, BinParseError::Parse(_)));
}
