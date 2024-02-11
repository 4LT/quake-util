#[cfg(feature = "std")]
mod bench_util;

#[cfg(feature = "std")]
mod main {

    use benchmarking::measure_function_with_times;
    use std::fs::File;
    use std::io::BufReader;
    use std::time::Duration;

    use quake_util::qmap::parse;

    use crate::bench_util::prepare_file;

    fn parse_file(file_path: &str) {
        let f = File::open(file_path).unwrap();
        let reader = BufReader::new(f);
        let _ = parse(reader).unwrap();
    }

    fn measure_parse(path: &str) -> Duration {
        let path = String::from(path);

        let results = measure_function_with_times(1, move |measurer| {
            measurer.measure(|| {
                parse_file(&path);
            });
        })
        .unwrap();

        results.elapsed()
    }

    pub fn run_benches() {
        let map_names = ["ad_heresp2.map", "standard.map"];
        let maps = map_names
            .iter()
            .map(|&m| (m, prepare_file(m).unwrap()))
            .collect::<Vec<_>>();

        for (map_name, map_path) in maps {
            println!(
                "Took {:?} to parse {}",
                measure_parse(&map_path),
                map_name
            );
        }
    }
}

#[cfg(not(feature = "std"))]
mod main {
    pub fn run_benches() {}
}

fn main() {
    main::run_benches();
}
