#![feature(test)]

extern crate test;

use std::fs::File;
use std::io::BufReader;
use test::Bencher;

use quake_util::qmap::parse;

mod bench_util;
use bench_util::prepare_file;

#[cfg(test)]
mod benchmarks {

    use crate::*;

    fn parse_file(file_path: &str) {
        let f = File::open(file_path).unwrap();
        let reader = BufReader::new(f);
        let _ = parse(reader).unwrap();
    }

    #[bench]
    fn parse_lg_file(bench: &mut Bencher) {
        let file_path = prepare_file("ad_heresp2.map").unwrap();

        bench.iter(|| {
            parse_file(&file_path);
        });
    }

    #[bench]
    fn parse_sm_file(bench: &mut Bencher) {
        let file_path = prepare_file("standard.map").unwrap();

        bench.iter(|| {
            parse_file(&file_path);
        });
    }
}
