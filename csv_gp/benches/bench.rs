use criterion::{criterion_group, criterion_main, Criterion};

use csv_gp::parser::CSVReader;

const DATA: &str = include_str!("data/customers-10000.csv");

fn parse_file(c: &mut Criterion) {
    c.bench_function("parse", |b| {
        b.iter(|| {
            let res = CSVReader::new(std::io::BufReader::new(DATA.as_bytes()), ',');
            assert_eq!(res.into_lines().count(), 10000);
        })
    });

}

criterion_group!(benches, parse_file);
criterion_main!(benches);