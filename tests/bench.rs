use std::fs::File;
use std::io::{sink, BufReader};

use quake_util::qmap::{parse, Writes};

#[test]
fn bench_parse() -> std::io::Result<()> {
    let f = File::open("test-res/ad_heresp2.map")?;
    let reader = BufReader::new(f);

    let ast = match parse(reader) {
        Ok(ast) => ast,
        Err(err) => panic!("{}", err),
    };

    ast.write_to(&mut sink())?;

    Ok(())
}
