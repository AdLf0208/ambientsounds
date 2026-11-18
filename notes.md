# ambientsounds — notes

**Class:** `client-ui` · **Tier:** `client` · **Role:** core · **Port:** external
(AmbientSounds is not in ATM10; its newest build is a native **26.3** NeoForge jar).
**Approach from `horizon/docs/modpacks.md`:** a `client` tier changes what the client hears
and sees — here the mod's own environment engine and two ambient beds played through the
pin's music layers, with vanilla's soundtrack held back. The engine's numbers and the mod's
region data are re-implemented in Rust; only the `.ogg` takes it plays are carried.
**Format:** 0.2.0 is the SDK package (`api = "0.2"`, one flat `.hmod` with `src/` beside
`assets/`), migrated from the retired `.hnmod` on 2026-11-18 — see
[`## Migration to the SDK format`](#migration-to-the-sdk-format-020-2026-11-18), which
supersedes the 0.1.0 bundle and its closed PR **#922**. The 0.1.0 sections below are kept
as the port's history and are marked historical where they no longer describe the code.

## Licence

- **LGPL-3.0-only** (the jar's `META-INF/neoforge.mods.toml`: `license="LGPL-3.0-only"`,
  `authors="CreativeMD"`; `fabric.mod.json` adds **AriaFreeze** and points at
  <https://github.com/CreativeMD/AmbientSounds>). **Rank 1 — copyleft.**
- Obligations, all satisfied in this tree and **inside the delivered `.hmod`** (the archive
  lists `LICENSE`, `LICENSE-GPL-3.0` and `LICENSE-NOTICE.md`):
  1. the **LGPL-3.0** text ([`LICENSE`](LICENSE)) and the **GPL-3.0** text it refers to
     ([`LICENSE-GPL-3.0`](LICENSE-GPL-3.0));
  2. [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md) naming **CreativeMD** (and AriaFreeze), the
     upstream URL, the exact jar and its SHA-256
     (`a49c6767…d43d48d`, 84,817,294 B), and that this port is a **derivative under the same
     licence with its complete source published**;
  3. the modifications are stated (a Rust rewrite on Horizon; the two `.ogg` takes and their
     `sounds.json` entries are the original's own assets, unchanged);
  4. **no further restriction** (LGPL-3.0 §3/§7): no non-commercial clause, no DRM, nothing
     charged for.
- Publishable as a rank-1 port (one repository, one PR per mod), under the same licence, once
  the maintainer asks for it. Nothing here is ARR.

## Slice boundary — what is in, and what is out

**In:** the environment engine at the mod's own cadence (`environment-tick-time` 40,
`sound-tick-time` 4, `fade-volume` 0.005, `average-height-scan-distance` 2 /
`-count` 5, `biome-scan-distance` 5 / `-count` 3), the regions that decide this slice
(`cave`, `cave.ambience`, `surface`, `sky`, `storm.close`), the **cave** bed (take
`cave.cave1`) and the **wind** bed (take `light-wind`), the volume easing, the mute ducking,
the vanilla-music suppression, the embedded content (one `.hnmod` ships the behaviour and the
audio), and `/ambientsounds status|engine|volume|rescan` as the harness surface.

**Out (gaps):**

- **118 of the 120 `.ogg` files** (85.5 MB of the jar's 84.8 MB uncompressed): every animal
  bed, `water`, `weather.rain*`/`storm-away`, `suspense`, and the cave/wind variants
  (`cave.cave2/3`, `cave.cave-water*`, `cave.cave-deep*`, `cave.cave-large*`,
  `cave.cave-lush`, `cave.deep-dark`, `cave.underworld`, `wind.light-wind2`,
  `wind.heavy-wind*`, `wind.howling-wind`, `wind.wind-in-leaves`, `wind.mesa`). The two beds
  are carried with **one take each** (2,055,326 B of the jar's 85.5 MB of audio).
- **The air-pocket scan** (`AirPocketScanner`, `block-scan-distance` 40, `air-pocket-count`
  50000, the weighted distance groups) and its **block groups / features** (`rock`, `lush`,
  `dripstone`, `sculk`, `grass`, `sand`, …): the port measures a 5 × 5 × 5 box around the eye
  and drops the `features` condition, so the cave ambience plays in every cave instead of
  only in rocky/lush/dripstone/sculk ones.
- **The `length` / `pause` / `transition` cross-fade** between takes (the mod plays a take
  for 20-35 s, then crosses to another): the pin's music layers loop one event per layer, so
  the port loops one take.
- **Pitch**: `fade-pitch`, `pitch`, `underwater-pitch` (the layer API carries no pitch).
- **The config GUI and the mod list icon** (`CreativeConfig` folders per dimension, region,
  sound and category, `ambientsounds.png`): replaced by `/ambientsounds volume` and
  `/ambientsounds engine`.
- **The other regions and dimensions**: `rain` (plain rain has no bed here), `cold`,
  `forest*`, `jungle`, `desert`, `ocean*`, `swamp`, `taiga`, `mesa`, `highland`, `beach`,
  `mushroom_fields`, `pale-garden`, `warden`, `cave.deep-dark`, `cave.lush`, `underworld`,
  `sky`'s other collections, the `nether`/`end`/`void`/`cave` dimension beds, `biome-types`
  (`surface`/`underground` selectors) and the SereneSeasons temperature hook.
- **The cave's water/feature-driven sounds** and `cave-large`'s air condition.
- **The `end`/`nether` dimension beds** (the `suspense` and `unknown` regions).

## Upstream files read

- **Jar** `AmbientSounds_NEOFORGE_v6.3.11_mc26.3.jar` (84,817,294 B, sha256 `a49c6767…`):
  `META-INF/neoforge.mods.toml`, `fabric.mod.json`, `assets/ambientsounds/config.json`,
  `basic/engine.json`, `basic/regions/{cave,cave.ambience,cave.deep-dark,surface,sky,rain,
  storm.close,storm.away,underworld}.json`, `basic/sound_collections/wind.json`,
  `basic/sound_categories/{cave,wind,weather}.json`, `basic/dimensions/{end,nether,cave,
  surface,void}.json`, `basic/features/*.json`, `basic/blockgroups/*.json`,
  `sounds.json` (87 events), the 120 `.ogg` names, and the 66 `.class` names (read as a
  list only — no decompilation).
- **Repo** `CreativeMD/AmbientSounds`, branch `26.x`, commit
  `e4c8c2df10b51733652b5ac612fb9bd2f72c2326` (`gradle.properties`: `mod_version=6.3.11`),
  read for behaviour: `engine/AmbientEngine.java`, `engine/AmbientTickHandler.java`,
  `environment/AmbientEnvironment.java`, `environment/TerrainEnvironment.java`,
  `environment/BiomeEnvironment.java`, `environment/pocket/AirPocket.java`,
  `condition/AmbientCondition.java`, `condition/AmbientSelection.java`,
  `condition/AmbientVolume.java`, `condition/BiomeCondition.java`,
  `region/AmbientRegion.java`, `dimension/AmbientDimension.java`,
  `sound/AmbientSound.java`, `sound/AmbientSoundCategory.java`,
  `sound/AmbientSoundEngine.java`, `sound/AmbientSoundProperties.java`,
  `sound/AmbientSoundCollection.java`, `block/AmbientBlockGroup.java`. The clone was deleted
  after [`FINDINGS.md`](FINDINGS.md) was written; the jar stays as the licence evidence.

## The hooks of the 0.1.0 bundle (historical)

*The retired `.hnmod`'s plan, kept for reading the old tree. The 0.2.0 package's typed
`Hooks` methods are in the migration section below.*

| Hook | Version | What it does here |
|---|---|---|
| `music_layers` | 1 (API 119) | the engine's client tick: the 40/4/1 cadence, the scan, the aim, the eased volume, and the beds as looping layers |
| `music` | 1 (API 119) | `SelectMusicEvent`: a bed audible → no track, volume 0 |
| `client_level_load` | 1 (API 135) | a level made or changed → the scan starts over |
| `content_root` | (target injection) | writes the embedded `.hmod` out once and appends it to the dev list |
| `register_commands` / `perform_command` | 1 (API 8) | `/ambientsounds status|engine|volume|rescan` |

**Semantic differences from NeoForge.** The mod runs on `ClientTickEvent` and reads the
client level directly; the pin gives a client tick through `music_layers` (asked once each
client tick, just before the music manager ticks its layers), with the world read through
`native_player`, `native_light`, `native_block`, `native_biome`, `native_weather` and
`native_world_height`. Two consequences: the beds are **music layers** (the game's music
source, looping, with the manager's own 2 s easing on top of the mod's 10 s fade) rather than
`AMBIENT`-source instances the mod starts and stops itself; and the "hold vanilla's music
back" is `SelectMusicEvent` returning `None`, exactly as NeoForge's event intends, instead of
the mod stopping streams.

## How the environment is read (0.1.0, historical)

*The 0.1.0 glue read the world through `Game`'s `native_*` calls. The 0.2.0 glue reads the
same numbers through the SDK's client hosts — the table in the migration section is the
current one.*

| Upstream | Here (0.1.0) |
|---|---|
| `TerrainEnvironment.getHeightBlock`: highest solid/leaves/water block, top-down | the column's sky light: under open sky the first dark block below the eye (exact, one block at a time); underground a stride-8 scan from the world top, refined (a sealed room's own roof is not mistaken for the surface) |
| `average-height-scan-distance` 2 × `-count` 5 → 11 × 11 columns | the same 121 columns, same spacing |
| the air pocket's `averageSkyLight`, `sky` (20…100 count → 0…1), `air` (0.01…0.07) over a 40-block scan | a 5 × 5 × 5 box around the eye: the mean sky light of its air, the share of that air open to the sky (sky light 15), the share of the box that is air - all 0…1, so the curves' ends differ from the mod's counts (a gap, recorded above) |
| `Level.isRainingAt(pos.above())` | rain level > 0 from `native_weather` and the eye's sky light ≥ 15 |
| the 7 × 7 biome scan, distance-weighted, for `bad-biomes` and `rainSurfaceVolume` | the eye's biome plus the four corners of the biome scan box; a `bad-biomes` match anywhere refuses the cave bed |
| `AmbientDimension.is` over `level.dimension().identifier()` | `native_player().dimension` |

## Upstream defects fixed

**`regions/sky.json` names a region that cannot exist.** The file's
`"bad-regions": ["end"]` (jar path `assets/ambientsounds/basic/regions/sky.json`) means "no
sky wind in the End", but `regions/` holds no `end` region — the End's bed is a *dimension*
(`dimensions/end.json`, whose region is `unknown`). `AmbientCondition.init`
(`condition/AmbientCondition.java:113-121`) resolves `bad-regions` through
`engine.getRegion(name)` and silently drops an unknown name (`if (region != null …)`), and
`AmbientEngine.addRegion` (`engine/AmbientEngine.java:334-346`) only ever registers
`<dimension>.<region>` for a dimension's own regions, so `"end"` can never resolve: the
condition is dead, and upstream plays the **open-air wind in the End** at any height above
the island.

- **Fix:** the port honours the entry by dimension — `Environment::sky_region` returns 0 when
  the level id contains `the_end`, which is the only reading that makes the data's intent
  true.
- **Compatible where cheap:** the region-name path is still what the data says; a later region
  literally named `end` would be honoured the same way.
- **What a user notices:** the End is silent instead of windy — which is also what makes the
  "environment with no bed" control in the e2e a real environment.
- **Worth an upstream report:** yes, a one-line data fix (`bad-regions: ["end"]` → a dimension
  field), but it is not filed: the campaign asks for reports only when it asks.

## Deviations (deliberate, not defects)

- Switching the engine off lets the current bed **fade out** (0.005 a tick) instead of cutting
  it: the game's own sound engine stops every sound on a level change anyway, and a 5 s fade
  is what a player expects from a volume switch.
- The game's `AMBIENT` source volume is not re-checked before scanning: the mixer multiplies
  the source volume into every instance's gain, so a player at 0 hears nothing either way.
  Upstream's early `getSoundSourceVolume(AMBIENT) > 0` return is an optimisation, not a rule.
- The port plays the two beds as `music`-source layers, so the game's Music slider and the
  Music page's per-track off switch apply to them (upstream's sounds are `ambient`-source).
  The content's `sounds.json` keeps the mod's own `"category": "ambient"` entry.

## Verification (0.1.0, historical)

*The retired `.hnmod`'s ladder at pin `1d63e0a`. The 0.2.0 ladder is in the migration
section; the logs below stay as the behaviour's in-game evidence, because `src/runtime.rs`
did not change.*

Ladder (pin `1d63e0a`; the remote build slot was used, `CARGO_INCREMENTAL=0`):

| Step | Command | Result |
|---|---|---|
| runtime tests (red first) | `cargo test --locked --offline` | 34 failed (`unimplemented!`) → **34 passed** |
| fmt | `rustfmt --check --edition 2024 mixins/ambientsounds/*.rs` | clean |
| clippy | `cargo clippy --all-targets --locked --offline -- -D warnings` | clean |
| staged check | `python3 tools/horizon.py check` | `cargo_exit 0`, 0 warnings |
| embedded tests | `python3 tools/horizon.py test` (as `game::ambientsounds::tests`) | **34 passed**, 0 warnings |
| build | `python3 tools/horizon.py build` (fast) | `cargo_exit 0` |
| content | `cargo horizon-mod build` + `check` | 2,039,817 B, 4 entries, `no code: content only` |
| bundle | `cargo horizon-mod check ambientsounds.hnmod --kind native-mod` | accepted |

Deliverables (both in the port, never `target/`):

- `content/ambientsounds.hmod` — 2,039,817 B, sha256
  `6a8d502978bda1c94b8bbb318ef8d7afe0744236f2055eb61b89405b71cad630`, 4 entries
  (`horizon-mod.toml`, `assets/ambientsounds/sounds.json`, the two `.ogg` takes).
- `ambientsounds.hnmod` — 2,076,794 B, sha256
  `f53345f4f0d5994b29dd0d4d313765a2e7542d54e3704db706011a565f012593`, 14 entries:
  `horizon-native.json`, `mixins/ambientsounds/{horizon-mixin.toml, runtime.rs, support.rs,
  music.rs, music_layers.rs, client_level_load.rs, register_commands.rs, perform_command.rs,
  embedded_content.rs}`, `mixins/ambientsounds/data/ambientsounds-content.hmod`, and the
  three licence files (`LICENSE`, `LICENSE-GPL-3.0`, `LICENSE-NOTICE.md`). Rebuilding
  reproduces the same bytes (`tools/package.py` writes fixed dates and modes).

In-game, `tools/e2e.sh` (headless, scratch home, The End and an Overworld arena; logs in
`ports/_shared/evidence/ambientsounds/`):

| Leg | Log | Verdict |
|---|---|---|
| `--dev-mod content`: cave → surface → thunderstorm → the End → engine off/on | `e2e.log` | `PASS` + `EXIT:0` |
| `EMBEDDED=1` (no `--dev-mod`, the bundle's own content) + its no-content control | `e2e-embedded.log` | `PASS` + `EXIT:0` |
| `SELFTEST=1` (the cave assertion flipped) | `e2e-selftest.log` | `FAIL: bed=cave, expected wind` + `EXIT:1` |

What the default leg showed, each number from `/ambientsounds status` and each bed from the
pin's own state:

- underground in a sealed room (sky light 0, `relative=-34.6`): `region_cave=1.000`,
  `region_ambience=1.000`, `bed=cave`, `current` 0.280 → 0.415 over 20 ticks → **0.500**, the
  pin's `MusicLayers` holding `ambientsounds:cave.cave1` at gain 0.500, `vanilla=held`,
  `music_held=1`, `music_received=minecraft:music.overworld.forest`;
- on a 21 × 21 platform at y=200 (sky light 15, `relative=2.0`): `region_cave=0.000`,
  `region_surface=1.000`, `aimed=0.000`, the bed fading 0.087 → 0.000 in 20 ticks and the
  layer dropped, `vanilla=plays`;
- `/weather thunder`: `rain=1.00 thunder=1.00`, `region_storm=1.000`, `bed=wind`,
  `wind=0.400`, the layer at gain 0.400, `vanilla=held`;
- the End: `dimension=minecraft:the_end`, `region_cave=0.000`, `region_sky=0.000` (the fix
  above), `bed=none`, no layer;
- `/ambientsounds engine off` in the same cave: `engine=off`, `aimed=0.000`, the bed fading
  to 0.000, no layer, `vanilla=plays`; `/ambientsounds engine on`: `bed=cave` at 0.500 again;
- `EMBEDDED=1`: the bundle writes `.horizon-cache/ambientsounds-embedded-content.hmod`,
  byte-identical to `content/ambientsounds.hmod` (2,039,817 B), and the same cave leg passes;
  its control (`.horizon-cache` a plain file) has the port still asking for the cave bed while
  `registry_cave=0` — nothing would sound.

The `layers=`, `registry_cave=`/`registry_wind=` and `EMBEDDED=` pieces of that readout belong
to 0.1.0: the SDK has no read of the music manager's live layers and no sound-registry query,
and code and content are one package now, so the 0.2.0 status line reports the layers the port
pushed itself and carries no `registry_*` fields (migration section, trap 8 and the gaps).

### Verification limits (measured, not assumed)

- **This box has no audio device and the prepared 26.3 resources carry no vanilla sounds**
  (`assets/minecraft/sounds.json` is absent on both build machines). `SoundEngine.loaded` is
  set from the opened output (`crates/horizon-client-render/src/audio.rs:1043`), so it is
  false here and the pin starts no instance at all: `horizon-ctl status`'s
  `sounds.events` is empty for *every* sound, vanilla ones included. The log prints that
  census (`(mixer census with no audio device: [])`) instead of hiding it, and the bed
  evidence is the pin's **`MusicLayers.playing()`** (the manager's layer and the gain it is
  applying) plus the pin's **`SoundRegistry`** resolving each carried event to its `.ogg`
  (`registry_cave=1`, `registry_wind=1`; `0` without the embedded content). A box with an
  audio device would show the same run in `sounds.events`; that is not claimed here.
- Because no vanilla track can start locally, `status.music` stays null in both states
  (`held` and `plays`); the suppression is proven by the `music` hook's own input and output
  (`music_received=minecraft:music.overworld.forest`, `music_held=1`) and by
  `suppresses_vanilla`'s unit tests.
- The environment read is the client's copy of the world (as upstream's is), and a scan is up
  to 40 ticks old; the e2e waits a full scan (60 ticks) after every move.

## Files

- `horizon-mod.toml` — the only manifest and the only identity (`id`, `name`,
  `version = "0.2.0"`, `api = "0.2"`, `side`, `capabilities`, `[native]` pins).
- `Cargo.toml` — one crate, `edition = "2024"`; its only dependency is `horizon-mod-api`
  from `.horizon/sdk/`.
- `src/lib.rs` — the one `#[mod_api::horizon_mod] impl Hooks`: `register` and the five hooks.
- `src/support.rs` — all the game glue: the scan, the layers, the music hold, the settings and
  the commands, plus 6 unit tests of the two decisions that can be checked without a game.
- `src/runtime.rs` — the pure engine (conditions, terrain, cadence, easing, ducking), 34 unit
  tests, **byte-identical** to the retired `mixins/ambientsounds/runtime.rs`.
- `assets/ambientsounds/` — the two `.ogg` takes, their `sounds.json` and the settings' lang
  entries.
- `LICENSE`, `LICENSE-GPL-3.0`, `LICENSE-NOTICE.md` — inside the package.
- `README.md`, `SOURCE.md`, `FINDINGS.md`, this file, `tools/{e2e.sh,assemble_content.py}`.
- `ambientsounds.hmod` — the deliverable, 2,081,784 B, sha256
  `39ce7931723ec5a7910beac42a120506bc2f9dcf92cc1383bf5f9086ab61c6b4`.
- Evidence: `../_shared/evidence/ambientsounds/` (`sdk-*.log`, `sdk-uses.json`,
  `sdk-uses-summary.txt`, `sdk-sha256.txt`, `sdk-runtime-sha256.txt`, and the retired 0.1.0
  tree under `retired-hnmod-0.1.0/`).

## Migration to the SDK format (0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`: the registry
closed every `.hnmod` submission on 2026-11-18 with one reason, and an `.hnmod` no longer
loads. This port's former submission, **PR #922** (`ambientsounds` 0.1.0), was closed by that
purge. **0.2.0** is the same behaviour republished as one flat SDK `.hmod`; the changelog text
is below.

Pins: SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18,
`package_format` 6, `content_api` `"0.2"`), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (the mixins commit **moved**:
`5a64af93…` is the retired bundle's), launcher 0.2.78. The SDK is cached in-tree at
`../_shared/sdk/0b14f8a/` and was copied into `.horizon/sdk/` (gitignored) for this build.

### What moved, and where the retired tree is

Everything below was **moved**, not copied, before the first `cargo horizon-mod build` (trap
1), into `ports/_shared/evidence/ambientsounds/retired-hnmod-0.1.0/`:

`mixins/` (the plan, `runtime.rs`, `support.rs` and the 6 per-hook helpers), `ambientsounds.hnmod`,
`compatibility.json`, `rust-toolchain.toml`, `content/` (its own `horizon-mod.toml`, `assets/`,
`SOURCE.md` and the built `content/ambientsounds.hmod`), `tools/horizon.py`,
`tools/package.py`, `tools/package.sh`, and `Cargo.lock` (deleted: it is packed and changes
the artifact, trap 5).

| 0.1.0 path | 0.2.0 |
|---|---|
| `mixins/ambientsounds/runtime.rs` | `src/runtime.rs`, `cp` + `cmp`-checked, **byte-identical** |
| `mixins/ambientsounds/support.rs` + 6 hook helpers | `src/support.rs` + `src/lib.rs` |
| `mixins/ambientsounds/horizon-mixin.toml` (2 `[[module]]`, 6 `[[injection]]`) | no plan: 6 typed `Hooks` methods and one `Registrar` declaration |
| `mixins/ambientsounds/data/ambientsounds-content.hmod` + `embedded_content.rs` | gone: the package carries `assets/` itself |
| `content/assets/ambientsounds/**` | `assets/ambientsounds/**`, byte for byte — `sounds.json` 411 B `09190a7f…2b0d`, `cave/cave1.ogg` 978,787 B `d98069e1…bda1`, `wind/light-wind.ogg` 1,076,539 B `0cce9d58…2b7d`, each **unchanged** against the retired copy |
| `content/horizon-mod.toml` | merged into the root `horizon-mod.toml` + `[native]` |
| `content/ambientsounds.hmod` (2,039,817 B) | not needed: content is inside the one package |
| `ambientsounds.hnmod` (2,076,794 B) | `ambientsounds.hmod` (2,081,784 B, native + content) |
| `compatibility.json`, `rust-toolchain.toml` | `[native]` in the manifest; the pinned SDK decides the toolchain |
| `tools/horizon.py`, `tools/package.py`, `tools/package.sh` | deleted (staging and `.hnmod` packaging belonged to the retired format) |
| `Cargo.lock` | not kept |

**Kept and updated:** `tools/e2e.sh` (now GAME=/CONTROL_GAME=, no `--dev-mod`),
`tools/assemble_content.py` (writes `assets/` at the package root and still verifies against
the jar), `notes.md`, `FINDINGS.md`, `SOURCE.md`, `README.md`, the three licence files.

### Helper → hook table

| 0.1.0 injection (hook, API) | 0.2.0 |
|---|---|
| `music-layers` (`music_layers` 1, API 119) | `Hooks::music_layers` → `support::music_layers` (unchanged job: the cadence, the scan, the aim, the layers) |
| `music` (`music` 1, API 119) | `Hooks::music` → `support::music`; the event's `volume` is now a **value** (`event.volume = 0.0`), not a `&mut f32` |
| `client-level-load` (`client_level_load` 1, API 135) | `Hooks::client_level_load` → `support::client_level_load` (`engine().reset()`) |
| `register-commands` (`register_commands` 1, API 8) | `Hooks::register_commands` → `support::declare_commands` (typed `Command` usages) |
| `perform-command` (`perform_command` 1, API 8) | `Hooks::run_mod_command` → `support::run_command` (typed `CommandInvocation`, `Flow::Cancel`) |
| `embedded-content` (`content_root`, AE2 pattern) | **none**: the flat `.hmod` carries `assets/` and `src/` together |
| `[[module]] runtime` | `src/runtime.rs` + `pub mod runtime;` |
| `[[module]] state` | `src/support.rs` + `pub mod support;` |
| — (the settings) | `Hooks::register` → `Registrar::register_setting` twice (`engine` toggle, `volume` range 0..1 step 0.01), which is also what makes `Host::setting` readable |

### How the environment is read now

*The current table; the 0.1.0 one above is historical.*

| Upstream | 0.2.0 |
|---|---|
| `TerrainEnvironment.getHeightBlock`: top-down, the highest `isSolidRender() \|\| leaves \|\| water` block | the client's own **`MOTION_BLOCKING` heightmap** for the same 121 columns, in one `client_surface_heights` call (minus one: `getHeight` answers one above the block). More faithful than 0.1.0's sky-light walk and exact per column; the one difference is that a fence, a slab or another non-full block that still blocks motion counts here and does not for `isSolidRender` (at most a block on a handful of columns) |
| `average-height-scan-distance` 2 × `-count` 5 → 11 × 11 | the same 121 columns, same spacing, unloaded columns skipped (the heightmap answers `None` there) |
| the air pocket's `averageSkyLight`, `sky`, `air` over a 40-block scan | a 5 × 5 × 5 box: a cell counts as air (`level.block`), and **open to the sky** when its `y` is at or above its column's heightmap value; `sky_light` is 15 over open cells and 0 over covered ones, `sky` is the share of the air open, `air` the share of the box that is air. The one difference from 0.1.0: the SDK has no client-side light read (trap 6), so "sky light" is now "open to the sky" rather than the light engine's stored value |
| `Level.isRainingAt(pos.above())` | `level.weather` (both sides) with `sky_light >= 15`, which the rule above makes exactly "open to the sky" |
| the 7 × 7 biome scan | the eye's biome plus four corners (`level.client_biome`), a `bad-biomes` match anywhere refusing the cave bed |
| `AmbientDimension.is` | `player.dimension(host)` → the id string (`minecraft:the_end` still silences the wind) |
| the eye at `EntityDimensions` 1.62 | `entity.eye_position(host)`, the game's own eye (sneaking and crawling included) |

### The ladder (measured, every step inside a build slot, `CARGO_INCREMENTAL=0`)

`build-slot.sh` still speaks the retired tool, so it is used as the **slot guard** (job
`ambientsounds-sdk`, `BUILD_LOCAL=1`), not as the driver: there is no stage, no staged tree and
no `--target-dir`. The commands run in the port itself.

| Step | Command | Result |
|---|---|---|
| runtime only, before any glue | `cargo test --offline` (`pub mod runtime;` alone) | **34 passed**, 0 failed — `evidence/ambientsounds/sdk-test-runtime.log` |
| glue first red | the same, with the new helpers `todo!()` | **6 failed** (`not yet implemented`), 34 passed — `sdk-test-red.log` |
| tests | `cargo test --offline` | **40 passed**, 0 failed (34 `runtime::`, 6 `support::`) — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` (**not** `--all`: that checks the fetched SDK) | exit 0, empty diff — `sdk-fmt.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `SDK 0b14f8a … in ./.horizon/sdk` / `Built ./target/horizon-mod/ambientsounds.hmod` — `sdk-build.log` |
| registry check | `cargo horizon-mod check ambientsounds.hmod` | `ambientsounds 0.2.0 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)` / `2081784 bytes, SHA-256 39ce7931…c6b4, unsigned`, **exit 0** — `sdk-check.log` |
| machine form | `cargo horizon-mod check … --json` | `{"ok":true,…,"kind":"native-mod","sha256":"39ce7931…","size":2081784,"version":"0.2.0"}` — `sdk-check-json.log` |
| what it uses | `cargo horizon-mod check … --uses` | exit 0 — `sdk-check-uses.log`, `sdk-uses.json` |
| fits the pinned build | `cargo horizon-mod fits ambientsounds.hmod --build nightly` | `AmbientSounds:` / `0b14f8a  made for`, exit 0 — `sdk-fits.log` |
| same package twice | `cargo horizon-mod build` again | 2,081,784 B and `39ce7931…c6b4` both times — `sdk-build-second.log`, `sdk-sha256.txt` |
| `runtime.rs` untouched | `cmp` with the retired file **and** `unzip -p ambientsounds.hmod src/runtime.rs \| sha256sum` | `df775ccf716d7d3d8d8161935e84977e0fa3d60312e447989da233fac4c94d06` both — `sdk-runtime-sha256.txt` |
| content carry unchanged | `python3 tools/assemble_content.py --check`, then size+sha256 against the retired `content/` | `assets: 2 .ogg file(s) + sounds.json, identical to the jar`; all three **unchanged** |

The archive is 14 entries: `horizon-mod.toml`, `Cargo.toml`, `LICENSE`,
`LICENSE-GPL-3.0`, `LICENSE-NOTICE.md`, `README.md`, `assets/ambientsounds/lang/en_us.json`,
`assets/ambientsounds/sounds.json`, the two `.ogg` takes, `src/{lib,runtime,support}.rs` and
`uses.json`. **No `Cargo.lock`** (trap 5). `SOURCE.md`, `FINDINGS.md` and this `notes.md` are
root documents the packer does not carry (the SDK packer's fixed list); they stay in the tree.

`uses.json` is a **lexical scan of the source**, not a subscription list: 165 entries —
14 `hook:`, 22 `method:`, 38 `handle:`, 72 `type:`, 15 `trait:`, 2 `part:`, 1 `capability:` and
1 `macro:` — including noise from words that happen to match API names (`hook:command` and
`hook:tooltip` come from `Command`/`.tooltip(`, `part:block`/`part:entity` from
`BlockKey`/`EntityRef`). The real surface: `Hooks::{register, music, music_layers,
client_level_load, register_commands, run_mod_command}`; the host reads
`client_surface_heights`, `local_player`, `setting`/`set_setting`, `commands().feedback`,
`level::{block, client_biome, is_loaded, weather}`, `player::{entity, dimension}`,
`entity.eye_position`, `block.id`; `Registrar::register_setting`; the `registries` capability
(the schema's one value).

### Traps this migration hit (the next client-side port must know)

1. **The retired paths refuse the package.** `cargo horizon-mod build` accepts only package
   paths; `mixins/`, the `.hnmod`, `content/`, `compatibility.json` and the old tools all moved
   to the evidence folder **before** the first build (the same wall `naturesaura`'s
   `tools/audit/` hit).
2. **`--port` is a label and the runner does not `cd`.** `bash -c 'cd <abs> && …'` is the
   working shape; a relative `cd external/ambientsounds` fails, and the forwarding attempt
   then falls back to a local slot. `BUILD_LOCAL=1` makes the SDK ladder explicit.
3. **`cargo test --offline` works** here for this crate (40 registry crates, all cached) and
   writes a `Cargo.lock` — remove it right before the build, never pack it.
4. **`| tee` hides the exit code from `build-slot.sh`.** The log's `test result:` line and the
   explicit `echo <step> exit:$?` are the evidence; the wrapper's exit 0 is not.
5. **`cargo fmt --all` would report the SDK's own files** and `cargo fmt` could rewrite
   `runtime.rs`; format the two glue files explicitly (`rustfmt --edition 2024 src/{lib,support}.rs`),
   then `cargo fmt --check`, then `cmp` the runtime again.
6. **There is no client-side light read in the SDK.** `level.brightness(host, LightLayer::Sky,
   pos)` (and `level.can_see_sky`, `level.is_raining_at`) are **server-side** reads, documented
   as such in `host/levels.rs`; the general notes' sentence about "the same way as the block
   read" sits in the *server levels* paragraph. So the cave/sky decision is derived from the
   client's `MOTION_BLOCKING` heightmap instead (the exact fix is in the table above), and a
   port that needs a *gradient* of sky light on a client is `blocked(no-client-light-read)`.
7. **`client_surface_heights` lives on `ClientHost`, not on `LevelRef`** — it takes the columns
   and the `Heightmap`, answers one value per column and `None` where this client holds no
   chunk, and is `#[access(WorldOnly)]` (fine: the music hooks run in `ModPhase::World`).
8. **The SDK has no read of the music manager's live layers and no sound-registry query.**
   `MusicLayers::playing()` and `game.audio.engine().registry` do not exist in the mod
   surface, so `layers=` now reports the volumes the port itself pushed, and
   `registry_cave=`/`registry_wind=` are gone (with the embedded-content control they
   distinguished: code and content are one package now). The e2e's assertions follow.
9. **`Host::setting` answers a registered key only.** The 0.1.0 glue read two keys nothing had
   declared; at 0.2 they must be declared with `Registrar::register_setting` before
   `Host::setting`/`set_setting` serve them. Registering them also gives the mod a real page
   in Options > Mods — the config GUI the notes list as a gap, partly closed.
10. **A stored toggle is `"true"`/`"false"`, not the old `"on"`/`"off"`.** The port accepts
    both spellings (trap-compatible read) and writes the typed one.
11. **The music event's volume is a value.** 0.1.0 took `&mut f32`; the SDK's
    `MusicEvent { music: &mut Option<Music>, volume: f32 }` is applied back by the dispatcher,
    so the glue sets `event.volume = 0.0`.
12. **`perform_command` is gone and `run_mod_command` only fires on a *parsed* usage.** The
    0.1.0 handler read a trailing `status <tag>` off the raw line while its declared usages had
    no such argument; at 0.2 an undeclared word never reaches the handler, so the usage is now
    `ambientsounds status [tag:word]` (the readout's `seq=<tag>` is unchanged and the e2e keeps
    using it). `engine <state:word>` and `volume <value:word>` stay `word` so every spelling
    still parses.
13. **The command runs on the server, the scan needs the client.** `/ambientsounds rescan`
    cannot re-read the level from `ServerHost`; it sets a flag the next client tick honours
    (reset + scan) and answers `AmbientSounds: rescan requested` instead of the full status
    line.
14. **`CommandInvocation` carries `literals` and named `arguments`.** Rebuild the old word
    list from `literals` (`["ambientsounds", "engine"]`) plus `argument("state")`, and match
    `CommandArgumentValue::String(_)` with a wildcard (`#[non_exhaustive]`).
15. **A lang file is needed for the settings page.** `SettingSpec`'s label/tooltip are
    translation keys; `assets/ambientsounds/lang/en_us.json` defines them (new file, not from
    the jar — recorded in `SOURCE.md` and the notes).
16. **`side` is the *installation* side.** `both`: the engine is client-side, the command and
    the setting writes are server-side of the same installation, and single player's
    integrated server needs the package.
17. **No in-game leg at 0.2.0.** `horizon-launcher` is not installed here, so no game carrying
    this package exists; `tools/e2e.sh` now takes `GAME=` (a launcher-built binary) and an
    optional `CONTROL_GAME=`, and the 0.1.0 logs are historical.

### Gaps after the migration

Everything in the 0.1.0 gap list above still stands. Added or changed by this format:

1. **No 0.2.0 in-game run.** The e2e needs a game built by
   `horizon-launcher native-build <port> --instance <i> --trust-native-code`, which this
   environment does not have. The 0.1.0 runs remain the behaviour's evidence
   (`evidence/ambientsounds/{e2e,e2e-embedded,e2e-selftest}.log`): what changed is the
   *transport* of the same calls and the four readout lines named above, while `src/runtime.rs`
   — the rules — is byte-identical and tested.
2. **Sky light is now "open to the sky"** (trap 6): a client-side light read does not exist in
   the SDK, so a cave with a side opening that the light engine would brighten reads 0 here.
   The two carried regions only need "covered" and "open", which the heightmap answers exactly.
3. **`layers=` is the port's own record**, not the music manager's applied gain (the manager
   still eases on top, up to 2 s, as before), and `registry_cave`/`registry_wind` are gone
   (trap 8).
4. **`/ambientsounds rescan` answers a request, not a fresh status line** (trap 13).
5. **The Options > Mods page is new** and only partly replaces the mod's config GUI: engine
   on/off and volume are there; the per-dimension/per-region/per-sound configuration is not.
   The registration call compiles and the package passes `check`, but the page itself has not
   been seen in a game (no launcher here); a refused `register_setting` would leave the page
   empty and the raw `Host::setting` reads on their defaults, which is the 0.1.0 behaviour.

### Changelog (0.2.0)

> **0.2.0 — republished as an SDK `.hmod`.** Horizon's mod-API merge (launcher ≥ 0.2.70)
> retired `.hnmod` and closed **#922** (`ambientsounds` 0.1.0); this version supersedes it.
> One flat `ambientsounds.hmod` now carries the code *and* the two `.ogg` takes: `src/lib.rs`
> is the one `impl mod_api::Hooks` (`music`, `music_layers`, `client_level_load`,
> `register_commands`, `run_mod_command`), `src/support.rs` the glue, and `src/runtime.rs` is
> byte-identical to 0.1.0, so the conditions, the cadence, the easing and the ducking behave
> exactly as before. Built against SDK `0b14f8acfd97fa2c750e42795249dcc981178ca4` and mixins
> `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` for 26.3. The plan, the 6 per-hook helpers and
> the embedded-content write-out are gone; the terrain scan now reads the client's own
> `MOTION_BLOCKING` heightmap, sky exposure is derived from it (no client light read exists),
> the engine's switch and volume are two settings on the mod's page in Options > Mods, and the
> status line reports the layers the port pushed (the SDK has no live-layer or sound-registry
> read). Package: 14 entries, **2,081,784 B**, sha256
> `39ce7931723ec5a7910beac42a120506bc2f9dcf92cc1383bf5f9086ab61c6b4`; `check` and `fits`
> exit 0. No 0.2.0 in-game run: the launcher is not installed here.

### Publication route (campaign ruling 2026-11-18)

The submission form is gone; publishing is now `horizon-rust/horizon-mods#1147` — the artifact
is attached to a release of the author's own repository under the entry's exact `file` name, a
PR adds the `[[versions]]` record and links the file, and a **maintainer** runs
`scripts/stage.sh` (a red `check` before that is the expected `fetch.sh` failure). The
orchestrator's `python3 ports/tools/publish-ready.py ambientsounds` does the bookkeeping; this
job owns no submission. The port stays local-only until that approval.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port,rust-review

