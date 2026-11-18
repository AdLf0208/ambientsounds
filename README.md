# AmbientSounds for Horizon — slice 1: the environment engine, the cave bed and the wind bed

A rewrite of [AmbientSounds](https://github.com/CreativeMD/AmbientSounds) 6.3.11
for **Horizon** (Minecraft 26.3, Rust): the mod's own environment engine decides
whether you are underground or in the open, and plays its `cave` or `wind`
ambient bed through the game's music layers instead of its soundtrack.

- **Class** `client-ui` · **tier** `client` · **external** (AmbientSounds is not in ATM10).
- **Licence** LGPL-3.0-only (the mod's own `mods.toml`) — see
  [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md); the port is a derivative under the
  same licence with its complete source.
- **Package** one flat `.hmod` for the SDK mod API 0.2 (`horizon-mod.toml`,
  `Cargo.toml`, `src/`, `assets/`): the code and the two carried `.ogg` takes
  travel together.

## What it does

- Scans the world on the mod's own cadence (`engine.json`):
  - every **40 client ticks**: the terrain columns (11 × 11, 2 blocks apart,
    read from the client's `MOTION_BLOCKING` heightmap), the sky exposure and
    air share of a 5 × 5 × 5 box around the eye, the biome, the weather;
  - every **4 ticks**: the regions' conditions are aimed again;
  - every tick: the bed's volume eases by `fade-volume` (0.005 a tick).
- Underground (dark, deep, covered) the **cave** bed plays, easing in to 0.5.
- Above ground it fades out; in a **thunderstorm** the **wind** bed takes over
  at 0.4; high above the terrain the open-air wind plays too.
- While a bed is audible, vanilla's music is held back (`SelectMusicEvent` gets
  no track and volume 0).
- `/ambientsounds status|engine|volume|rescan` reports what it decided; the
  engine's on/off switch and volume are also two settings on the mod's page in
  Options → Mods.

## What it does not do (yet)

The other 118 `.ogg` files and their beds (animals, water, weather, suspense, the
cave and wind variants), the air-pocket scan and its block-group features, the
cross-fade between takes, the pitch curves and the mod list icon. The full list,
with the SDK surfaces each one would need, is in [`notes.md`](notes.md).

## Install

`ambientsounds.hmod` is one file: the launcher installs it, builds its Rust
against the pinned game build and loads the same package's `cave`/`wind` takes.
It is client-side; publishing it needs the maintainer's approval (copyleft,
rank 1) and nothing here claims that.

## Build and verify

```sh
python3 tools/assemble_content.py --check     # the carried oggs are still the jar's
cargo test                                    # the pure engine and the glue's decisions
cargo clippy --all-targets -- -D warnings
cargo fmt --check                             # not --all: that checks the fetched SDK
rm -f Cargo.lock && cargo horizon-mod build   # -> target/horizon-mod/ambientsounds.hmod
cargo horizon-mod check ambientsounds.hmod
cargo horizon-mod fits ambientsounds.hmod --build nightly
GAME=<launcher-built horizon-client> tools/e2e.sh   # in-game: cave, surface, storm, controls
```

The in-game leg needs a game built by `horizon-launcher native-build` from this
package; this environment has no launcher, so the 0.1.0 run stands as the
behaviour's evidence (see [`notes.md`](notes.md), "Verification").

Evidence, numbers and the decisions behind them: [`notes.md`](notes.md),
[`FINDINGS.md`](FINDINGS.md), [`SOURCE.md`](SOURCE.md).
