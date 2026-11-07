# Plan: safer and faster reading of visibilities

Status: phase 1 (tests) is done. The MWAX benchmarks from phase 3 are added. The rest of phases 2 to 4 is not started.

Scope: `CorrelatorContext::read_by_baseline`, `read_by_frequency`, `read_weights_by_baseline` and their
`*_into_buffer` versions, the converters in `src/convert/mod.rs`, `get_fine_chan_freqs_hz_array`
(also in `VoltageContext`), and their C and Python wrappers.

Benchmark numbers from one laptop exist (see Benchmarks). Every performance item is still a hypothesis until phase 3
measures it, ideally on more than one machine.

## Decisions (made by Greg)

| # | Decision |
|---|---|
| V1 | A new `GpuboxError` (or voltage error) variant is a minor change. |
| V2 | `get_fine_chan_freqs_hz_array` returns `Result` (breaking), for `CorrelatorContext` and `VoltageContext`. |
| V3 | All Python classes become read-only (breaking). This is every `pyclass(get_all, set_all, ...)`: antenna, baseline, calibration_fit, coarse_channel, convert, correlator_context, gpubox_files (2), metafits_context, rfinput, signal_chain_correction, timestep, voltage_beam, voltage_context, voltage_files (2). |
| V4 | The buffer length check is lenient: a buffer that is too small is an error. A bigger buffer is accepted and only the first part is used. |
| V5 | Threads inside mwalib are opt-in: a runtime setting on the context, default 1 thread, `std::thread::scope`, no new dependency. |
| - | Version is **4.0.0**. The `3.1.0 Unreleased` section of `CHANGELOG.md` becomes `4.0.0`, with a "Breaking" section. `Cargo.toml` moves to 4.0.0 in the phase 2 diff. |
| - | Not doing: caching open FITS handles. `FitsFile` is not `Sync`, so it would make `CorrelatorContext` `!Sync` and block parallel reads. |

## API changes

Rust (breaking):
- `CorrelatorContext::get_fine_chan_freqs_hz_array` and `VoltageContext::get_fine_chan_freqs_hz_array` return
  `Result<Vec<f64>, _>`. Out of range index gives the existing `InvalidCoarseChanIndex`.
- New error variants: buffer too small (expected, got), invalid legacy conversion table, unsupported legacy tile count.
- `read_*_into_buffer` return an error for a buffer that is too small (V4).
- Additive: a scratch buffer type or functions (phase 3), `set_conversion_threads` (phase 4).

C (no struct or signature changes):
- Bad buffer length, bad indices and a bad legacy tile count return `MWALIB_FAILURE` with a message (no abort, no
  partial read).
- Additive (phase 4): `mwalib_correlator_context_set_conversion_threads`.

Python:
- Breaking: all attributes are read-only; the setters are removed from `mwalib.pyi`.
- `get_fine_chan_freqs_hz_array` with a bad index raises `GpuboxErrorInvalidCoarseChanIndex` (voltage equivalent for
  `VoltageContext`), not `PanicException`. `VoltageContext.read_second` with a count of 0 raises
  `VoltageErrorInvalidGpsSecondCount`.
- Additive: optional `out=` argument on the read methods, reads release the GIL, `set_conversion_threads` method.

## Findings

Safety:
1. `convert_legacy_hdu_to_mwax_*_order` use `get_unchecked` (48 places). Only the buffer lengths are asserted. The
   table length and each entry's `xx_index` etc. are not. The table was settable from Python. Fix: validate the table
   once at construction (length equals number of baselines, `index + 2 <= floats_per_fine_chan`), and read-only
   Python (V3) so it cannot change after.
2. `*_into_buffer` do not check the buffer length. MWAX by baseline reads `buffer.len()` floats with `ffgpv`
   (short buffer = silent partial read). Legacy and frequency paths `assert!` (panic; abort with the release profile).
3. `get_fits_filename_and_batch_and_hdu` uses `self.num_timesteps - 1` (underflow if 0) and `num_*` counters instead of
   `.len()`.
4. `get_fine_chan_freqs_hz_array` indexes `self.coarse_chans[*c]` without a check (panic). Same in `VoltageContext`.
   `VoltageContext::read_second` with a count of 0 panics (subtract with overflow, `voltage_context/mod.rs`).
5. Legacy assumes 128 tiles: `generate_conversion_array` has `assert_eq!(rf_inputs.len(), 256)` and the converters use
   `get_baseline_count(128)`. Should be an error at construction. The converters should take the baseline count from
   the table.
6. Python wrappers build the numpy shape with `.expect(...)` from `metafits_context.num_*`, which were settable.
7. C wrapper: `&mut *ptr` for read-only calls (aliasing UB if two C threads read at once; use `&*`). A null
   `buffer_ptr` returns failure without writing the error message.
