#[cfg(feature = "std")]
use quake_util::{lump, wad};

#[cfg(feature = "std")]
use std::{env::args, fs::File, io::BufReader};

#[cfg(feature = "std")]
fn main() {
    let mut arguments = args();

    let arg1 = if let Some(arg1) = arguments.nth(1) {
        arg1
    } else {
        panic!("No arguments");
    };

    let file = File::open(arg1).expect("Could not open file");
    let mut cursor = BufReader::new(file);

    let mut parser = wad::Parser::new(&mut cursor).unwrap();

    for entry in parser.directory().to_vec() {
        let name = entry.name_to_str().expect("Bad entry name").to_string();
        print!("Entry `{}`: ", name);

        match &parser
            .parse_inferred(&entry)
            .map_err(|e| format!("{}: {}", name, e))
            .unwrap()
        {
            lump::Lump::MipTexture(tex) => {
                println!("Texture");
                for image in tex.mips() {
                    println!(
                        "\t{}x{}: {} bytes",
                        image.width(),
                        image.height(),
                        image.pixels().len()
                    );
                }
            }
            lump::Lump::Palette(_) => {
                println!("Palette");
                println!("\t768 bytes");
            }
            lump::Lump::StatusBar(img) => {
                println!("Status bar image");
                println!(
                    "\t{}x{}: {} bytes",
                    img.width(),
                    img.height(),
                    img.pixels().len(),
                );
            }
            lump::Lump::Flat(bytes) => {
                println!("Flat");
                println!("\t{} bytes", bytes.len());
            }
        }
    }
}

#[cfg(not(feature = "std"))]
fn main() {}
