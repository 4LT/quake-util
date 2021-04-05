use std::io::{BufReader, sink};
use std::fs::File;

use quake_util::qmap::{ lex, parse, QuakeMapElement };

#[test]
fn bench_parse() -> std::io::Result<()> {
    let f = File::open("test-res/ad_heresp2.map")?;
    let reader = BufReader::new(f);

    let tokens = lex(reader);
    let parse_result = parse(tokens); 
    
    match parse_result {
        Ok(ast) => {
            ast.write_to(&mut sink())?;
        },
        Err(err) => panic!("{}", err.message())
    }

    Ok(())
}
