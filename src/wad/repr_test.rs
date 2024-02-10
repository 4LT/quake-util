use super::repr::{Entry, EntryConfig, Head};
use crate::error;
use std::{ffi::CString, string::String};

#[test]
fn construct_head() {
    let expected_count = 13;
    let expected_dir_offset = 43234;
    let head = Head::new(expected_count, expected_dir_offset);
    assert_eq!(head.entry_count(), expected_count);
    assert_eq!(head.directory_offset(), expected_dir_offset);
}

#[test]
fn parse_good_head() {
    let expected_count: u32 = 37;
    let expected_dir_offset: u32 = 600;
    let mut bytes = [0; std::mem::size_of::<Head>()];
    bytes[0..4].copy_from_slice(b"WAD2");
    bytes[4..8].copy_from_slice(&expected_count.to_le_bytes());
    bytes[8..12].copy_from_slice(&expected_dir_offset.to_le_bytes());

    let head: Head = bytes.try_into().unwrap();

    assert_eq!(head.entry_count(), expected_count);
    assert_eq!(head.directory_offset(), expected_dir_offset);
}

#[test]
fn parse_bad_head() {
    let mut bytes = [0; std::mem::size_of::<Head>()];
    bytes[0..4].copy_from_slice(b"BAD2");

    let err = Head::try_from(bytes).unwrap_err();

    match err {
        error::BinParse::Parse(_) => {}
        _ => panic!("Incorrect error type"),
    }
}

#[test]
fn construct_entry() {
    let expected_offset = 200;
    let expected_length = 111;
    let expected_kind = crate::lump::kind::SBAR;
    let expected_name = [
        b'h', b'e', b'l', b'l', b'o', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let config = EntryConfig {
        offset: expected_offset,
        length: expected_length,
        lump_kind: expected_kind,
        name: expected_name,
    };

    let entry = Entry::new(config);

    assert_eq!(entry.name(), expected_name);
    assert_eq!(entry.offset(), expected_offset);
    assert_eq!(entry.length(), expected_length);
    assert_eq!(entry.kind(), expected_kind);
    assert_eq!(
        entry.name_to_cstring(),
        CString::new(String::from("hello")).unwrap()
    );
    assert_eq!(entry.name_to_string(), Ok(String::from("hello")));
}

#[test]
fn parse_good_entry() {
    let expected_offset: u32 = 20049;
    let expected_length: u32 = 3001;
    let expected_kind = crate::lump::kind::MIPTEX;
    let expected_name = [
        b'h', b'o', b'w', b'd', b'y', b'_', b'p', b'a', b'r', b't', b'n', b'e',
        b'r', 0, 0, 0,
    ];

    let mut bytes = [0; std::mem::size_of::<Entry>()];
    bytes[0..4].copy_from_slice(&expected_offset.to_le_bytes());
    bytes[4..8].copy_from_slice(&expected_length.to_le_bytes());
    bytes[8..12].copy_from_slice(&expected_length.to_le_bytes());
    bytes[12] = expected_kind;
    bytes[13] = 0u8;
    bytes[16..].copy_from_slice(&expected_name);

    let entry: Entry = bytes.try_into().unwrap();

    assert_eq!(entry.name(), expected_name);
    assert_eq!(entry.offset(), expected_offset);
    assert_eq!(entry.length(), expected_length);
    assert_eq!(entry.kind(), expected_kind);
}

#[test]
fn parse_entry_bad_compression() {
    let mut bytes = [0; std::mem::size_of::<Entry>()];
    bytes[13] = 1u8;

    let err = Entry::try_from(bytes).unwrap_err();

    match err {
        error::BinParse::Parse(_) => {}
        _ => panic!("Incorrect error type"),
    }
}
