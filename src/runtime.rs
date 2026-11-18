// AmbientSounds' environment engine, in the shape of the mod's own `basic`
// engine: the scan cadence, the region conditions, the bed volumes and the
// volume easing that decides whether the `cave` or the `wind` bed plays.
//
// Every number here is the jar's or the mod's source:
// `assets/ambientsounds/basic/engine.json` in
// `AmbientSounds_NEOFORGE_v6.3.11_mc26.3.jar` (the tick times, the scan
// distances, the fades), `assets/ambientsounds/basic/regions/*.json` (the
// conditions and volumes of the `cave`, `cave.ambience`, `surface`, `sky` and
// `storm.close` regions) and the classes named beside each rule.
//
// Pure: no world access, no files, no clock. `support.rs` reads the world and
// hands a [`Sample`]; this module answers which bed plays and how loud.

// ---------------------------------------------------------------------------
// The engine's own numbers
// ---------------------------------------------------------------------------

/// `environment-tick-time`: the slow environment is analysed once every 40
/// client ticks (2 s), as `AmbientTickHandler.onTick` does at
/// `timer % engine.environmentTickTime == 0`.
pub const ENVIRONMENT_TICK_TIME: u32 = 40;

/// `sound-tick-time`: the regions and the sounds are aimed once every 4 client
/// ticks, as `AmbientTickHandler.onTick` does at `timer % engine.soundTickTime`.
pub const SOUND_TICK_TIME: u32 = 4;

/// `average-height-scan-distance`: the spacing between the terrain columns the
/// average height is taken from (`TerrainEnvironment.analyzeHeight`).
pub const AVERAGE_HEIGHT_SCAN_DISTANCE: i32 = 2;

/// `average-height-scan-count`: the columns each way, so 11 x 11 samples.
pub const AVERAGE_HEIGHT_SCAN_COUNT: i32 = 5;

/// `biome-scan-distance`: the spacing of the biome scan the rain volume comes
/// from (`BiomeEnvironment`); the port reads the one biome under the eye.
pub const BIOME_SCAN_DISTANCE: i32 = 5;

/// `biome-scan-count`: the biomes each way, so 7 x 7.
pub const BIOME_SCAN_COUNT: i32 = 3;

/// `fade-volume` (a string in the JSON, `"0.005"`): how much a sound's volume
/// moves a tick, so 200 ticks from silent to full
/// (`AmbientSound.fastTick`; `AmbientSoundProperties.getFadeInVolume`).
pub const FADE_VOLUME: f64 = 0.005;

/// The two beds this slice carries, as `sounds.json` names them.
pub const CAVE_BED: &str = "ambientsounds:cave.cave1";
/// The wind bed: the `wind` sound collection's `light-wind`
/// (`sound_collections/wind.json`).
pub const WIND_BED: &str = "ambientsounds:wind.light-wind";

/// `regions/cave.ambience.json`'s `cave` sound volume.
const CAVE_SOUND_VOLUME: f64 = 0.5;
/// `regions/storm.close.json`'s `wind` sound volume.
const STORM_WIND_VOLUME: f64 = 0.4;
/// `regions/storm.close.json`'s `mute`: the storm wind ducks the beds of a lower
/// `mute-priority` by this fraction.
const STORM_WIND_MUTE: f64 = 0.3;
/// `regions/cave.json`'s `bad-biomes`: a biome id containing this plays no cave.
const CAVE_BAD_BIOMES: [&str; 1] = ["alexscaves"];

// ---------------------------------------------------------------------------
// Conditions
// ---------------------------------------------------------------------------

/// A minimum/maximum with a fade, exactly
/// `AmbientCondition.AmbientMinMaxFadeCondition.volume` (`AmbientCondition.java:374-392`):
/// zero outside the range, one a `fade` inside it, linear between.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Range {
    min: Option<f64>,
    max: Option<f64>,
    fade: Option<f64>,
}

impl Range {
    /// A condition that passes between `min` and `max`, easing over `fade`.
    pub const fn new(min: Option<f64>, max: Option<f64>, fade: Option<f64>) -> Self {
        Self { min, max, fade }
    }

    /// `AmbientMinMaxCondition.is`: the value is inside the range at all.
    pub fn passes(&self, value: f64) -> bool {
        if self.min.is_some_and(|min| value < min) {
            return false;
        }
        !self.max.is_some_and(|max| value > max)
    }

    /// The condition's volume at `value`, 0 to 1.
    pub fn intensity(&self, value: f64) -> f64 {
        if !self.passes(value) {
            return 0.0;
        }
        let Some(fade) = self.fade else {
            return 1.0;
        };
        let mut volume = 1.0;
        if let Some(min) = self.min {
            volume = ((value - min).abs() / fade).clamp(0.0, 1.0);
        }
        if let Some(max) = self.max {
            volume = volume.min(((value - max).abs() / fade).clamp(0.0, 1.0));
        }
        volume
    }
}