8. `fits_open_hdu!` moves cfitsio's current HDU, and `ffgpv` reads from the current HDU; the `hdu` argument is only used
   for error text. Make the dependency explicit. Also check that the constructor validates the weights HDU
   (`hdu_index + 1`) dimensions for MWAX v2.

Performance (all unmeasured):
- Python wrappers hold the GIL during the whole read. `CorrelatorContext` is `Send + Sync` (checked by a test).
- Each read allocates about 34 MB (8256 x 128 x 8 x 4 bytes), and `read_by_frequency` and the legacy baseline path
  allocate a second temp buffer. Measured on a laptop: about 28 % of a by baseline read (see Benchmarks).
- `convert_mwax_hdu_to_frequency_order` is a transpose that strides the output by about 264 KB in its inner loop.
  The legacy converters write with a large stride and gather through the table. Both undo and redo a double
  conjugation (`-x` then `*= -1.0`); a precomputed +1.0 / -1.0 per entry would remove the branch and the extra pass.
- Each read opens the file and moves to HDU n. Probably small next to a 34 MB read; measure before changing.
- Check whether cfitsio applies BSCALE/BZERO per element inside `ffgpv`.

## Benchmarks

`benches/correlator_context_benches.rs`:
- Group `mwax` (new): `test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits`, timestep 0, coarse
  channel 10. `read_by_baseline`, `read_by_frequency`, both `_into_buffer`, `read_weights_by_baseline`, and its
  `_into_buffer`. MiB/s is reported. Results are dropped inside the timed code, so allocation is included.
- Group `my_group` (existing): the legacy file. The two `_into_buffer` benchmarks sized their buffer with
  `num_timestep_coarse_chan_bytes` (4 times too many floats); now `num_timestep_coarse_chan_floats`. An oversized buffer
  is also an error for MWAX by baseline today, which is why this mattered.
- Still to add: benchmarks of each converter alone (so conversion time can be separated from the cfitsio read), and a
  file with many HDUs (to see the cost of moving to a late HDU).
- Run: `cargo bench --bench correlator_context_benches -- mwax`. Compare before and after on the same machine. The
  files are read from the OS file cache after the first read, so disk speed is not measured.

Results on Greg's laptop (release profile, middle of the range; the sandbox numbers agreed on the file read but not on
allocation, so they are not kept):

| benchmark | time | MiB/s |
|---|---|---|
| `mwax/read_by_baseline` | 77.1 ms | 418 |
| `mwax/read_by_baseline_into_buffer` | 55.4 ms | 583 |
| `mwax/read_by_frequency` | 99.6 ms | 324 |
| `mwax/read_by_frequency_into_buffer` | 81.3 ms | 397 |
| `mwax/read_weights_by_baseline` | 2.70 ms | 47 |
| `mwax/read_weights_by_baseline_into_buffer` | 2.70 ms | 47 |
| `my_group/read_by_baseline` (legacy) | 108.1 ms | |
| `my_group/read_by_baseline_into_buffer` (legacy) | 90.3 ms | |
| `my_group/read_by_frequency` (legacy) | 99.9 ms | |
| `my_group/read_by_frequency_into_buffer` (legacy) | 83.3 ms | |
| `voltage_context_benches` `mwaxvcs_read_file` | 400 ms | about 100 MB/s |
| `voltage_context_benches` `mwaxvcs_read_second` | 457 ms (400 to 569) | |

(The legacy `_into_buffer` numbers were taken after the buffer size fix. `mwaxvcs_read_second` warned that 10 samples
did not fit in the target time.)

What the numbers say:
- **Allocating the result costs about 22 ms of 77 ms (28 %)** for MWAX by baseline (77.1 against 55.4). Legacy by
  baseline is about 18 ms (108.1 against 90.3), by frequency about 17 to 18 ms. The result is 33.8 MB, just over
  32 MiB. A guess (not measured): glibc then uses a fresh `mmap` and `munmap` for every call, so each read pays about
  8,000 page faults. Check with `perf stat -e page-faults` or by reusing a buffer.
- **`read_by_frequency_into_buffer` is 26 ms slower than `read_by_baseline_into_buffer`** (81.3 against 55.4). It still
  allocates a 34 MB temp buffer on every call, so that 26 ms is the temp allocation plus the transpose. If the
  allocation costs about 20 ms (like the result), the transpose itself is only about 5 ms. Not measured; needs the
  converter-only benchmarks.
- **The cfitsio read is the floor: about 55 ms for 34 MB (583 MiB/s)** with no conversion at all (open, `ffgpv`, byte
  swap, copy). It is the biggest piece left after allocation. Check BSCALE/BZERO handling and the read size.
