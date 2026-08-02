# CODESTYLE.md

Follow these conventions when writing or modifying code.

## Guiding principles

- **Functional style.** Prefer plain functions over methods where a method
  adds nothing. Prefer producing new values over mutating existing ones.
  When mutation or `&mut` is genuinely needed (e.g. taking ownership out of
  a map to enable zero-copy), justify it in a doc comment.
- **Small, focused modules.** One module per file-format artifact, named
  after it (`calibration_data.rs` decodes `CalibrationData.gpbenc`). The
  module tree mirrors the structure of the format being decoded
  (`parsing/zip/format/...`).
- **Separate decodeing types from domain types.** Raw decode structs (e.g. 
  prost `Zip*` messages) stay private to their module; convert them to clean
  domain types via `From`/`TryFrom` implementations. Ideally, the public API
  exposes only domain types.
- **Single entry point.** The library is entered through
  `SerendipThermogram::new_from_path` / `new_from_bytes`. New capabilities
  hang off the thermogram type; format variants are enum cases.
- **Compute on access.** Derived data (e.g. kelvin temperatures) is computed
  from raw data when requested, not stored at parse time.

## Error handling

- Extraction functions return `Option<T>`: an absent or broken artifact is
  recoverable and must not abort decoding of the rest of the thermogram.
- Log the reason before discarding an error:
  `.inspect_err(|e| warn!("Could not decode {FILE}: {e}")).ok()`.
- Use `?` liberally for early returns on `Option`/`Result`; avoid nested
  `if let` pyramids.

## Naming

- `extract_*` — pull an artifact out of a container (zip file map, records).
- `decode_*` — turn bytes of a known format into a struct.
- `parse_*` — lower-level byte/stream parsing.
- File-path constants at the top of the module that reads them:
  `const CAMERA_INFO_FILE: &str = "CameraInfo.gpbenc";`
- Descriptive full-word names for public items; terse names (`t`, `raw`,
  `buf`) are fine in tight local scopes.

## Documentation

- Every decoding module starts with `//!` docs saying which file it decodes
  and what that file holds.
- Public items get `///` docs. Explain the *why* and the domain — byte
  offsets, units, format quirks — not the mechanics the code already shows.
- Reverse-engineered knowledge is recorded with explicit uncertainty:
  "Presumed Celsius", "Verified on Ti400 samples only", "Blended image
  width? Not IR width". Never present a guess as fact.
- Struct fields carry short trailing `// comments` for presumptions and
  units.

## Logging

- Use the `log` crate. `debug!` traces decoding progress; `warn!` reports
  recoverable decode failures. No `println!`/`eprintln!` in library code —
  those belong to `main.rs` only.

## Testing

- Test against real sample files under `thermograms/`, loaded via
  `concat!(env!("CARGO_MANIFEST_DIR"), "/thermograms/...")`, asserting
  concrete known values from the sample.
- Every extraction function gets at least one happy-path test on a real
  sample and one absence test (`extract_x(&HashMap::new())` yields `None`).
