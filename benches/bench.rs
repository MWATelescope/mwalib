use criterion::*;
use fitsio::*;

use mwalib::*;

/// Benchmark the recommended way of reading a key against the custom way
/// derived for mwalib. Use a file located at /tmp/file.fits (use a symlink for
/// convenience), reading NAXIS2 from HDU 2 (N.B. these are zero-indexed in
/// rust).
fn criterion_benchmark(c: &mut Criterion) {
    let filename = "/tmp/file.fits";
    let mut fptr = fits_open!(&filename).unwrap();
    let hdu = fits_open_hdu!(&mut fptr, 1).unwrap();

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
    c.bench_function("get_required_fits_key", |b| {
        b.iter(|| {
            let i: i64 = get_required_fits_key!(&mut fptr, &hdu, "NAXIS2").unwrap();
            i
        })
    });
    c.bench_function("get_required_fits_key with smaller type", |b| {
        b.iter(|| {
            let i: u8 = get_required_fits_key!(&mut fptr, &hdu, "NAXIS2").unwrap();
            i
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
