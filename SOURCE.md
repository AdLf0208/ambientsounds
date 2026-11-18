# Where the carried assets come from

- **Jar:** `ports/_shared/licence-cache/AmbientSounds_NEOFORGE_v6.3.11_mc26.3.jar`
  — 84,817,294 B, sha256
  `a49c67677aa5964ea5afbfb7613bbc8e01743399560f7b6d405cd2794d43d48d`,
  318 entries. `META-INF/neoforge.mods.toml`: `modId="ambientsounds"`,
  `version="6.3.11"`, `authors="CreativeMD"`, `license="LGPL-3.0-only"`;
  `fabric.mod.json` adds **AriaFreeze** to the authors.
- **Upstream source:** `CreativeMD/AmbientSounds`, branch `26.x`, commit
  `e4c8c2df10b51733652b5ac612fb9bd2f72c2326` ("Switched from NeoGradle to
  ModDevGradle", `mod_version=6.3.11` in `gradle.properties`) — the same version
  the jar is. The repo has no release tag for 6.3.11, only `archive/*`.

## Carried (2 files, 2,055,326 B)

| File | B | Why |
|---|---:|---|
| `assets/ambientsounds/sounds/cave/cave1.ogg` | 978,787 | the `cave` bed (`regions/cave.ambience.json`'s `cave` sound, files `cave.cave1/cave2/cave3`) |
| `assets/ambientsounds/sounds/wind/light-wind.ogg` | 1,076,539 | the `wind` bed (`sound_collections/wind.json`'s `light-wind`, files `wind.light-wind`/`light-wind2`; `regions/storm.close.json` names the same two) |
| `assets/ambientsounds/sounds.json` | 548 | those two events, copied verbatim from the jar's own `sounds.json` (`"category": "ambient"`, `"stream": true`) |

`tools/assemble_content.py` writes them into the package's `assets/` from the
jar, and `--check` re-reads the jar to prove the carried bytes are the mod's.

## Port-owned (not from the jar)

`assets/ambientsounds/lang/en_us.json` — the two labels and tooltips of the settings the
mod's page in Options > Mods shows (`Registrar::register_setting`, SDK format). The mod's
own jar has no such page (its configuration is the `CreativeConfig` GUI this slice does not
carry), so these strings are the port's own.

## Not carried (118 `.ogg`, 81.6 MB, and every JSON engine file)

- `sounds/animals/` (98 files), `sounds/cave/` 15 of 16, `sounds/wind/` 6 of 8,
  `sounds/water/` (6), `sounds/weather/` (5), `sounds/suspense/` (4): the beds,
  animals, water, weather, suspense and the cave/wind variants this slice does
  not play (`title: cave.cave2`, `cave.cave3`, `cave.cave-water*`,
  `cave.cave-deep*`, `cave.cave-large*`, `cave.cave-lush`, `cave.deep-dark`,
  `cave.underworld`, `wind.light-wind2`, `wind.heavy-wind*`,
  `wind.howling-wind`, `wind.wind-in-leaves`, `wind.mesa`,
  `weather.rain*`, `weather.storm-away`).
- `assets/ambientsounds/basic/**` (the engine's JSON: `engine.json`, the 42
  region/dimension/feature/sound_collection/sound_category/blockgroup files),
  `assets/ambientsounds/config.json`, `ambientsounds.png`: the port rewrites the
  engine in Rust (`src/runtime.rs`), so the JSON is read, not
  carried; the config file and the icon are the config GUI's and the mod list's.
- the `team/creative/ambientsounds/**` classes (79): the port is a clean-room
  rewrite on Horizon, so no Java is carried and none is translated.
