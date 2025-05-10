#[cfg(feature = "std")]
fn main() {
    use quake_util::qmap;
    use std::env::args;
    use std::io;

    let mut arguments = args();
    arguments.next();

    let inpath = if let Some(path) = arguments.next() {
        path
    } else {
        panic!("No input path");
    };

    let outpath = if let Some(path) = arguments.next() {
        path
    } else {
        panic!("No output path");
    };

    let infile = std::fs::File::open(inpath).unwrap();
    let mut reader = io::BufReader::new(infile);
    let map = qmap::parse(&mut reader).unwrap();

    let outfile = std::fs::File::create(outpath).unwrap();
    let mut writer = io::BufWriter::new(outfile);
    map.write_to(&mut writer).unwrap();
}

#[cfg(not(feature = "std"))]
fn main() {}
