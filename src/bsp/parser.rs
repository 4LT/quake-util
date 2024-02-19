use super::{Entry, EntryOffset, Head};
use crate::{BinParseError, BinParseResult, TextParseError};
use io::{Read, Seek, SeekFrom};
use std::io;
use std::mem::size_of;
use std::string::String;

use crate::qmap;
use qmap::QuakeMap;

#[derive(Debug)]
pub struct Parser<'a, Reader: Seek + Read> {
    cursor: &'a mut Reader,
    start: u64,
    header: Head,
}

impl<'a, Reader: Seek + Read> Parser<'a, Reader> {
    pub fn new(cursor: &'a mut Reader) -> BinParseResult<Self> {
        let start = cursor.stream_position()?;
        let mut header_bytes = [0u8; size_of::<Head>()];
        cursor.read_exact(&mut header_bytes[..])?;
        let header = header_bytes.try_into()?;

        Ok(Self {
            cursor,
            start,
            header,
        })
    }

    pub fn version(&self) -> u32 {
        self.header.version()
    }

    pub fn lump_reader(
        &mut self,
        entry_offset: EntryOffset,
    ) -> BinParseResult<std::io::Take<&mut Reader>> {
        let Entry { offset, length } = self.header.entry(entry_offset);
        let length = length.into();

        let abs_offset = self
            .start
            .checked_add(offset.into())
            .ok_or(BinParseError::Parse(String::from("Bad offset")))?;

        self.cursor.seek(SeekFrom::Start(abs_offset))?;

        Ok(self.cursor.take(length))
    }

    pub fn lump_empty(&self, offset: EntryOffset) -> bool {
        let length = self.header.entry(offset).length;
        length == 0
    }

    pub fn parse_entities(&mut self) -> BinParseResult<QuakeMap> {
        let lump = self.lump_reader(EntryOffset::Entities)?;
        let byte_iter = lump.bytes().take_while(|b| match b {
            Ok(byte) => byte != &0,
            _ => true,
        });

        qmap::parse(&mut IterReader::new(byte_iter)).map_err(|e| match e {
            TextParseError::Io(ioe) => ioe.into(),
            err => BinParseError::Parse(format!("{err}")),
        })
    }
}

struct IterReader<I>
where
    I: Iterator<Item = Result<u8, io::Error>>,
{
    iter: I,
}

impl<I> IterReader<I>
where
    I: Iterator<Item = Result<u8, io::Error>>,
{
    pub fn new(iter: I) -> Self {
        IterReader { iter }
    }
}

impl<I> Read for IterReader<I>
where
    I: Iterator<Item = Result<u8, io::Error>>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let count = buf.len();
        let mut i = 0;

        while i < count {
            let b = self.iter.next();

            match b {
                None => {
                    return Ok(i);
                }
                Some(Err(e)) => {
                    return Err(e);
                }
                Some(Ok(byte)) => {
                    buf[i] = byte;
                }
            }

            i += 1;
        }

        Ok(i)
    }
}
