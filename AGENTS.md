# unit

The rules for this repo on top of the org's (`../AGENTS.md`).

## The gate

`ship ci`: every job in `.config/ship.figl`, in order, as CI runs them.

## Building with corgi

**unit builds under corgi where corgi is installed** (`corgi.toml`,
https://corgi.build): one build store for every checkout on the machine, the
one diaryx builds in, so Blitz and Stylo are built once for both. The lanes
that compile call `scripts/cargo.sh`, which runs corgi where it is on PATH and
cargo where it is not, as on GitHub's runners. Formatting and the docs are
cargo's everywhere. `UNIT_CARGO=1` makes the script use cargo.

- Iterate under corgi too: `corgi test -p unit-gallery --test blitz`,
  `corgi clippy --workspace --all-targets -- -D warnings`. unit has no
  `[roots]`; its two packages resolve together.
- corgi keeps a test pass until something the test read changes, and the
  environment is not one of those things. A run that sets `UNIT_APPEARANCE`,
  `UNIT_DENSITY`, `UNIT_AUDIT` or `UNIT_NO_TIMER` needs `corgi test --force`,
  or it reports the pass of a run without it. `scripts/cargo.sh test` adds
  `--force` itself when any `UNIT_*` variable is set.
- corgi shows a compile only its crate's `.rs` files. A file a build script or
  a test reads is declared under `[extra-inputs]` in `corgi.toml`, or an edit
  to it leaves a stale pass in the store. A native library or a build tool is
  pinned there by hash, and kept to diaryx's, so the two share what they build.
- Never delete or reset the store; `corgi clean` trims it.