/// The `relative-height` variant
/// (`AmbientCondition.AmbientMinMaxFadeSpecialCondition.volume`, `AmbientCondition.java:394-404`):
/// satisfied when the eye, the lowest or the highest sampled column is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeightRange {
    range: Range,
}

impl HeightRange {
    /// The condition over the eye height and the sampled columns' extremes.
    pub const fn new(min: Option<f64>, max: Option<f64>, fade: Option<f64>) -> Self {
        Self {
            range: Range::new(min, max, fade),
        }
    }

    /// The condition's volume given the relative minimum, the eye's relative
    /// height and the relative maximum.
    pub fn intensity(&self, min: f64, value: f64, max: f64) -> f64 {
        if self.range.fade.is_none() {
            return if self.range.passes(value) { 1.0 } else { 0.0 };
        }
        let volume = self.range.intensity(value);
        if volume == 1.0 {
            return volume;
        }
        volume
            .max(self.range.intensity(min))
            .max(self.range.intensity(max))
    }
}

// ---------------------------------------------------------------------------
// The terrain scan
// ---------------------------------------------------------------------------

/// The height scan `TerrainEnvironment.analyzeHeight` takes: the average of the
/// sampled columns and their extremes, in blocks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Terrain {
    /// The mean of the sampled column heights.
    pub average: f64,
    /// The lowest sampled column.
    pub min: i32,
    /// The highest sampled column.
    pub max: i32,
}

impl Terrain {
    /// The scan over the columns' heights, or none when no column is loaded.
    pub fn from_heights(heights: &[i32]) -> Option<Self> {
        let min = *heights.iter().min()?;
        let max = *heights.iter().max()?;
        let sum: i64 = heights.iter().map(|height| i64::from(*height)).sum();
        let average = sum as f64 / heights.len() as f64;
        Some(Self { average, min, max })
    }
}

// ---------------------------------------------------------------------------
// The environment sample
// ---------------------------------------------------------------------------

/// What one slow scan read from the world: the player's eye and the boxes
/// around it, the weather, the biome and the terrain columns.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    /// The level's id (`minecraft:overworld`), as `Level.dimension()` gives it.
    pub dimension: String,
    /// The eye's y (`AmbientEnvironment.absoluteHeight`, `AmbientEnvironment.java:62`).
    pub eye_y: f64,
    /// The mean sky light, 0 to 15, of the air box around the eye: the stand-in
    /// for the air pocket's `averageSkyLight`.
    pub sky_light: f64,
    /// The share of that box open to the sky, 0 to 1: the stand-in for the air
    /// pocket's `sky`.
    pub sky: f64,
    /// The share of the box that is air, 0 to 1: the stand-in for the air
    /// pocket's `air`.
    pub air: f64,
    /// The rain level, 0 to 1 (`Level.getRainLevel`).
    pub rain: f64,
    /// The thunder level, 0 to 1 (`Level.getThunderLevel`).
    pub thunder: f64,
    /// The biome under the eye (`minecraft:plains`), when the client has it.
    pub biome: Option<String>,
    /// The terrain scan.
    pub terrain: Terrain,
}

/// The values the regions are asked about, derived from a [`Sample`] as
/// `AmbientEnvironment.analyzeFast` derives them
/// (`AmbientEnvironment.java:56-72`).
#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    /// The eye's y.
    pub absolute_height: f64,
    /// The eye above the average column.
    pub relative_height: f64,
    /// The eye above the highest column.
    pub relative_min_height: f64,
    /// The eye above the lowest column.
    pub relative_max_height: f64,
    /// The mean sky light around the eye, 0 to 15.
    pub sky_light: f64,
    /// The share of the box open to the sky, 0 to 1.
    pub sky: f64,
    /// The share of the box that is air, 0 to 1.
    pub air: f64,
    /// The rain level, 0 to 1.
    pub rain: f64,
    /// The thunder level, 0 to 1.
    pub thunder: f64,
    /// The level's id.
    pub dimension: String,
    /// The biome under the eye.
    pub biome: Option<String>,
    /// A `bad-biomes` entry of the cave region matches the biome.
    pub biome_excluded: bool,
}

impl Sample {
    /// The region inputs, as the mod derives them.
    pub fn environment(&self) -> Environment {
        Environment {
            absolute_height: self.eye_y,
            relative_height: self.eye_y - self.terrain.average,
            relative_min_height: self.eye_y - f64::from(self.terrain.max),
            relative_max_height: self.eye_y - f64::from(self.terrain.min),
            sky_light: self.sky_light,
            sky: self.sky,
            air: self.air,
            rain: self.rain,
            thunder: self.thunder,
            dimension: self.dimension.clone(),
            biome: self.biome.clone(),
            biome_excluded: self.biome.as_deref().is_some_and(biome_is_bad),
        }
    }
}

// The regions' conditions, as `assets/ambientsounds/basic/regions/*.json` states
// them. `fade` is in the same unit as the value.

