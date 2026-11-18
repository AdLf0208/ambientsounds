# FINDINGS — AmbientSounds 6.3.11, the environment engine and the two beds

Evidence: the jar `ports/_shared/licence-cache/AmbientSounds_NEOFORGE_v6.3.11_mc26.3.jar`
(84,817,294 B, sha256 `a49c67677aa5964ea5afbfb7613bbc8e01743399560f7b6d405cd2794d43d48d`,
318 entries) and the upstream source `CreativeMD/AmbientSounds`, branch `26.x`,
commit `e4c8c2df10b51733652b5ac612fb9bd2f72c2326` (`gradle.properties`:
`mod_version=6.3.11`, so the clone matches the jar; the repo has no 6.3.11 tag,
only `archive/*`). The clone was deleted after these notes were written; the jar
stays as the licence and provenance evidence.

## The jar's inventory (measured)

| Kind | Count | Bytes |
|---|---:|---:|
| total entries | 318 | 84,817,294 |
| `.ogg` | 120 | 85,538,631 (uncompressed; `animals` 86, `cave` 15, `wind` 7, `water` 5, `weather` 4, `suspense` 3) |
| `.json` | 85 | — |
| `.class` | 66 | — |
| other | 47 | `META-INF/neoforge.mods.toml`, `fabric.mod.json`, `pack.mcmeta`, `ambientsounds.png`, mixin config, directories |

`META-INF/neoforge.mods.toml`: `modId="ambientsounds"`, `version="6.3.11"`,
`authors="CreativeMD"`, `license="LGPL-3.0-only"`, dependencies `neoforge
26.3.0.1-beta+`, `minecraft 26.3`, `creativecore 2.14.14+`. `fabric.mod.json`
adds **AriaFreeze** to the authors and says `"environment": "client"`.
`assets/ambientsounds/config.json`: `{"default-engine": "basic", "engines":
["basic"]}`.

## The engine's numbers — `assets/ambientsounds/basic/engine.json`

