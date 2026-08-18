use std::ffi::CStr;
use std::mem::size_of;
use std::str::Utf8Error;
use std::string::{String, ToString};

use crate::common::Junk;
use crate::error;
use crate::BinParseResult;

pub const MAGIC: [u8; 4] = *b"WAD2";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C, packed)]
pub struct Head {
    magic: [u8; 4],
    entry_count: u32,
    directory_offset: u32,
}

impl Head {
    pub fn new(entry_count: u32, directory_offset: u32) -> Self {
        Head {
            magic: MAGIC,
            entry_count,
            directory_offset,
        }
    }

    pub fn entry_count(&self) -> u32 {
        self.entry_count
    }

    pub fn directory_offset(&self) -> u32 {
        self.directory_offset
    }
}

impl TryFrom<[u8; size_of::<Head>()]> for Head {
    type Error = error::BinParse;

    fn try_from(bytes: [u8; size_of::<Head>()]) -> Result<Self, Self::Error> {
        let mut chunks = bytes.chunks_exact(4usize);

        if chunks.next().unwrap() != &MAGIC[..] {
            let magic_str: String =
                MAGIC.iter().copied().map(char::from).collect();

            return Err(error::BinParse::Parse(format!(
                "Magic number does not match `{magic_str}`"
            )));
        }

        let entry_count = u32::from_le_bytes(
            <[u8; 4]>::try_from(chunks.next().unwrap()).unwrap(),
        );

        let directory_offset = u32::from_le_bytes(
            <[u8; 4]>::try_from(chunks.next().unwrap()).unwrap(),
        );

        Ok(Head::new(entry_count, directory_offset))
    }
}

/// Provides the location of a lump within a WAD archive, length of the lump,
/// name (16 bytes, null-terminated), and lump kind
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C, packed)]
pub struct Entry {
    offset: u32,
    length: u32,
    uncompressed_length: u32, // unused?
    lump_kind: u8,
    compression: u8, // 0 - uncompressed, other values unused?
    _padding: Junk<u16>,
    name: [u8; 16],
}

impl Entry {
    pub(crate) fn from_config(config: EntryConfig) -> BinParseResult<Entry> {
        let err_string =
            "WAD entry name doesn't terminate after 15 characters".to_string();

        CStr::from_bytes_until_nul(&config.name)
            .map_err(|_| error::BinParse::Parse(err_string))?;

        Ok(Entry {
            offset: config.offset,
            length: config.length,
            uncompressed_length: config.length,
            lump_kind: config.lump_kind,
            compression: 0u8,
            _padding: Junk::default(),
            name: config.name,
        })
    }

    /// Obtain the name as a C string.
    pub fn name_to_cstr(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.name).expect("unterminated name")
    }

    /// Attempt to interpret the name as UTF-8 encoded string
    pub fn name_to_str(&self) -> Result<&str, Utf8Error> {
        self.name_to_cstr().to_str()
    }

    /// Name in raw bytes
    pub fn name(&self) -> [u8; 16] {
        self.name
    }

    /// WAD offset of lump
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Length of lump in bytes
    pub fn length(&self) -> u32 {
        self.length
    }

    /// Lump kind as a byte
    pub fn kind(&self) -> u8 {
        self.lump_kind
    }
}

impl TryFrom<[u8; size_of::<Entry>()]> for Entry {
    type Error = error::BinParse;

    // Attempt to read an entry from a block of bytes.  Fails if compression
    // flag is on (unsupported).
    fn try_from(bytes: [u8; size_of::<Entry>()]) -> Result<Self, Self::Error> {
        let (offset_bytes, rest) = bytes.split_at(4);

        let offset =
            u32::from_le_bytes(<[u8; 4]>::try_from(offset_bytes).unwrap());

        let (length_bytes, rest) = rest.split_at(4);

        let length =
            u32::from_le_bytes(<[u8; 4]>::try_from(length_bytes).unwrap());

        let (uc_length_bytes, rest) = rest.split_at(4);

        let _uc_length =
            u32::from_le_bytes(<[u8; 4]>::try_from(uc_length_bytes).unwrap());

        let (&[lump_kind], rest) = rest.split_at(1) else {
            unreachable!()
        };

        let (&[compression], rest) = rest.split_at(1) else {
            unreachable!()
        };

        if compression != 0 {
            return Err(error::BinParse::Parse(
                "Compression is unsupported".to_string(),
            ));
        }

        let name: [u8; 16] = rest[2..].try_into().unwrap();

        Entry::from_config(EntryConfig {
            offset,
            length,
            lump_kind,
            name,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EntryConfig {
    pub offset: u32,
    pub length: u32,
    pub lump_kind: u8,
    pub name: [u8; 16],
}
