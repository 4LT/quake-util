use crate::{bsp, BinParseError};
use bsp::{Entry, EntryOffset, BSP2_VERSION, BSP_VERSION};
use std::mem::size_of;

#[test]
fn get_offset_as_integer() {
    assert_eq!(usize::from(EntryOffset::Entities), 0);
    assert_eq!(usize::from(EntryOffset::Planes), 1);
    assert_eq!(usize::from(EntryOffset::Textures), 2);
    assert_eq!(usize::from(EntryOffset::Vertices), 3);
    assert_eq!(usize::from(EntryOffset::Vis), 4);
    assert_eq!(usize::from(EntryOffset::Nodes), 5);
    assert_eq!(usize::from(EntryOffset::TexInfo), 6);
    assert_eq!(usize::from(EntryOffset::Faces), 7);
    assert_eq!(usize::from(EntryOffset::Light), 8);
    assert_eq!(usize::from(EntryOffset::ClipNodes), 9);
    assert_eq!(usize::from(EntryOffset::Leaves), 10);
    assert_eq!(usize::from(EntryOffset::MarkSurfaces), 11);
    assert_eq!(usize::from(EntryOffset::Edges), 12);
    assert_eq!(usize::from(EntryOffset::SurfEdges), 13);
    assert_eq!(usize::from(EntryOffset::Models), 14);
}

#[test]
fn bsp2_head_from_bytes() {
    let mut bytes = [0u8; size_of::<bsp::Head>()];
    bytes[0] = b'B';
    bytes[1] = b'S';
    bytes[2] = b'P';
    bytes[3] = b'2';

    let head: bsp::Head = bytes.try_into().unwrap();
    assert_eq!(head.version(), BSP2_VERSION);
}

#[test]
fn bsp29_head_from_bytes() {
    let mut bytes = [0u8; size_of::<bsp::Head>()];
    bytes[0] = 29;

    let head: bsp::Head = bytes.try_into().unwrap();
    assert_eq!(head.version(), BSP_VERSION);
}

#[test]
fn get_bsp_entries() {
    const HEAD_SZ: usize = size_of::<bsp::Head>();
    let mut bytes = [0u8; HEAD_SZ];
    bytes[0] = 29;
    bytes[4..8].copy_from_slice(&(300u32).to_le_bytes());
    bytes[8..12].copy_from_slice(&(37u32).to_le_bytes());
    bytes[(HEAD_SZ - 8)..(HEAD_SZ - 4)]
        .copy_from_slice(&(667u32).to_le_bytes());
    bytes[(HEAD_SZ - 4)..HEAD_SZ].copy_from_slice(&(1000u32).to_le_bytes());

    let head: bsp::Head = bytes.try_into().unwrap();
    let entities_entry = head.entry(EntryOffset::Entities);
    let models_entry = head.entry(EntryOffset::Models);

    assert_eq!(
        entities_entry,
        Entry {
            offset: 300,
            length: 37
        }
    );
    assert_eq!(
        models_entry,
        Entry {
            offset: 667,
            length: 1000
        }
    );
}

#[test]
fn bad_version_head() {
    let mut bytes = [0u8; size_of::<bsp::Head>()];
    bytes[0] = 13;

    let err = bsp::Head::try_from(bytes).unwrap_err();
    assert!(matches!(err, BinParseError::Parse(_)));
}
