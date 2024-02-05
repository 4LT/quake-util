mod parse;
mod repr;

pub use parse::{
    parse_image, parse_mip_texture, parse_palette, read_raw, ParseInferenceInfo,
};

pub use repr::{Image, Lump, MipTexture, MipTextureHead};

pub mod kind {
    pub const PALETTE: u8 = 0x40;
    pub const SBAR: u8 = 0x42;
    pub const MIPTEX: u8 = 0x44;
    pub const FLAT: u8 = 0x45;
}
