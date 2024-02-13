//! Data lumps as used in WAD archive or as loose files

mod parse;
mod repr;

pub use parse::{parse_image, parse_mip_texture, parse_palette, read_raw};

pub use repr::{Image, Lump, MipTexture, MipTextureHead};

/// Lump identifiers
pub mod kind {
    /// 768 byte (256 packed colors) palette lump
    pub const PALETTE: u8 = 0x40;

    /// 2D image lump
    pub const SBAR: u8 = 0x42;

    /// Mip-mapped texture lump
    pub const MIPTEX: u8 = 0x44;

    /// Raw (headerless) bytes lump
    pub const FLAT: u8 = 0x45;
}

#[cfg(test)]
mod parse_test;

#[cfg(test)]
mod repr_test;
