use quake_util::wad;
use std::env::args;
use std::fs::File;
use std::io::BufReader;
use wad::Lump;

fn main() {
    let arguments = args();

    let arg1 = if let Some(arg1) = arguments.skip(1).next() {
        arg1
    } else {
        panic!("No arguments");
    };

    let file = File::open(arg1).expect("Could not open file");
    let mut reader = BufReader::new(file);

    let entries = wad::parse_directory(&mut reader).unwrap();

    for entry in entries {
        let name = entry.name_as_cstring();
        let name = name.to_string_lossy();

        let lump = wad::parse_lump(&entry, &mut reader)
            .map_err(|e| format!("`{}`: {}", name, e))
            .unwrap();

        print!("Entry `{}`: ", name);

        match lump {
            Lump::MipTexture(tex) => {
                println!("Texture");
                for image in &tex {
                    println!(
                        "\t{}x{}: {} bytes",
                        image.width(),
                        image.height(),
                        image.pixels().len()
                    );
                }
            }
            Lump::Palette(_) => {
                println!("Palette");
                println!("\t768 bytes");
            }
            Lump::StatusBar(img) => {
                println!("Status bar image");
                println!(
                    "\t{}x{}: {} bytes",
                    img.width(),
                    img.height(),
                    img.pixels().len(),
                );
            }
            Lump::Flat(bytes) => {
                println!("Flat");
                println!("\t{} bytes", bytes.len());
            }
        }
    }
}
