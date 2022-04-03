#![feature(test)]

extern crate test;

use std::fs::File;
use std::io::{sink, BufReader};
use test::Bencher;

use quake_util::qmap::{parse, Writes};

/*
mod test {
    pub struct Bencher {}

    impl Bencher {
        pub fn iter(&self, func: &dyn Fn() -> ()) {
            func();
        }
    }
}
*/

#[cfg(test)]
#[bench]
fn bench_parse(bench: &mut Bencher) {
    bench.iter(|| {
        let f = File::open("test-res/ad_heresp2.map").unwrap();
        let reader = BufReader::new(f);

        let ast = match parse(reader) {
            Ok(ast) => ast,
            Err(err) => panic!("{}", err),
        };

        ast.write_to(&mut sink()).unwrap();
    });
}
