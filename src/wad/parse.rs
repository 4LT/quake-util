use crate::wad::repr::{Entry, Head};
use crate::wad::{ReadError, ReadResult};
use io::{Read, Seek, SeekFrom};
use std::io;
use std::mem::size_of;
use std::vec::Vec;

pub fn parse_directory(
    cursor: &mut (impl Seek + Read),
) -> ReadResult<Vec<Entry>> {
    let wad_start = cursor.stream_position().map_err(ReadError::Io)?;

    let mut header_bytes = [0u8; size_of::<Head>()];
    cursor.read_exact(&mut header_bytes[..])?;
    let header: Head = header_bytes.try_into().map_err(ReadError::Parse)?;
    let entry_ct = header.entry_count();
    let dir_offset = header.directory_offset();

    let dir_pos =
        wad_start.checked_add(dir_offset.into()).ok_or_else(|| {
            ReadError::Io(io::Error::new(
                io::ErrorKind::Other,
                "Offset too large",
            ))
        })?;

    cursor
        .seek(SeekFrom::Start(dir_pos))
        .map_err(ReadError::Io)?;

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
