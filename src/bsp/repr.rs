use std::mem::size_of;
use std::mem::MaybeUninit;

pub const BSP_VERSION: u32 = 29;
pub const BSP2_VERSION: u32 = u32::from_le_bytes(*b"BSP2");
pub const ENTRY_COUNT: usize = 15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum EntryOffset {
    Entities = 0,
    Planes,
    Textures,
    Vertices,
    Vis,
    Nodes,
    TexInfo,
    Faces,
    Light,
    ClipNodes,
    Leaves,
    MarkSurfaces,
    Edges,
    SurfEdges,
    Models,
}

impl From<EntryOffset> for usize {
    fn from(offset: EntryOffset) -> Self {
        offset as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C, packed)]
pub struct Entry {
    pub offset: u32,
    pub length: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C, packed)]
pub struct Head {
    version: u32,
    entries: [Entry; ENTRY_COUNT],
}

impl Head {
    pub fn entry(&self, offset: EntryOffset) -> Entry {
        let idx: usize = offset.into();
        self.entries[idx]
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

impl TryFrom<[u8; size_of::<Head>()]> for Head {
    type Error = crate::BinParseError;

    fn try_from(bytes: [u8; size_of::<Head>()]) -> crate::BinParseResult<Head> {
        let version =
            u32::from_le_bytes(<[u8; 4]>::try_from(&bytes[..4]).unwrap());

        if version != BSP_VERSION && version != BSP2_VERSION {
            return Err(crate::BinParseError::Parse(format!(
                "Unrecognized BSP version {} ({:?})",
                version,
                version.to_le_bytes(),
            )));
        }

        let rest = &bytes[4..];
        let mut entries = [MaybeUninit::<Entry>::uninit(); ENTRY_COUNT];

        for (idx, chunk) in rest.chunks(size_of::<Entry>()).enumerate() {
            entries[idx] = MaybeUninit::new(Entry {
                offset: u32::from_le_bytes(
                    <[u8; 4]>::try_from(&chunk[..4]).unwrap(),
                ),
                length: u32::from_le_bytes(
                    <[u8; 4]>::try_from(&chunk[4..]).unwrap(),
                ),
            });
        }

        let entries = entries.map(|entry| unsafe { entry.assume_init() });

        Ok(Head { version, entries })
    }
}