- **Fixed cost of a read is about 2.7 ms** (the 132 KB weights read: open the file and move to the HDU), about 5 % of a
  data read. A file with many HDUs may cost more (not measured).
- **Voltage reads look slow: `mwaxvcs_read_file` reads about 41 MB in 400 ms, about 5 times slower per byte than a
  correlator read, and `read_second` is very variable.** The voltage code was not read for this plan, so the cause is
  unknown. It is outside this plan; worth its own look.

Before trusting the allocation story, do these first:
1. Converter-only benchmarks (each converter, on an in-memory buffer).
2. A throwaway experiment that reuses the temp buffer in `read_by_frequency_into_buffer`, and see whether it falls
   toward 60 ms.

## Phases

Each phase is one diff. `cargo test` and `uv run pytest` must pass with the diff applied on its own. Outputs of the
converters and the reads must not change (the golden and reference tests enforce this).

1. **Tests (done).** Characterisation tests that pass on the current code and must stay green:
   - `src/convert/test.rs`: every output element of the legacy (both orders) and MWAX converters compared to a
     reference implementation, using unique inputs; both orders are transposes; a bigger output buffer is not touched.
   - `src/correlator_context/test.rs`: golden hashes (legacy and MWAX, all three reads); by-frequency is the exact
     transpose of by-baseline; MWAX by-baseline equals the raw HDU; `*_into_buffer` equals the allocating reads; bigger
     buffers leave the rest alone (not MWAX by baseline); last valid indices; `Send + Sync`; reads from 4 threads;
     fine channel frequencies for every coarse channel.
   - `src/correlator_context/ffi_test.rs`: C read functions: invalid timestep, invalid coarse channel, no data
     (`MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN`) and results equal to the Rust reads.
   - `tests/`: Python golden SHA-256 for correlator reads (legacy and MWAX) and voltage `read_file`; array layout
     (C contiguous, writeable, independent arrays); reads from many threads; boundary indices; fine channel frequencies.
2. **Safety and breaking changes, version 4.0.0.** Findings 1 to 8 above, V1 to V3, V4. Add tests first for each
   (too small buffer, bad index, count 0, 120-tile legacy metafits if one can be made, assignment raises
   `AttributeError`). Update `mwalib.pyi` (`tools/generate_stubs.sh`; see Notes), CHANGELOG, `Cargo.toml` version.
3. **Performance.** MWAX benchmarks are added (see Benchmarks). Order, by what the numbers say:
   1. Add converter-only benchmarks, and the temp buffer experiment above.
   2. Avoid the allocations: optional `out=` in Python, and a reusable scratch buffer for the temp (expected to be the
      biggest win: about 20 ms of 77 ms by baseline, about 40 ms of 100 ms by frequency).
   3. The cfitsio read itself (scaling, read size).
   4. Release the GIL (`py.detach`).
   5. Converter loop order and blocking, and the sign table. Last: the transpose may only be about 5 ms.
   Measure each one on its own, and record numbers here.
4. **Opt-in threads (V5).** `set_conversion_threads(n)`, default 1. Split the output with `chunks_mut` (one fine
   channel or one baseline per chunk) so no `unsafe` is needed. Keep the setting in an atomic so it can be changed
   through `&self` (a `&mut self` setter would conflict with concurrent reads in Python). C function and Python method.
   Check there is no oversubscription surprise: default stays 1.

## Notes for whoever continues

- Workflow: fresh clone of the branch, local changes, give Greg a plain diff, a base64 diff and a commit message
  (first line 50 characters at most). Do not push or touch GitHub.
- Checks: `cargo fmt`, `cargo clippy --all-targets` (also with `--features python,python-stubgen`), `cargo test`,
  `uv run pytest`, `uv run ruff check`, `uv run ty check`. `cargo fmt` also reformats `src/types/test.rs` and
  `src/voltage_context/ffi_test.rs`, which are already not formatted; leave those out of diffs.
- Stubs: `tools/generate_stubs.sh` stops at `ruff check` because of `PYI036` on the generated `__exit__` (now ignored in
  `pyproject.toml`). It then runs `ruff format` at the default line length (88), which would reformat the whole stub.
  Either run the steps by hand (`cargo build --features python,python-stubgen`, `target/debug/stub_gen`,
  `ruff check --fix`) or set `line-length = 120` first.
- Other things seen, not part of this plan:
  - voltage reads are about 5 times slower per byte than correlator reads (see Benchmarks);
  - `examples/mwalib-sum-vcs.py` uses an undefined `t` in `context.timesteps[t]`;
  - the MWAX test metafits `1244973688.metafits` is guessed as `CorrLegacy` when no MWA version is given.
