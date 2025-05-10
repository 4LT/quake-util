#[cfg(feature = "std")]
fn main() {
    use bsp::EntryOffset;
    use quake_util::bsp;
    use std::env::args;
    use std::ffi::CString;
    use std::io;

    let mut arguments = args();

    let bsp_path = if let Some(path) = arguments.nth(1) {
        path
    } else {
        panic!("No arguments");
    };

    let file = std::fs::File::open(bsp_path).unwrap();
    let mut reader = io::BufReader::new(file);
    let mut parser = bsp::Parser::new(&mut reader).unwrap();

    let lighting = if parser.lump_empty(EntryOffset::Light) {
        "No"
    } else {
        "Yes"
    };

    let vis = if parser.lump_empty(EntryOffset::Vis) {
        "No"
    } else {
        "Yes"
    };

    let qmap = parser.parse_entities().unwrap();

    let mut map_name = String::from("<None>");

    for edict in qmap.entities.into_iter().map(|e| e.edict) {
        let map_name_cstr = if edict
            .iter()
            .flat_map(|(key, value)| {
                if key == &CString::new("classname").unwrap() {
                    Some(value)
                } else {
                    None
                }
            })
            .next()
            == Some(&CString::new("worldspawn").unwrap())
        {
            edict
                .iter()
                .flat_map(|(key, value)| {
                    if key == &CString::new("message").unwrap() {
                        Some(value)
                    } else {
                        None
                    }
                })
                .next()
                .map(Clone::clone)
        } else {
            None
        };

        if let Some(map_name_cstr) = map_name_cstr {
            map_name = format!("\"{}\"", map_name_cstr.to_string_lossy());
        }
    }

    println!(
        r"Map Name: {map_name}
Lighting: {lighting}
VIS: {vis}"
    );
}

#[cfg(not(feature = "std"))]
fn main() {}