| Key | Value | Used here as |
|---|---|---|
| `environment-tick-time` | 40 | the slow scan's period (2 s) |
| `sound-tick-time` | 4 | the regions'/sounds' period |
| `average-height-scan-distance` | 2 | terrain columns 2 blocks apart |
| `average-height-scan-count` | 5 | 11 × 11 columns |
| `biome-scan-distance` | 5 | the biome scan's spacing |
| `biome-scan-count` | 3 | 7 × 7 biomes |
| `block-scan-distance` | 40 | the air-pocket scan's reach — **not carried** (gap) |
| `air-pocket-count` | 50000 | points in that scan — not carried |
| `air-pocket-groups` | 4 groups (distance 5/5/5/10, weight 10/5/2/1) | not carried |
| `air-distance` / `air-sky-distance` / `air-sky-weight` | 60 / 13 / 50 | not carried |
| `air-min` / `air-max` | 0.01 / 0.07 | the air curve's ends (the port's air is a plain 0..1 share) |
| `sky-distance` / `sky-min-count` / `sky-max-count` | 20 / 20 / 100 | the sky curve (the port's sky is a 0..1 share) |
| `solids` | `#minecraft:doors`, `#minecraft:impermeable`, `#minecraft:stairs`, `#c:glass_panes`, `#minecraft:slabs`, `#minecraft:trapdoors` | the air-pocket scan only |
| `water` | `minecraft:water`, `c:water`, `c:salt_water` | underwater depth |
| `biome-types` | `surface`, `underground` | not carried |
| `default-biome-type` | `surface` | not carried |
| `fade-volume` | `"0.005"` (a string) | the easing step: 200 ticks silent → full |
| `fade-pitch` | `"0.005"` | not carried (music layers have no pitch) |

Java defaults beside them (`AmbientEngine.java:268-322`): the same numbers, with
`blockScanDistance = 40`, `airDistance = 25`, `airMin = 0.1`, `airMax = 0.5` and
`skyMinCount = 10` as *field* defaults, overridden by the JSON.

## The cadence — `AmbientTickHandler.java:157-207`

```java
if (level != null && player != null && mc.options.getSoundSourceVolume(SoundSource.AMBIENT) > 0) {
    if (mc.isPaused()) return;
    if (environment == null) environment = new AmbientEnvironment();
    AmbientDimension newDimension = engine.getDimension(level);
    if (environment.dimension != newDimension) { engine.changeDimension(environment, newDimension); environment.dimension = newDimension; }
    if (timer % engine.environmentTickTime == 0) environment.analyzeSlow(engine, newDimension, player, level, timer);
    if (timer % engine.soundTickTime == 0) {
        environment.analyzeFast(engine, newDimension, player, level, mc.getDeltaTracker().getGameTimeDeltaPartialTick(false));
        environment.dimension.manipulateEnviroment(environment);
        engine.tick(environment);
    }
    engine.fastTick(environment);
    timer++;
} else if (!engine.activeRegions.isEmpty()) engine.stopEngine();
```

So: a **slow scan every 40 ticks**, an **aim every 4**, and an **eased step every
tick** (`engine.fastTick` → `AmbientSound.fastTick`). `engine.tick`
(`AmbientEngine.java:463-492`) ticks the sound categories, then the dimension's
regions, then every general region, and adds/removes the active ones.

## The environment the regions read — `AmbientEnvironment.java:56-113`

```java
this.raining = level.isRainingAt(player.blockPosition().above());
this.snowing = level.getBiome(player.blockPosition()).value().coldEnoughToSnow(...) && level.isRaining();
this.thundering = level.isThundering() && !snowing;
this.absoluteHeight = player.getEyeY();
this.relativeHeight = absoluteHeight - terrain.averageHeight;
this.relativeMinHeight = absoluteHeight - terrain.maxHeight;
this.relativeMaxHeight = absoluteHeight - terrain.minHeight;
```
`analyzeSlow` (line 109) also runs the terrain scan and the biome scan, whose
`rainVolume()` becomes `rainSurfaceVolume` (the `overall-raining` multiplier).

The terrain scan — `TerrainEnvironment.java:81-113`: 11 × 11 columns at twice
`averageHeightScanDistance`, each stopping at the highest block that is
`isSolidRender() || leaves || water` read from the world's top down
(`getHeightBlock`, lines 28-42); `average = sum / count`, plus `min`/`max`. A
dimension may override it with `average-height` (the End sets `0`).

## The conditions — `condition/AmbientCondition.java`

```java
// AmbientMinMaxCondition.is            (line 350)
if (min != null && value < min) return false;
if (max != null && value > max) return false;
// AmbientMinMaxFadeCondition.volume    (line 378)
if (!is(value)) return 0;
if (fade == null) return 1;
double volume = 1;
if (min != null) volume = Mth.clamp(Math.abs(value - min) / fade, 0, 1);
if (max != null) volume = Math.min(volume, Mth.clamp(Math.abs(value - max) / fade, 0, 1));
return volume;
// AmbientMinMaxFadeSpecialCondition.volume (line 396, the `relative-height` fields)
if (fade == null) return is(value) ? 1 : 0;
double volume = volume(value);
if (volume == 1) return volume;
return Math.max(volume(min), volume(max));
```

`value(env)` (lines 130-343) refuses first (`always`, `volume <= 0`, day/night,
`raining`, `overall-raining`, `snowing`, `storming`, bad features, bad regions)
and then multiplies every condition's volume in a fixed order: time, regions,
biomes, `overall-raining` (× `rainSurfaceVolume`), underwater, relative height,
min/max relative height, absolute height, light, block light, sky light, air,
sky, features, temperature, entity, variants (the loudest variant wins).

## The regions of this slice — `assets/ambientsounds/basic/regions/*.json`

`cave.json` — the region that says "a dark, deep, covered place":

```json
{"name": "cave", "bad-biomes": ["alexscaves"],
 "sky-light": {"max": 5, "fade": 2}, "sky": {"max": 0.5, "fade": 0.1},
 "relative-height": {"max": -10, "fade": 5}, "absolute-height": {"min": -70, "fade": 5}}
```

`cave.ambience.json` — the bed itself: `regions: ["cave", "underworld"]`,
`min-height-relative: {max: -2, fade: 2}`, `features: ["rock", "lush",
"dripstone", "sculk"]`, `variants: [{bad-regions: ["cave.deep-dark"]},
{volume: 0.25}]`, and the `cave` sound: files `cave.cave1/2/3`, **volume 0.5**,
`length {min: 400, max: 700}`, `transition: 200`.

`sky.json` — where the wind lives: `relative-height {min: 15, fade: 20}`,
`bad-regions: ["end"]`, `sound-collections: ["wind"]`.

`storm.close.json` — the storm's own wind: `storming: true`, `overall-raining:
true`, `bad-regions: ["cold"]`, `regions: ["surface"]`, sounds `storm-close` and
`wind` (files `wind.light-wind`, `light-wind2`, **volume 0.4**, `mute: 0.3`,
`mute-priority: 5`). `rain.json` (not carried) is the plain-rain region:
`regions: ["surface"]`, `bad-regions: ["cold"]`, variants on `raining` and
`overall-raining`.

`sound_collections/wind.json` — the collection `sky` uses: one sound
`light-wind`, files `ambientsounds:wind.light-wind` / `light-wind2`,
`transition: 200`, `air: {min: 0.6}`, `length {min: 1000, max: 5000}`.

`sounds.json` — 87 events, all `"stream": true`, `"category": "ambient"`:
`cave.cave1` → `ambientsounds:cave/cave1`, `wind.light-wind` →
`ambientsounds:wind/light-wind` (and `light-wind2`), `weather.rain1/2`,
`weather.storm-close/away`, the remaining cave and animal beds.

Dimensions: `end.json` (`dimension-names: ["the*end"]`, `rain: false`,
`storm: false`, `average-height: 0`, one region `unknown` at volume 0.3 with the
`suspense` collection); `nether.json`; `cave.json` (`deep_dark`,
`underground`); `surface.json` (no `dimension-names`, so `AmbientDimension.is`
returns true for every level the earlier files did not claim).
`AmbientDimension.java:102-116` matches `.*<name>.*` with `*` → `.*` against
`level.dimension().identifier()`.

## The sound engine's volume law

`AmbientSound.tick` (`AmbientSound.java:273-299`) aims two numbers:

```java
cachedAimedConditionVolume = (float) selection.conditionVolume();            // line 282
cachedAimedOutputVolume   = (float) (aimedVolume.volume() * volumeSetting
                                     * env.dimension.volumeSetting * AmbientSounds.CONFIG.volume); // 283
```

and `AmbientSound.fastTick` (lines 162-171) moves the current values toward them
every tick:

```java
if (currentConditionVolume < cachedAimedConditionVolume)
    currentConditionVolume += Math.min(currentPropertries.getFadeInVolume(engine), cachedAimedConditionVolume - currentConditionVolume);
else if (currentConditionVolume > cachedAimedConditionVolume)
    currentConditionVolume -= Math.min(currentPropertries.getFadeOutVolume(engine), currentConditionVolume - cachedAimedConditionVolume);
```

with `getFadeInVolume`/`getFadeOutVolume` falling back to `engine.fadeVolume`
(`AmbientSoundProperties.java:52-66`), i.e. **0.005 a tick**: 200 ticks from
silent to full, 100 ticks for the 0.5 the cave bed reaches.

Ducking: `AmbientSoundEngine.tick` (`AmbientSoundEngine.java:33-83`) builds the
sounds' `mute` by priority; a sound's `mute-priority` above another's means its
`mute` multiplies `1 - mute` into the other's effective volume. The storm wind
(priority 5, mute 0.3) therefore ducks the cave bed (priority 0) by 30 %.

## Differences recorded for the rewrite

- The air-pocket scan (`AirPocketScanner`, 329 lines, 50,000 points, a weighted
  ray distribution) is not carried: the port measures a 5 × 5 × 5 box around the
  eye instead, and the `features` condition (rock/lush/dripstone/sculk block
  groups) is dropped, so the cave ambience plays in every cave.
- `length`/`pause`/`transition` (the mod cross-fades between takes every 20-35 s)
  cannot be expressed by the pin's music layers, which loop one event per layer:
  the port loops one take.
- `fade-pitch` and the pitch curves are not carried (the layer API has no pitch).
- The biome scan is four corner samples plus the eye rather than 7 × 7 weighted.
