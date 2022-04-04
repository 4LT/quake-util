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
mod benchmarks {

    use crate::*;

    fn parse_file(file_name: &str) {
        let f = File::open(file_name).unwrap();
        let reader = BufReader::new(f);
        let ast = parse(reader).unwrap();
        ast.write_to(&mut sink()).unwrap();
    }

    #[bench]
    fn parse_lg_file(bench: &mut Bencher) {
        bench.iter(|| {
            parse_file("test-res/ad_heresp2.map");
        });
    }

    #[bench]
    fn parse_sm_file(bench: &mut Bencher) {
        bench.iter(|| {
            parse_file("test-res/standard.map");
        });
    }
}
