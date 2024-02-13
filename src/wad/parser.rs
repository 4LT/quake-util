use crate::{error, error::BinParseResult, lump, wad, Palette};
use io::{Read, Seek, SeekFrom};
use lump::Lump;
use std::boxed::Box;
use std::collections::hash_map::Entry as MapEntry;
use std::collections::HashMap;
use std::io;
use std::mem::size_of;
use std::mem::size_of_val;
use std::string::{String, ToString};
use std::vec::Vec;
use wad::repr::Head;

/// WAD parser.  Wraps a mutable reference to a Read + Seek cursor to provide
/// random read access.
#[derive(Debug)]
pub struct Parser<'a, Reader: Seek + Read> {
    cursor: &'a mut Reader,
    start: u64,
    directory: HashMap<String, wad::Entry>,
}

impl<'a, Reader: Seek + Read> Parser<'a, Reader> {
    /// Constructs a new wad parser starting at the provided cursor.  May
    /// produce a list of warnings for duplicate entriess (entries sharing the
    /// same name).
    pub fn new(cursor: &'a mut Reader) -> BinParseResult<(Self, Vec<String>)> {
        let start = cursor.stream_position().map_err(error::BinParse::Io)?;
        let (directory, warnings) = parse_directory(cursor, start)?;

        Ok((
            Self {
                cursor,
                start,
                directory,
            },
            warnings,
        ))
    }

    /// Clones WAD entries into a hash map.  Entries are used to access lumps
    /// within the WAD.
    pub fn directory(&self) -> HashMap<String, wad::Entry> {
        self.directory.clone()
    }

    /// Attempts to parse a mip-mapped texture at the offset provided by the
    /// entry
    pub fn parse_mip_texture(
        &mut self,
        entry: &wad::Entry,
    ) -> BinParseResult<lump::MipTexture> {
        self.seek_to_entry(entry)?;
        lump::parse_mip_texture(self.cursor)
    }

    /// Attempts to parse a 2D at the offset provided by the entry
    pub fn parse_image(
        &mut self,
        entry: &wad::Entry,
    ) -> BinParseResult<lump::Image> {
        self.seek_to_entry(entry)?;
        lump::parse_image(self.cursor)
    }

    /// Attempts to parse a 768 byte palette at the offset provided by the entry
    pub fn parse_palette(
        &mut self,
        entry: &wad::Entry,
    ) -> BinParseResult<Box<Palette>> {
        self.seek_to_entry(entry)?;
        lump::parse_palette(self.cursor)
    }

    /// Attempts to read a number of bytes using the provided entry's length and
    /// offset
    pub fn read_raw(
        &mut self,
        entry: &wad::Entry,
    ) -> BinParseResult<Box<[u8]>> {
        self.seek_to_entry(entry)?;
        let length = usize::try_from(entry.length()).map_err(|_| {
            error::BinParse::Parse("Length too large".to_string())
        })?;
        lump::read_raw(self.cursor, length)
    }