/// `regions/cave.json`: `sky-light` max 5, fade 2, on the mean sky light.
const CAVE_SKY_LIGHT: Range = Range::new(None, Some(5.0), Some(2.0));
/// `regions/cave.json`: `sky` max 0.5, fade 0.1, on the sky exposure.
const CAVE_SKY: Range = Range::new(None, Some(0.5), Some(0.1));
/// `regions/cave.json`: `relative-height` max -10, fade 5.
const CAVE_RELATIVE_HEIGHT: HeightRange = HeightRange::new(None, Some(-10.0), Some(5.0));
/// `regions/cave.json`: `absolute-height` min -70, fade 5.
const CAVE_ABSOLUTE_HEIGHT: Range = Range::new(Some(-70.0), None, Some(5.0));
/// `regions/cave.ambience.json`: `min-height-relative` max -2, fade 2.
const CAVE_MIN_HEIGHT_RELATIVE: Range = Range::new(None, Some(-2.0), Some(2.0));
/// `regions/cave.ambience.json`'s variants: the first has no condition (volume
/// 1), the second is `volume: 0.25`; the loudest variant wins, so 1.
const CAVE_VARIANT_VOLUME: f64 = 1.0;
/// `regions/surface.json`: `relative-height` min -10, max 20, fade 10.
const SURFACE_RELATIVE_HEIGHT: HeightRange = HeightRange::new(Some(-10.0), Some(20.0), Some(10.0));
/// `regions/sky.json`: `relative-height` min 15, fade 20.
const SKY_RELATIVE_HEIGHT: HeightRange = HeightRange::new(Some(15.0), None, Some(20.0));
/// `sound_collections/wind.json`: `air` min 0.6, no fade.
const WIND_AIR: Range = Range::new(Some(0.6), None, None);

impl Environment {
    /// The cave region (`regions/cave.json`): a dark, deep, covered place.
    pub fn cave_region(&self) -> f64 {
        let mut volume = 1.0;
        for condition in [
            CAVE_SKY_LIGHT.intensity(self.sky_light),
            CAVE_SKY.intensity(self.sky),
            CAVE_RELATIVE_HEIGHT.intensity(
                self.relative_min_height,
                self.relative_height,
                self.relative_max_height,
            ),
            CAVE_ABSOLUTE_HEIGHT.intensity(self.absolute_height),
        ] {
            if condition <= 0.0 {
                return 0.0;
            }
            volume *= condition;
        }
        volume
    }

    /// The cave ambience region (`regions/cave.ambience.json`), whose `min-height-relative`
    /// asks the columns around the player to be above the eye.
    pub fn cave_ambience(&self) -> f64 {
        // Its `features` (rock, lush, dripstone, sculk) need the block-group scan,
        // which this slice does not carry: the port plays the ambience of every
        // cave instead of only a rocky, lush, dripstone or sculk one.
        CAVE_MIN_HEIGHT_RELATIVE.intensity(self.relative_min_height) * CAVE_VARIANT_VOLUME
    }

    /// The surface region (`regions/surface.json`).
    pub fn surface_region(&self) -> f64 {
        SURFACE_RELATIVE_HEIGHT.intensity(
            self.relative_min_height,
            self.relative_height,
            self.relative_max_height,
        )
    }

    /// The sky region (`regions/sky.json`), which carries the `wind` sound
    /// collection.
    pub fn sky_region(&self) -> f64 {
        // `regions/sky.json` lists `end` in `bad-regions`, but no region of that
        // name exists upstream, so the entry never fires; the port honours it by
        // dimension (notes.md, "Upstream defects fixed").
        if self.dimension.contains("the_end") {
            return 0.0;
        }
        SKY_RELATIVE_HEIGHT.intensity(
            self.relative_min_height,
            self.relative_height,
            self.relative_max_height,
        )
    }

    /// The storm region (`regions/storm.close.json`), which carries the storm's
    /// own wind.
    pub fn storm_region(&self) -> f64 {
        if !self.storming() {
            return 0.0;
        }
        let rain = self.rain_audible();
        if rain <= 0.0 {
            return 0.0;
        }
        // `regions: ["surface"]` contributes the surface condition's volume.
        self.surface_region() * rain
    }

    /// Whether the mod considers it raining where the player stands
    /// (`Level.isRainingAt`): a level above zero with the sky open.
    pub fn raining(&self) -> bool {
        self.rain > 0.0 && self.sky_light >= 15.0
    }

    /// Whether it thunders (`Level.isThundering`).
    pub fn storming(&self) -> bool {
        self.thunder > 0.0
    }

