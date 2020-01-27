#[macro_use]
extern crate criterion;

use criterion::Criterion;
use fitsio::*;

use mwalib::fits_read::*;

/// Benchmark the recommended way of reading a key against the custom way
/// derived for mwalib. Use a file located at /tmp/file.fits (use a symlink for
/// convenience), reading NAXIS2 from HDU 2 (N.B. these are zero-indexed in
/// rust).
fn criterion_benchmark(c: &mut Criterion) {
    let mut fptr = FitsFile::open("/tmp/file.fits").unwrap();
    let hdu = fptr.hdu(1).unwrap();

    c.bench_function("read_key from fptr", |b| {
        b.iter(|| {
            fptr.hdu(1)
                .unwrap()
                .read_key::<i64>(&mut fptr, "NAXIS2")
                .unwrap()
        })
    });
    c.bench_function("read_key directly from HDU", |b| {
        b.iter(|| hdu.read_key::<i64>(&mut fptr, "NAXIS2").unwrap())
    });
    c.bench_function("get_fits_key", |b| {
        b.iter(|| get_fits_key::<i64>(&mut fptr, &hdu, "NAXIS2"))
    });
    c.bench_function("get_fits_key with smaller type", |b| {
        b.iter(|| get_fits_key::<u8>(&mut fptr, &hdu, "NAXIS2"))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
