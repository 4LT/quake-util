use std::io::{BufReader, sink};
use std::fs::File;

use quake_util::qmap::{ lex, parse, QuakeMapElement };

#[test]
fn bench_parse() -> std::io::Result<()> {
    let f = File::open("test-res/ad_heresp2.map")?;
    let reader = BufReader::new(f);

    let tokens = match lex(reader) {
        Ok(tokens) => tokens,
        Err(err) => panic!("{}", err)
    };
    
    let ast = match parse(tokens) {
        Ok(ast) => ast,
        Err(err) => panic!("{}", err)
    };

    ast.write_to(&mut sink())?;

    Ok(())
}
