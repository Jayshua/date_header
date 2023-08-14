use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn parse_imf_fixdate(c: &mut Criterion) {
    c.bench_function("parse_imf_fixdate", |b| {
        b.iter(|| {
            let d = black_box("Sun, 06 Nov 1994 08:49:37 GMT");
            black_box(date_header::parse(d.as_bytes())).unwrap();
        })
    });
}

pub fn parse_rfc850_date(c: &mut Criterion) {
    c.bench_function("parse_rfc850_date", |b| {
        b.iter(|| {
            let d = black_box("Sunday, 06-Nov-94 08:49:37 GMT");
            black_box(date_header::parse(d.as_bytes())).unwrap();
        })
    });
}

pub fn parse_asctime(c: &mut Criterion) {
    c.bench_function("parse_asctime", |b| {
        b.iter(|| {
            let d = black_box("Sun Nov  6 08:49:37 1994");
            black_box(date_header::parse(d.as_bytes())).unwrap();
        })
    });
}

struct BlackBoxWrite;

impl std::fmt::Write for BlackBoxWrite {
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        black_box(s);
        Ok(())
    }
}

pub fn encode_date(c: &mut Criterion) {
    let time = 1691891847;
    let mut buffer = [0u8; 29];
    c.bench_function("encode_date", |b| {
        b.iter(|| {
            black_box(date_header::format(time, &mut buffer)).unwrap();
        });
    });
}

criterion_group!(
    benches,
    parse_imf_fixdate,
    parse_rfc850_date,
    parse_asctime,
    encode_date
);
criterion_main!(benches);
