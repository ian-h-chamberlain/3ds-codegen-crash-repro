# crash-repro

Reproduce crash in code generated by rustc for nintendo 3ds.

## Problem 1

## Notes

* Must use 3dsxtool, it crashes on the .elf somewhere in apt code...
* Needs clean build?
  * Since crash is in regex code, probably

* Ugh! fails even when no args used with `cargo rustc`...

Trying with `RUSTFLAGS` instead of `cargo rustc --` to impact other crates

### Passing commands

- [x] `cargo rustc --target armv6k-nintendo-3ds --release -- -C opt-level=1`
- [ ] `RUSTFLAGS="-C opt-level=1" cargo rustc --target armv6k-nintendo-3ds -- -C opt-level=1`

### Failing commands

- `cargo rustc --target armv6k-nintendo-3ds -- -C opt-level=1`