    /// The rain volume a `overall-raining` may multiply by
    /// (`AmbientEnvironment.rainSurfaceVolume`, from the biome scan).
    pub fn rain_audible(&self) -> f64 {
        if self.raining() {
            self.rain.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// The aimed volumes of the two beds, each 0 to 1.
    pub fn beds(&self) -> Beds {
        let cave = if self.biome_excluded {
            0.0
        } else {
            self.cave_region() * self.cave_ambience() * CAVE_SOUND_VOLUME
        };
        // The wind is the sky region's sound collection (open air above the
        // surface) and, in a thunderstorm, the storm region's own wind; the
        // louder of the two plays.
        let sky_wind = self.sky_region() * WIND_AIR.intensity(self.air);
        let storm_wind = self.storm_region() * STORM_WIND_VOLUME;
        Beds {
            cave,
            wind: sky_wind.max(storm_wind),
        }
    }
}

/// The aimed volume of each bed, 0 to 1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Beds {
    /// The cave bed's volume.
    pub cave: f64,
    /// The wind bed's volume.
    pub wind: f64,
}

impl Beds {
    /// The louder bed's name, for the status line.
    pub fn loudest(&self) -> Option<&'static str> {
        if self.wind > self.cave {
            Some("wind")
        } else if self.cave > 0.0 {
            Some("cave")
        } else {
            None
        }
    }

    /// Whether any bed is audible.
    pub fn audible(&self) -> bool {
        self.cave > 0.0 || self.wind > 0.0
    }
}

// ---------------------------------------------------------------------------
// The cadence
// ---------------------------------------------------------------------------

/// What a tick owes: the engine's two clocks
/// (`AmbientTickHandler.onTick`, `AmbientTickHandler.java:192-202`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Step {
    /// The slow environment scan is due.
    pub environment: bool,
    /// The regions and sounds are aimed again.
    pub sound: bool,
}

/// The tick counter and its two clocks.
#[derive(Clone, Copy, Debug, Default)]
pub struct Cadence {
    tick: u32,
}

impl Cadence {
    /// Advance one client tick and say what is due.
    pub fn step(&mut self) -> Step {
        let tick = self.tick;
        self.tick = (self.tick + 1) % ENVIRONMENT_TICK_TIME;
        Step {
            environment: tick.is_multiple_of(ENVIRONMENT_TICK_TIME),
            sound: tick.is_multiple_of(SOUND_TICK_TIME),
        }
    }

    /// Start over, as a level load does.
    pub fn reset(&mut self) {
        self.tick = 0;
    }
}

// ---------------------------------------------------------------------------
// The easing
// ---------------------------------------------------------------------------

/// Whether a `bad-biomes` entry matches a biome id. Upstream compiles every
/// entry of the list into the regex `.*<entry>.*` with `*` as its only wildcard
/// (`condition/BiomeCondition.of`), so `alexscaves` matches
/// `alexscaves:toxic_cave` and a `#tag` entry matches a tag id.
pub fn pattern_matches(pattern: &str, biome: &str) -> bool {
    // `.*<pattern>.*` with `*` as the only wildcard: every literal segment must
    // appear in order, anywhere in the id, and nothing anchors either end.
    let mut rest = biome;
    for segment in pattern.split('*') {
        if segment.is_empty() {
            continue;
        }
        let Some(at) = rest.find(segment) else {
            return false;
        };
        rest = &rest[at + segment.len()..];
    }
    true
}

/// Whether the cave region refuses this biome (`regions/cave.json`'s
/// `bad-biomes`), asked of every biome the scan finds.
pub fn biome_is_bad(biome: &str) -> bool {
    CAVE_BAD_BIOMES
        .iter()
        .any(|pattern| pattern_matches(pattern, biome))
}

/// The beds after the mute priorities (`AmbientSoundEngine.tick`,
/// `AmbientSoundEngine.java:33-77`): a bed with a `mute` ducks every bed of a
/// lower `mute-priority` by that fraction, which is what the storm's own wind
/// does to the beds under it.
pub fn duck(beds: Beds) -> Beds {
    // The storm wind carries `mute: 0.3, mute-priority: 5`, the cave bed neither
    // (`mute-priority` defaults to 0), so the wind ducks the cave and nothing
    // ducks the wind.
    let mute = if beds.wind > 0.0 {
        STORM_WIND_MUTE
    } else {
        0.0
    };
    Beds {
        cave: beds.cave * (1.0 - mute),
        wind: beds.wind,
    }
}

/// One tick of `AmbientSound.fastTick`'s volume easing
/// (`AmbientSound.java:163-171`): `current` moves toward `aimed` by
/// [`FADE_VOLUME`] and never past it.
pub fn ease(current: f64, aimed: f64) -> f64 {
    if current < aimed {
        (current + FADE_VOLUME).min(aimed)
    } else if current > aimed {
        (current - FADE_VOLUME).max(aimed)
    } else {
        aimed
    }
}

// ---------------------------------------------------------------------------
// The engine
// ---------------------------------------------------------------------------

/// The engine's state: on or off, its volume setting, the last sample and the
/// two beds' current volumes.
#[derive(Clone, Debug)]
pub struct Engine {
    enabled: bool,
    volume: f64,
    cadence: Cadence,
    environment: Option<Environment>,
    aimed: Beds,
    current: Beds,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// An enabled engine at volume 1, as `AmbientEngine.fadeVolume`'s defaults
    /// leave it.
    pub const fn new() -> Self {
        Self {
            enabled: true,
            volume: 1.0,
            cadence: Cadence { tick: 0 },
            environment: None,
            aimed: Beds {
                cave: 0.0,
                wind: 0.0,
            },
            current: Beds {
                cave: 0.0,
                wind: 0.0,
            },
        }
    }

