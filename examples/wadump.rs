use quake_util::{wad, Palette, QUAKE_PALETTE};
use wad::Lump;

use std::env::args;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter, Write};
use std::mem::transmute;
use std::path::PathBuf;

use png::{ColorType, Encoder};

fn main() {
    let arguments = args();

    let wad_path = if let Some(wad_path) = arguments.skip(1).next() {
        wad_path
    } else {
        panic!("No arguments");
    };

    let file = File::open(wad_path).expect("Could not open file");
    let mut reader = BufReader::new(file);

    let entries = wad::parse_directory(&mut reader).unwrap();

    for entry in entries {
        let name = entry.name_as_cstring();
        let name = name.to_string_lossy();

        let lump = wad::parse_lump(&entry, &mut reader)
            .map_err(|e| format!("`{}`: {}", name, e))
            .unwrap();

        match lump {
            Lump::MipTexture(tex) => {
                println!("Writing texture...");
                for (idx, image) in tex.into_iter().enumerate() {
                    write_png(
                        &format!("{}.{}", &name, idx,),
                        image.width(),
                        image.pixels(),
                    );
                }
            }
            Lump::Palette(bytes) => {
                println!("Writing palette...");
                write_palette(&name, &bytes);
            }
            Lump::StatusBar(img) => {
                println!("Writing image...");
                write_png(&name, img.width(), img.pixels());
            }
            Lump::Flat(bytes) => {
                let dimensions = if &name == "CONCHARS" {
                    Some((128u32, 128u32))
                } else if &name == "CONBACK" {
                    Some((320u32, 200u32))
                } else {
                    eprintln!("Unknown lump \"{}\"", &name);
                    None
                };

                if let Some((width, height)) = dimensions {
                    if bytes.len() as u32 == width * height {
                        println!("Writing {} image...", name);
                        write_png(&name, width, &bytes);
                    } else {
                        eprintln!("Bad dimensions for \"{}\"", &name);
                    }
                }
            }
        }
    }
}

fn new_writer(file_name: &str) -> impl Write {
    let mut path = PathBuf::from("dump");
    create_dir_all(&path).unwrap();
    path.push(file_name);
    let file = File::create(path).unwrap();
    BufWriter::new(file)
}

fn write_png(name: &str, width: u32, pixels: &[u8]) {
    let height = pixels.len() as u32 / width;
    let writer = new_writer(&format!("{}.png", name));
    let mut encoder = Encoder::new(writer, width, height);
    encoder.set_color(ColorType::Rgb);
    let mut writer = encoder.write_header().unwrap();
    let colors = pixels_to_colors(pixels);
    writer
        .write_image_data(
            &colors.iter().flatten().copied().collect::<Vec<u8>>(),
        )
        .unwrap();
}

fn write_palette(name: &str, bytes: &Palette) {
    let mut writer = new_writer(&format!("{}.lmp", name));
    writer
        .write(&bytes.iter().flatten().copied().collect::<Vec<u8>>())
        .unwrap();
}

fn pixels_to_colors(pixels: &[u8]) -> Box<[[u8; 3]]> {
    let ct = pixels.len();
    let mut colors = Box::<[[u8; 3]]>::from(vec![[0u8; 3]; ct]);

    for (idx, pixel) in pixels.iter().copied().enumerate() {
        colors[idx] = QUAKE_PALETTE[usize::from(pixel)];
    }

    colors
}
