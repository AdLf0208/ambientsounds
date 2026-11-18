# Licence notice — AmbientSounds for Horizon

This project is a **derivative work** of **AmbientSounds** by **CreativeMD**
(and **AriaFreeze**, as the jar's `fabric.mod.json` lists), and it is
distributed under the **same licence** as the original: **GNU Lesser General
Public License, version 3.0 only (LGPL-3.0-only)**.

## Where the licence was read

- The mod's own manifest, `META-INF/neoforge.mods.toml` inside
  `AmbientSounds_NEOFORGE_v6.3.11_mc26.3.jar`: `license="LGPL-3.0-only"`,
  `authors="CreativeMD"`, `displayName="AmbientSounds"`,
  `displayURL="https://modrinth.com/mod/ambientsounds"`.
- The same jar's `fabric.mod.json`: `"license": "LGPL-3.0-only"`, authors
  `CreativeMD` and `AriaFreeze`, sources
  `https://github.com/CreativeMD/AmbientSounds`.
- The upstream repository `CreativeMD/AmbientSounds` (branch `26.x`,
  commit `e4c8c2df10b51733652b5ac612fb9bd2f72c2326`) carries the same
  `LICENSE` (LGPL-3.0).

## The exact upstream artefact this port was written from

- File: `AmbientSounds_NEOFORGE_v6.3.11_mc26.3.jar`
- Size: 84,817,294 bytes
- SHA-256: `a49c67677aa5964ea5afbfb7613bbc8e01743399560f7b6d405cd2794d43d48d`
- Version 6.3.11 for Minecraft 26.3 (NeoForge).

## The licence texts

Both travel with the port and inside the delivered archive:

- [`LICENSE`](LICENSE) — the LGPL-3.0 text (from
  <https://www.gnu.org/licenses/lgpl-3.0.txt>).
- [`LICENSE-GPL-3.0`](LICENSE-GPL-3.0) — the GPL-3.0 text the LGPL-3.0 refers
  to (from <https://www.gnu.org/licenses/gpl-3.0.txt>).

## What was modified

This is **not** a copy of the mod: it is a clean-room rewrite of the part of
AmbientSounds described in [`notes.md`](notes.md) — the environment engine and
the `cave` and `wind` beds — written in Rust for [Horizon](https://github.com/horizon-rust/horizon)
and injected as a native Horizon mod. No Java class of the original was
translated or copied. The two `.ogg` takes and the two `sounds.json` entries
carried in `content/` are the original's own assets, unchanged.

Changes from the original, all recorded in [`notes.md`](notes.md) and
[`FINDINGS.md`](FINDINGS.md):

- the engine is re-implemented in Rust (`mixins/ambientsounds/runtime.rs`);
- the air-pocket scan, the block-group features, the `length`/`pause`/
  `transition` cross-fades, the pitch curves, the remaining beds and the config
  GUI are **not** carried (the gap list);
- `regions/sky.json`'s `bad-regions: ["end"]` entry is honoured by dimension
  ("Upstream defects fixed" in `notes.md`).

## The corresponding source

The complete corresponding source of this derivative ships with it, as the LGPL
requires:

- the whole `external/ambientsounds/` tree of the porting campaign (the Rust
  sources, the mixin plan, the packaging tools, the content sources and these
  notices), and
- the same `.rs`/`.toml` sources plus the licence texts inside the delivered
  `ambientsounds.hnmod` (see the archive's `mixins/ambientsounds/`).

Anyone redistributing this port must keep it under the LGPL-3.0-only, keep this
notice and both licence texts with it, and make the corresponding source
available.

## No further restriction

This port adds **no** additional terms, no field-of-use limit, no
non-commercial clause and no DRM on top of the LGPL-3.0-only (LGPL-3.0 §3 and
§7). Nothing here is charged for.

## Contact

The original mod: CreativeMD — <https://github.com/CreativeMD/AmbientSounds>.
This derivative: the port lives in the ATM10 → Horizon porting campaign tree at
`external/ambientsounds/`.