    /// One client tick's cadence: the support module scans when
    /// `environment` is set, aims when `sound` is set, and always ticks.
    pub fn step(&mut self) -> Step {
        self.cadence.step()
    }

    /// A fresh slow scan: the sample is kept and the beds aimed at once.
    pub fn observe(&mut self, sample: Sample) {
        self.environment = Some(sample.environment());
        self.aim();
    }

    /// The world was not readable this scan (no player, a screen): the beds
    /// aim at silence but keep ramping down rather than cutting out.
    pub fn lost(&mut self) {
        self.environment = None;
        self.aim();
    }

    /// Aim the beds again from the last sample.
    pub fn aim(&mut self) {
        self.aimed = match &self.environment {
            Some(environment) if self.enabled && self.volume > 0.0 => {
                let beds = environment.beds();
                Beds {
                    cave: beds.cave * self.volume,
                    wind: beds.wind * self.volume,
                }
            }
            _ => Beds::default(),
        };
    }

    /// One client tick of easing; answers what to hand to the sound engine.
    pub fn tick(&mut self) -> Beds {
        self.current.cave = ease(self.current.cave, self.aimed.cave);
        self.current.wind = ease(self.current.wind, self.aimed.wind);
        duck(self.current)
    }

    /// The layers to play: a bed as its event and volume, only while audible.
    pub fn layers(&self) -> Vec<(&'static str, f32)> {
        let played = duck(self.current);
        let mut layers = Vec::new();
        if played.cave > 0.0 {
            layers.push((CAVE_BED, played.cave as f32));
        }
        if played.wind > 0.0 {
            layers.push((WIND_BED, played.wind as f32));
        }
        layers
    }

    /// Whether vanilla's music is held back: any bed is audible
    /// (`SelectMusicEvent` gets `None` and volume 0 while a bed plays).
    pub fn suppresses_vanilla(&self) -> bool {
        self.enabled && duck(self.current).audible()
    }

