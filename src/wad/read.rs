use crate::wad::repr::{Entry, Head};
use crate::wad::{ReadError, ReadResult};
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;
use std::vec::Vec;

pub fn directory(mut cursor: impl Seek + Read) -> ReadResult<Vec<Entry>> {
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
