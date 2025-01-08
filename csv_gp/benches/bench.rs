use criterion::{criterion_group, criterion_main, Criterion};

use csv_gp::parser::CSVReader;
use csv_gp::scanner::Scanner;

const DATA: &[u8] = include_bytes!("data/customers-10000.csv");

fn parse_file(c: &mut Criterion) {
    c.bench_function("parse", |b| {
        b.iter(|| {
            let scanner = Scanner::from_reader(DATA, b',');
            let res = CSVReader::from_scanner(scanner);
            assert_eq!(res.into_iter().count(), 10000);
        })
    });
}

criterion_group!(benches, parse_file);
criterion_main!(benches);