    /// Turn the engine on or off, as the setting does.
    pub fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
        self.aim();
    }

    /// The engine's own volume, `AmbientSounds.CONFIG.volume` upstream, 0 to 1.
    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
        self.aim();
    }

    /// A level load or a dimension change: the next scan starts over.
    pub fn reset(&mut self) {
        self.cadence.reset();
        self.environment = None;
        self.aimed = Beds::default();
        self.current = Beds::default();
    }

    /// The status line `/ambientsounds status` answers.
    pub fn status(&self) -> String {
        let played = duck(self.current);
        let bed = played.loudest().unwrap_or("none");
        let (aimed, current) = match played.loudest() {
            Some("wind") => (self.aimed.wind, played.wind),
            _ => (self.aimed.cave, played.cave),
        };
        let mut line = format!(
            "AmbientSounds: engine={} volume={:.2} bed={bed} aimed={aimed:.3} \
             current={current:.3} cave={:.3} wind={:.3}",
            if self.enabled { "on" } else { "off" },
            self.volume,
            played.cave,
            played.wind,
        );
        let vanilla = if self.suppresses_vanilla() {
            "held"
        } else {
            "plays"
        };
        line.push_str(&format!(" vanilla={vanilla}"));
        if let Some(environment) = &self.environment {
            line.push_str(&format!(
                " region_cave={:.3} region_ambience={:.3} region_surface={:.3} \
                 region_sky={:.3} region_storm={:.3}",
                environment.cave_region(),
                environment.cave_ambience(),
                environment.surface_region(),
                environment.sky_region(),
                environment.storm_region(),
            ));
            line.push_str(&format!(
                " dimension={} sky_light={:.2} sky={:.2} air={:.2} relative={:.1} \
                 rain={:.2} thunder={:.2} excluded={}",
                environment.dimension,
                environment.sky_light,
                environment.sky,
                environment.air,
                environment.relative_height,
                environment.rain,
                environment.thunder,
                u8::from(environment.biome_excluded),
            ));
            if let Some(biome) = &environment.biome {
                line.push_str(&format!(" biome={biome}"));
            }
        } else {
            line.push_str(
                " region_cave=0.000 region_ambience=0.000 region_surface=0.000 \
                 region_sky=0.000 region_storm=0.000 no_scan",
            );
        }
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scan underground: dark, deep, covered, the terrain far above the eye.
    fn cave_sample() -> Sample {
        Sample {
            dimension: "minecraft:overworld".into(),
            eye_y: 30.0,
            sky_light: 0.0,
            sky: 0.0,
            air: 0.7,
            rain: 0.0,
            thunder: 0.0,
            biome: Some("minecraft:plains".into()),
            terrain: Terrain {
                average: 70.0,
                min: 68,
                max: 74,
            },
        }
    }

    /// A scan on open ground: the eye just above the average column.
    fn ground_sample() -> Sample {
        Sample {
            dimension: "minecraft:overworld".into(),
            eye_y: 71.0,
            sky_light: 15.0,
            sky: 1.0,
            air: 1.0,
            rain: 0.0,
            thunder: 0.0,
            biome: Some("minecraft:plains".into()),
            terrain: Terrain {
                average: 70.0,
                min: 68,
                max: 74,
            },
        }
    }

    /// A scan far above the ground, in open air.
    fn altitude_sample() -> Sample {
        Sample {
            eye_y: 140.0,
            ..ground_sample()
        }
    }

    #[test]
    fn sky_light_condition_fades_below_five() {
        let condition = Range::new(None, Some(5.0), Some(2.0));
        assert_eq!(condition.intensity(0.0), 1.0);
        assert_eq!(condition.intensity(3.0), 1.0);
        assert_eq!(condition.intensity(4.0), 0.5);
        assert_eq!(condition.intensity(5.0), 0.0);
        assert_eq!(condition.intensity(6.0), 0.0);
    }

    #[test]
    fn absolute_height_condition_fades_above_minus_seventy() {
        let condition = Range::new(Some(-70.0), None, Some(5.0));
        assert_eq!(condition.intensity(-70.0), 0.0);
        assert_eq!(condition.intensity(-67.5), 0.5);
        assert_eq!(condition.intensity(-65.0), 1.0);
        assert_eq!(condition.intensity(30.0), 1.0);
        assert_eq!(condition.intensity(-80.0), 0.0);
    }

    #[test]
    fn a_condition_without_a_fade_is_a_gate() {
        let condition = Range::new(Some(0.6), None, None);
        assert_eq!(condition.intensity(0.6), 1.0);
        assert_eq!(condition.intensity(0.59), 0.0);
        assert!(condition.passes(0.9));
        assert!(!condition.passes(0.1));
    }

    #[test]
    fn relative_height_keeps_the_special_fallback() {
        // regions/cave.json: relative-height max -10 fade 5.
        let condition = HeightRange::new(None, Some(-10.0), Some(5.0));
        // The eye is inside the fade but a sampled column is far below it.
        assert_eq!(condition.intensity(-44.0, -8.0, -2.0), 1.0);
        // All three in the fade's span, four blocks under the ceiling.
        let near = condition.intensity(-12.0, -12.0, -12.0);
        assert!((near - 0.4).abs() < 1.0E-9, "{near}");
        // At the ceiling itself the fade has not started.
        assert_eq!(condition.intensity(-10.0, -10.0, -10.0), 0.0);
        // Above the ceiling everywhere: nothing.
        assert_eq!(condition.intensity(100.0, 100.0, 100.0), 0.0);
    }

    #[test]
    fn terrain_averages_the_height_scan() {
        let terrain = Terrain::from_heights(&[60, 62, 64, 68, 70]).expect("heights");
        assert_eq!(terrain.average, 64.8);
        assert_eq!((terrain.min, terrain.max), (60, 70));
    }

    #[test]
    fn a_scan_with_no_loaded_column_has_no_terrain() {
        assert_eq!(Terrain::from_heights(&[]), None);
    }

    #[test]
    fn cadence_is_the_engine_json() {
        let mut cadence = Cadence::default();
        let mut environment = Vec::new();
        let mut sound = Vec::new();
        for tick in 0..=80_u32 {
            let step = cadence.step();
            if step.environment {
                environment.push(tick);
            }
            if step.sound {
                sound.push(tick);
            }
        }
        assert_eq!(environment, vec![0, 40, 80]);
        assert_eq!(sound.len(), 21);
        assert!(sound.iter().all(|tick| tick % SOUND_TICK_TIME == 0));
    }

    #[test]
    fn an_underground_scan_aims_the_cave_bed_at_one_half() {
        let beds = cave_sample().environment().beds();
        assert_eq!(beds.cave, 0.5);
        assert_eq!(beds.wind, 0.0);
        assert_eq!(beds.loudest(), Some("cave"));
    }

    #[test]
    fn open_ground_plays_neither_bed() {
        let beds = ground_sample().environment().beds();
        assert_eq!(beds, Beds::default());
    }

    #[test]
    fn sky_light_keeps_a_lit_cave_quiet() {
        let mut sample = cave_sample();
        sample.sky_light = 15.0;
        sample.sky = 1.0;
        assert_eq!(sample.environment().beds().cave, 0.0);
    }

    #[test]
    fn a_half_lit_cave_fades() {
        let mut sample = cave_sample();
        sample.sky_light = 4.0;
        // regions/cave.json sky-light max 5 fade 2 -> 0.5, times 0.5 the sound.
        assert_eq!(sample.environment().beds().cave, 0.25);
    }

    #[test]
    fn a_high_platform_plays_the_wind_bed() {
        let beds = altitude_sample().environment().beds();
        assert_eq!(beds.wind, 1.0);
        assert_eq!(beds.cave, 0.0);
        assert_eq!(beds.loudest(), Some("wind"));
    }

    #[test]
    fn the_wind_bed_needs_open_air() {
        let mut sample = altitude_sample();
        sample.air = 0.3;
        // sound_collections/wind.json: air min 0.6, no fade.
        assert_eq!(sample.environment().beds().wind, 0.0);
    }

    #[test]
    fn a_thunderstorm_at_ground_plays_the_storm_wind() {
        let mut sample = ground_sample();
        sample.rain = 1.0;
        sample.thunder = 1.0;
        let beds = sample.environment().beds();
        assert!((beds.wind - 0.4).abs() < 1.0E-9, "wind {}", beds.wind);
        assert_eq!(beds.cave, 0.0);
    }

    #[test]
    fn plain_rain_at_ground_carries_no_wind() {
        let mut sample = ground_sample();
        sample.rain = 1.0;
        // The storm region is the only ground-level wind, and it needs thunder.
        assert_eq!(sample.environment().beds().wind, 0.0);
    }

    #[test]
    fn a_half_strength_storm_carries_half_the_wind() {
        let mut sample = ground_sample();
        sample.rain = 0.5;
        sample.thunder = 1.0;
        let beds = sample.environment().beds();
        assert!((beds.wind - 0.2).abs() < 1.0E-9, "wind {}", beds.wind);
    }

    #[test]
    fn a_storm_above_the_ground_does_not_stack_wind() {
        let mut sample = altitude_sample();
        sample.rain = 1.0;
        sample.thunder = 1.0;
        // The surface region is off up here, so only the sky region's 1.0 runs.
        assert_eq!(sample.environment().beds().wind, 1.0);
    }

    #[test]
    fn a_storm_underground_is_still_only_the_cave() {
        let mut sample = cave_sample();
        sample.rain = 1.0;
        sample.thunder = 1.0;
        let beds = sample.environment().beds();
        assert_eq!(beds.cave, 0.5);
        assert_eq!(beds.wind, 0.0);
    }

    #[test]
    fn the_end_has_no_sky_wind() {
        // regions/sky.json names `end` in bad-regions, but no region of that
        // name exists upstream, so the condition never fires there. The port
        // honours it by dimension (see notes.md, "Upstream defects fixed").
        let mut sample = altitude_sample();
        sample.dimension = "minecraft:the_end".into();
        let beds = sample.environment().beds();
        assert_eq!(beds.wind, 0.0);
        assert_eq!(beds.cave, 0.0);
    }

    #[test]
    fn a_bad_biome_silences_the_cave_bed() {
        let mut sample = cave_sample();
        sample.biome = Some("alexscaves:toxic_cave".into());
        let environment = sample.environment();
        assert!(environment.biome_excluded);
        assert_eq!(environment.beds().cave, 0.0);
    }

    #[test]
    fn biome_patterns_match_like_upstreams_regex() {
        assert!(pattern_matches("alexscaves", "alexscaves:toxic_cave"));
        assert!(!pattern_matches("alexscaves", "minecraft:lush_caves"));
        // `*` is the only wildcard upstream compiles (`BiomeCondition.of`).
        assert!(pattern_matches("alex*s", "alexscaves:primordial_caves"));
        assert!(pattern_matches("cave", "minecraft:lush_caves"));
    }

    #[test]
    fn volume_eases_by_the_fade_volume() {
        assert_eq!(ease(0.0, 0.5), FADE_VOLUME);
        assert_eq!(ease(0.5, 0.0), 0.5 - FADE_VOLUME);
        // A step smaller than the fade lands exactly on the target.
        assert_eq!(ease(0.499, 0.5), 0.5);
        assert_eq!(ease(0.001, 0.0), 0.0);
    }

    #[test]
    fn a_full_swing_takes_two_hundred_ticks() {
        let mut volume = 0.0;
        for _ in 0..100 {
            volume = ease(volume, 0.5);
        }
        assert_eq!(volume, 0.5, "reaches the aim in 100 ticks at half volume");
        for _ in 0..100 {
            volume = ease(volume, 0.0);
        }
        assert_eq!(volume, 0.0);
    }

    #[test]
    fn the_engine_ramps_up_and_down_with_the_bed() {
        let mut engine = Engine::new();
        engine.observe(cave_sample());
        engine.aim();
        assert_eq!(engine.tick().cave, FADE_VOLUME);
        for _ in 0..99 {
            engine.tick();
        }
        assert_eq!(engine.current.cave, 0.5);
        // Out of the cave: the aimed volume is zero, the current one falls.
        engine.observe(ground_sample());
        engine.aim();
        assert_eq!(engine.aimed, Beds::default());
        assert_eq!(engine.tick().cave, 0.5 - FADE_VOLUME);
        for _ in 0..99 {
            engine.tick();
        }
        assert_eq!(engine.current.cave, 0.0);
        assert!(engine.layers().is_empty());
    }

    #[test]
    fn a_disabled_engine_aims_at_nothing() {
        let mut engine = Engine::new();
        engine.observe(cave_sample());
        engine.aim();
        engine.set_enabled(false);
        assert_eq!(engine.aimed, Beds::default());
        assert_eq!(engine.tick(), Beds::default());
        assert!(engine.layers().is_empty());
        assert!(!engine.suppresses_vanilla());
    }

    #[test]
    fn a_zero_volume_aims_at_nothing() {
        let mut engine = Engine::new();
        engine.observe(cave_sample());
        engine.aim();
        assert_eq!(engine.aimed.cave, 0.5);
        engine.set_volume(0.0);
        assert_eq!(engine.aimed, Beds::default());
        assert!(engine.layers().is_empty());
    }

    #[test]
    fn half_volume_halves_the_bed() {
        let mut engine = Engine::new();
        engine.set_volume(0.5);
        engine.observe(cave_sample());
        engine.aim();
        assert_eq!(engine.aimed.cave, 0.25);
    }

    #[test]
    fn a_lost_world_aims_at_silence() {
        let mut engine = Engine::new();
        engine.observe(cave_sample());
        engine.aim();
        assert_eq!(engine.aimed.cave, 0.5);
        engine.lost();
        assert_eq!(engine.aimed, Beds::default());
        assert!(engine.environment.is_none());
    }

    #[test]
    fn vanilla_music_is_held_back_only_while_a_bed_is_audible() {
        let mut engine = Engine::new();
        engine.observe(cave_sample());
        engine.aim();
        assert!(!engine.suppresses_vanilla(), "silent before the first tick");
        engine.tick();
        assert!(engine.suppresses_vanilla(), "the ramp has started");
        let mut silent = Engine::new();
        silent.observe(ground_sample());
        silent.aim();
        silent.tick();
        assert!(!silent.suppresses_vanilla());
    }

    #[test]
    fn layers_are_the_audible_bed_as_its_event() {
        let mut engine = Engine::new();
        engine.observe(cave_sample());
        engine.aim();
        for _ in 0..100 {
            engine.tick();
        }
        assert_eq!(engine.layers(), vec![(CAVE_BED, 0.5_f32)]);
        let mut windy = Engine::new();
        windy.observe(altitude_sample());
        windy.aim();
        for _ in 0..200 {
            windy.tick();
        }
        assert_eq!(windy.layers(), vec![(WIND_BED, 1.0_f32)]);
    }

    #[test]
    fn the_storm_wind_ducks_the_cave_bed() {
        // AmbientSoundEngine.tick: a sound's mute ducks every lower priority.
        let beds = Beds {
            cave: 0.5,
            wind: 0.4,
        };
        let ducked = duck(beds);
        assert!((ducked.cave - 0.35).abs() < 1.0E-9, "cave {}", ducked.cave);
        assert!((ducked.wind - 0.4).abs() < 1.0E-9, "wind {}", ducked.wind);
        assert_eq!(
            duck(Beds {
                cave: 0.5,
                wind: 0.0
            })
            .cave,
            0.5
        );
    }

    #[test]
    fn a_level_load_starts_over() {
        let mut engine = Engine::new();
        engine.observe(cave_sample());
        engine.aim();
        engine.tick();
        engine.reset();
        assert_eq!(engine.current, Beds::default());
        assert_eq!(engine.aimed, Beds::default());
        assert!(engine.environment.is_none());
        assert_eq!(
            engine.step(),
            Step {
                environment: true,
                sound: true
            }
        );
    }

    #[test]
    fn the_status_line_names_the_bed_and_the_numbers() {
        let mut engine = Engine::new();
        engine.observe(cave_sample());
        engine.aim();
        for _ in 0..100 {
            engine.tick();
        }
        let line = engine.status();
        assert!(line.starts_with("AmbientSounds:"), "{line}");
        assert!(line.contains("engine=on"), "{line}");
        assert!(line.contains("bed=cave"), "{line}");
        assert!(line.contains("cave=0.500"), "{line}");
        assert!(line.contains("wind=0.000"), "{line}");
        assert!(line.contains("region_cave=1.000"), "{line}");
        assert!(line.contains("region_sky=0.000"), "{line}");
        assert!(line.contains("dimension=minecraft:overworld"), "{line}");
        assert!(line.contains("biome=minecraft:plains"), "{line}");
        assert!(line.contains("sky_light=0.00"), "{line}");
        assert!(line.contains("relative=-40.0"), "{line}");
        assert!(line.contains("aimed=0.500"), "{line}");
        assert!(line.contains("current=0.500"), "{line}");
        assert!(line.contains("vanilla=held"), "{line}");
    }

    #[test]
    fn the_status_line_says_when_the_engine_is_off() {
        let mut engine = Engine::new();
        engine.set_enabled(false);
        let line = engine.status();
        assert!(line.contains("engine=off"), "{line}");
        assert!(line.contains("bed=none"), "{line}");
    }
}