    /// Attempts to read a lump based on the provided entry's name and lump
    /// kind.  All known kinds of lump are attempted based on the entry.  E.g.
    /// there is a special case where Quake's gfx.wad has a flat lump named
    /// CONCHARS which is erroneously tagged as miptex.
    pub fn parse_inferred(
        &mut self,
        entry: &wad::Entry,
    ) -> BinParseResult<Lump> {
        const CONCHARS_NAME: &[u8; 9] = b"CONCHARS\0";

        let mut attempt_order = [
            lump::kind::MIPTEX,
            lump::kind::SBAR,
            lump::kind::PALETTE,
            lump::kind::FLAT,
        ];

        // Some paranoid nonsense because not even Id can be trusted to tag
        // their lumps correctly
        let mut prioritize = |first_kind| {
            let mut index = 0;

            for (i, kind) in attempt_order.into_iter().enumerate() {
                if kind == first_kind {
                    index = i;
                }
            }

            while index > 0 {
                attempt_order[index] = attempt_order[index - 1];
                attempt_order[index - 1] = first_kind;
                index -= 1;
            }
        };

        prioritize(entry.kind());

        let length = usize::try_from(entry.length()).map_err(|_| {
            error::BinParse::Parse("Length too large".to_string())
        })?;

        // It's *improbable* that a palette-sized lump could be a valid
        // status bar image OR miptex, though it's possibly just 768
        // rando bytes.  So if the explicit type is FLAT and it's 768 bytes,
        // we can't know for sure that it
        if length == size_of::<Palette>() && entry.kind() != lump::kind::FLAT {
            prioritize(lump::kind::PALETTE);
        }

        // Quake's gfx.wad has CONCHARS's type set explicitly to MIPTEX,
        // even though it's a FLAT (128x128 pixels)
        if entry.name()[..size_of_val(CONCHARS_NAME)] == CONCHARS_NAME[..] {
            prioritize(lump::kind::FLAT);
        }

        let mut last_error = error::BinParse::Parse("Unreachable".to_string());

        for attempt_kind in attempt_order {
            match attempt_kind {
                lump::kind::MIPTEX => match self.parse_mip_texture(entry) {
                    Ok(miptex) => {
                        return Ok(Lump::MipTexture(miptex));
                    }
                    Err(e) => {
                        last_error = e;
                    }
                },
                lump::kind::SBAR => match self.parse_image(entry) {
                    Ok(img) => {
                        return Ok(Lump::StatusBar(img));
                    }
                    Err(e) => {
                        last_error = e;
                    }
                },
                lump::kind::PALETTE => match self.parse_palette(entry) {
                    Ok(pal) => {
                        return Ok(Lump::Palette(pal));
                    }
                    Err(e) => {
                        last_error = e;
                    }
                },
                lump::kind::FLAT => match self.read_raw(entry) {
                    Ok(bytes) => {
                        return Ok(Lump::Flat(bytes));
                    }
                    Err(e) => {
                        last_error = e;
                    }
                },
                _ => unreachable!(),
            }
        }

        Err(last_error)
    }

    fn seek_to_entry(&mut self, entry: &wad::Entry) -> BinParseResult<()> {
        let offset = self
            .start
            .checked_add(entry.offset().into())
            .ok_or(error::BinParse::Parse("Offset too large".to_string()))?;

        self.cursor.seek(SeekFrom::Start(offset))?;
        Ok(())
    }
}

fn parse_directory(
    cursor: &mut (impl Seek + Read),
    start: u64,
) -> BinParseResult<(HashMap<String, wad::Entry>, Vec<String>)> {
    let mut header_bytes = [0u8; size_of::<Head>()];
    cursor.read_exact(&mut header_bytes[..])?;
    let header: Head = header_bytes.try_into()?;
    let entry_ct = header.entry_count();
    let dir_offset = header.directory_offset();

    let dir_pos = start
        .checked_add(dir_offset.into())
        .ok_or(error::BinParse::Parse("Offset too large".to_string()))?;

    cursor
        .seek(SeekFrom::Start(dir_pos))
        .map_err(error::BinParse::Io)?;

    let mut entries = HashMap::<String, wad::Entry>::with_capacity(
        entry_ct.try_into().unwrap(),
    );

    let mut warnings = Vec::new();

    for _ in 0..entry_ct {
        const WAD_ENTRY_SIZE: usize = size_of::<wad::Entry>();
        let mut entry_bytes = [0u8; WAD_ENTRY_SIZE];
        cursor.read_exact(&mut entry_bytes[0..WAD_ENTRY_SIZE])?;
        let entry: wad::Entry = entry_bytes.try_into()?;

        let entry_name = entry
            .name_to_string()
            .map_err(|e| error::BinParse::Parse(e.to_string()))?;

        if let MapEntry::Vacant(map_entry) = entries.entry(entry_name.clone()) {
            map_entry.insert(entry);
        } else {
            warnings
                .push(format!("Skipping duplicate entry for `{entry_name}`"));
        }
    }

    Ok((entries, warnings))
}
