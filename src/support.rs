//! Game glue for AmbientSounds: the environment scan, the two beds handed to the
//! music manager, the vanilla-music suppression and the `/ambientsounds` surface.
//!
//! The rules live in [`crate::runtime`]; this module reads the world through the
//! SDK's client hosts and feeds the engine. The two decisions that can be checked
//! without a game — how a heightmap reading becomes the engine's `sky_light`/`sky`
//! inputs, and how a command word becomes a setting — are pure and tested here.

use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use mod_api::ClientHost;
use mod_api::Command;
use mod_api::CommandArgumentValue;
use mod_api::Flow;
use mod_api::Heightmap;
use mod_api::Host;
use mod_api::LevelRef;
use mod_api::MusicEvent;
use mod_api::MusicLayer;
use mod_api::MusicLayersEvent;
use mod_api::Registrar;
use mod_api::RunModCommandEvent;
use mod_api::ServerHost;
use mod_api::SettingSpec;
use mod_api::Text;

use crate::runtime::AVERAGE_HEIGHT_SCAN_COUNT;
use crate::runtime::AVERAGE_HEIGHT_SCAN_DISTANCE;
use crate::runtime::BIOME_SCAN_COUNT;
use crate::runtime::BIOME_SCAN_DISTANCE;
use crate::runtime::Engine;
use crate::runtime::Sample;
use crate::runtime::Terrain;
use crate::runtime::biome_is_bad;

/// The mod's id: its setting keys' owner and the command's name.
pub const MOD_ID: &str = "ambientsounds";

/// The radius of the air box the sky and air shares are measured over, in blocks.
/// The stand-in for the mod's 40-block air-pocket scan (`notes.md`, gaps).
const EXPOSURE_RADIUS: i32 = 2;

/// The engine, shared by the client ticks and the command (single player runs
/// both in one process; on a dedicated server the client hooks never fire, so it
/// stays at its defaults).
static ENGINE: Mutex<Engine> = Mutex::new(Engine::new());

/// The saved settings were read once.
static LOADED: AtomicBool = AtomicBool::new(false);

/// A `/ambientsounds rescan` was asked for on the command's side and the next
/// client tick owes the scan (the command cannot read the client's own world).
static RESCAN: AtomicBool = AtomicBool::new(false);

/// The track the game chose on the last music tick (evidence for the e2e that the
/// hook saw vanilla's choice), and whether it was held back.
static MUSIC_SEEN: Mutex<Option<String>> = Mutex::new(None);
static MUSIC_HELD: AtomicBool = AtomicBool::new(false);

/// The layers this mod handed to the music manager on the last client tick, so
/// the command can report them from the server thread.
static LAYERS: Mutex<Vec<(String, f32)>> = Mutex::new(Vec::new());

/// The engine's state. The guard is taken per call and never held across a game
/// read: `music_layers` releases it while it scans the world.
fn engine() -> MutexGuard<'static, Engine> {
    ENGINE
        .lock()
        .expect("the ambient engine lock is never poisoned")
}

// ---------------------------------------------------------------------------
// The pure decisions
// ---------------------------------------------------------------------------

/// Whether a cell sees the sky. `height` is the column's heightmap value, one
/// above the highest block that counts (`LevelReader.getHeight`), so a cell at or
/// above it is open; a column this client holds no chunk for is not.
fn open_to_sky(y: i32, height: Option<i32>) -> bool {
    height.is_some_and(|height| y >= height)
}

/// One cell of the box around the eye: its `y`, its column's heightmap value and
/// whether it is air.
struct Cell {
    y: i32,
    height: Option<i32>,
    air: bool,
}

/// The mean sky light of the box's air, the share of that air open to the sky and
/// the share of the whole box that is air; `None` when the box holds no air.
fn exposure(cells: &[Cell]) -> Option<(f64, f64, f64)> {
    let mut sky_light = 0.0;
    let mut open = 0.0;
    let mut air = 0.0;
    for cell in cells {
        if !cell.air {
            continue;
        }
        air += 1.0;
        if open_to_sky(cell.y, cell.height) {
            sky_light += 15.0;
            open += 1.0;
        }
    }
    if air == 0.0 {
        return None;
    }
    Some((sky_light / air, open / air, air / cells.len() as f64))
}

/// `/ambientsounds engine <state>`'s argument: every spelling the 0.1.0 setting
/// and the Options > Mods toggle can leave behind.
fn engine_setting(word: &str) -> Option<bool> {
    match word {
        "on" | "true" | "1" => Some(true),
        "off" | "false" | "0" => Some(false),
        _ => None,
    }
}

/// `/ambientsounds volume <value>`'s argument, 0 to 1.
fn volume_setting(word: &str) -> Option<f64> {
    word.parse::<f64>()
        .ok()
        .filter(|volume| (0.0..=1.0).contains(volume))
}

// ---------------------------------------------------------------------------
// The settings
// ---------------------------------------------------------------------------

/// The saved settings, read once per session (`engine` on/off and `volume`).
///
/// Both keys are registered with [`declare_settings`], which is what lets this
/// mod read them: `Host::setting` answers only a registered key. A value an older
/// build or the Options page left in another spelling still reads (see
/// [`engine_setting`]).
fn load_settings(host: &dyn Host) {
    if LOADED.swap(true, Ordering::Relaxed) {
        return;
    }
    let enabled = host
        .setting(MOD_ID, "engine")
        .is_none_or(|value| engine_setting(value.trim()).unwrap_or(true));
    let volume = host
        .setting(MOD_ID, "volume")
        .and_then(|value| volume_setting(value.trim()))
        .unwrap_or(1.0);
    let mut engine = engine();
    engine.set_enabled(enabled);
    engine.set_volume(volume);
}

/// The two settings the Options > Mods page shows and the command writes: the
/// engine on or off and its volume. Registering them is what makes the keys
/// readable through `Host::setting`; the page's labels are in
/// `assets/ambientsounds/lang/en_us.json`.
pub fn declare_settings(registrar: &mut dyn Registrar) {
    let _ = registrar.register_setting(
        SettingSpec::toggle("engine", "ambientsounds.setting.engine", true)
            .tooltip("ambientsounds.setting.engine.tooltip"),
    );
    let _ = registrar.register_setting(
        SettingSpec::range(
            "volume",
            "ambientsounds.setting.volume",
            0.0,
            1.0,
            0.01,
            1.0,
        )
        .tooltip("ambientsounds.setting.volume.tooltip"),
    );
}

// ---------------------------------------------------------------------------
// The world reads
// ---------------------------------------------------------------------------

/// Whether a block id is one of the three airs. A position this client holds no
/// chunk for reads as no block and counts as not air, as 0.1.0's scan skipped
/// unloaded columns.
fn is_air(host: &dyn ClientHost, level: LevelRef, pos: [i32; 3]) -> bool {
    level.block(host, pos).is_some_and(|block| {
        matches!(
            block.id(host).as_str(),
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        )
    })
}

/// The columns the average height is taken from
/// (`TerrainEnvironment.analyzeHeight`: 11 x 11 at twice the scan distance,
/// `average-height-scan-distance` / `-count` in `engine.json`), as the height of
/// the top block that counts.
fn terrain(host: &dyn ClientHost, at: [i32; 3]) -> Option<Terrain> {
    let mut columns = Vec::new();
    for dx in -AVERAGE_HEIGHT_SCAN_COUNT..=AVERAGE_HEIGHT_SCAN_COUNT {
        for dz in -AVERAGE_HEIGHT_SCAN_COUNT..=AVERAGE_HEIGHT_SCAN_COUNT {
            columns.push((
                at[0] + dx * AVERAGE_HEIGHT_SCAN_DISTANCE,
                at[2] + dz * AVERAGE_HEIGHT_SCAN_DISTANCE,
            ));
        }
    }
    let heights = host
        .client_surface_heights(&columns, Heightmap::MotionBlocking)
        .ok()?;
    // `getHeight` answers one above the top block; upstream's `getHeightBlock`
    // stops on the block itself (`TerrainEnvironment.java:28-42`).
    let heights: Vec<i32> = heights.into_iter().flatten().map(|top| top - 1).collect();
    Terrain::from_heights(&heights)
}

/// The air box around the eye: the mean sky light of its air, the share of that
/// air open to the sky and the share of the box that is air (`AmbientEnvironment`
/// reads the same three numbers from its air pocket; `notes.md` records why this
/// slice measures a 5 x 5 x 5 box instead).
fn exposure_box(host: &dyn ClientHost, level: LevelRef, at: [i32; 3]) -> Option<(f64, f64, f64)> {
    let offsets: Vec<(i32, i32)> = (-EXPOSURE_RADIUS..=EXPOSURE_RADIUS)
        .flat_map(|dx| (-EXPOSURE_RADIUS..=EXPOSURE_RADIUS).map(move |dz| (dx, dz)))
        .collect();
    let columns: Vec<(i32, i32)> = offsets
        .iter()
        .map(|(dx, dz)| (at[0] + dx, at[2] + dz))
        .collect();
    let heights = host
        .client_surface_heights(&columns, Heightmap::MotionBlocking)
        .ok()?;
    let mut cells = Vec::with_capacity(offsets.len() * (2 * EXPOSURE_RADIUS as usize + 1));
    for ((dx, dz), height) in offsets.iter().zip(heights) {
        for dy in -EXPOSURE_RADIUS..=EXPOSURE_RADIUS {
            let pos = [at[0] + dx, at[1] + dy, at[2] + dz];
            cells.push(Cell {
                y: pos[1],
                height,
                air: is_air(host, level, pos),
            });
        }
    }
    exposure(&cells)
}

/// The biome group: the eye's biome, replaced by a `bad-biomes` one found at the
/// corners of the biome scan (`biome-scan-distance` / `-count`), because the
/// mod's cave region is refused by one bad biome anywhere in its scan.
fn biome_group(host: &dyn ClientHost, level: LevelRef, at: [i32; 3]) -> Option<String> {
    let mut biome = level.client_biome(host, at);
    let reach = BIOME_SCAN_DISTANCE * BIOME_SCAN_COUNT;
    for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let pos = [at[0] + dx * reach, at[1], at[2] + dz * reach];
        if level.is_loaded(host, pos)
            && let Some(id) = level.client_biome(host, pos)
            && biome_is_bad(&id)
        {
            return Some(id);
        }
    }
    biome.take()
}

/// One slow scan of the world, or none when there is no player to scan around.
fn sample(host: &dyn ClientHost) -> Option<Sample> {
    let player = host.local_player()?;
    let eye = player.entity().eye_position(host)?;
    let at = [
        eye[0].floor() as i32,
        eye[1].floor() as i32,
        eye[2].floor() as i32,
    ];
    let dimension = player.dimension(host)?.to_string();
    let level = host.level(&dimension)?;
    if !level.is_loaded(host, at) {
        return None;
    }
    let terrain = terrain(host, at)?;
    let (sky_light, sky, air) = exposure_box(host, level, at)?;
    let (rain, thunder) = level.weather(host).unwrap_or((0.0, 0.0));
    Some(Sample {
        dimension,
        eye_y: eye[1],
        sky_light,
        sky,
        air,
        rain: f64::from(rain),
        thunder: f64::from(thunder),
        biome: biome_group(host, level, at),
        terrain,
    })
}

// ---------------------------------------------------------------------------
// The two music hooks
// ---------------------------------------------------------------------------

/// Once a client tick, before the music manager's own choice: vanilla's track is
/// held back while a bed plays (`SelectMusicEvent`).
pub fn music(host: &dyn ClientHost, event: &mut MusicEvent<'_>) {
    load_settings(host);
    if let Ok(mut seen) = MUSIC_SEEN.lock() {
        *seen = event.music.as_ref().map(|music| music.sound.clone());
    }
    let held = engine().suppresses_vanilla();
    MUSIC_HELD.store(held, Ordering::Relaxed);
    if held {
        *event.music = None;
        event.volume = 0.0;
    }
}

/// Once a client tick, before the music manager ticks its layers: the engine's
/// cadence (a scan every 40 ticks, an aim every 4, an eased step every tick) and
/// the beds as looping layers.
pub fn music_layers(host: &dyn ClientHost, event: &mut MusicLayersEvent<'_>) {
    load_settings(host);
    if RESCAN.swap(false, Ordering::Relaxed) {
        engine().reset();
    }
    let step = engine().step();
    if step.environment {
        // The world reads happen with the engine unlocked: each one is a game
        // call, and the engine is only a value this mod owns.
        match sample(host) {
            Some(sample) => engine().observe(sample),
            None => engine().lost(),
        }
    }
    if step.sound {
        engine().aim();
    }
    engine().tick();
    let wanted = engine().layers();
    for (sound, volume) in &wanted {
        event.layers.push(MusicLayer {
            sound: (*sound).to_owned(),
            volume: *volume,
        });
    }
    if let Ok(mut kept) = LAYERS.lock() {
        *kept = wanted
            .iter()
            .map(|(sound, volume)| ((*sound).to_owned(), *volume))
            .collect();
    }
}

/// A level was made or changed: the scan starts over (`LevelEvent.Load`), as the
/// mod's `loadLevel` drops its scanner.
pub fn client_level_load() {
    engine().reset();
}

// ---------------------------------------------------------------------------
// The harness surface
// ---------------------------------------------------------------------------

/// The engine's status line, plus what this mod's own state says about it:
/// `layers` are the beds it handed the music manager on the last client tick,
/// `music_received` is the track the game chose and `music_held` whether a bed
/// held it back.
pub fn status_line() -> String {
    let mut line = engine().status();
    let layers: Vec<String> = LAYERS
        .lock()
        .map(|kept| {
            kept.iter()
                .map(|(sound, gain)| format!("{sound}:{gain:.3}"))
                .collect()
        })
        .unwrap_or_default();
    line.push_str(&format!(
        " layers={}",
        if layers.is_empty() {
            "none".to_owned()
        } else {
            layers.join(",")
        }
    ));
    let seen = MUSIC_SEEN
        .lock()
        .ok()
        .and_then(|seen| seen.clone())
        .unwrap_or_else(|| "none".to_owned());
    let held = if MUSIC_HELD.load(Ordering::Relaxed) {
        "1"
    } else {
        "0"
    };
    line.push_str(&format!(" music_received={seen} music_held={held}"));
    line
}

/// `/ambientsounds status|engine|volume|rescan`.
pub fn declare_commands(commands: &mut Vec<Command>) {
    // The optional tag is what 0.1.0's handler read off the raw line; a typed
    // command has to declare it or the word never reaches the handler.
    commands.push(Command::new("ambientsounds status [tag:word]"));
    commands.push(Command::new("ambientsounds engine <state:word>"));
    commands.push(Command::new("ambientsounds volume <value:word>"));
    commands.push(Command::new("ambientsounds rescan"));
}

/// A `word` argument of the invocation.
fn argument<'a>(event: &'a RunModCommandEvent<'_>, name: &str) -> Option<&'a str> {
    match event.command.argument(name) {
        Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
        _ => None,
    }
}

/// `/ambientsounds ...`: answer the harness and take the command.
pub fn run_command(host: &mut dyn ServerHost, event: &RunModCommandEvent<'_>) -> Flow {
    if event.command.literals.first().map(String::as_str) != Some(MOD_ID) {
        return Flow::Continue;
    }
    let Some(subcommand) = event.command.literals.get(1).map(String::as_str) else {
        return Flow::Continue;
    };
    match subcommand {
        "status" => {
            // `status <tag>` echoes the tag back, so a caller reading the chat
            // window can tell its own reply from an earlier one.
            let tag = argument(event, "tag").unwrap_or("-");
            let line = format!("{} seq={tag}", status_line());
            let _ = host.commands().feedback(Text::literal(line));
        }
        "engine" => {
            let Some(on) = argument(event, "state").and_then(engine_setting) else {
                let _ = host
                    .commands()
                    .feedback(Text::literal("usage: /ambientsounds engine <on|off>"));
                return Flow::Cancel;
            };
            engine().set_enabled(on);
            host.set_setting(MOD_ID, "engine", if on { "true" } else { "false" });
            let _ = host.commands().feedback(Text::literal(format!(
                "AmbientSounds: engine {}",
                if on { "on" } else { "off" }
            )));
        }
        "volume" => {
            let Some(volume) = argument(event, "value").and_then(volume_setting) else {
                let _ = host
                    .commands()
                    .feedback(Text::literal("usage: /ambientsounds volume <0..1>"));
                return Flow::Cancel;
            };
            engine().set_volume(volume);
            host.set_setting(MOD_ID, "volume", &format!("{volume:.2}"));
            let _ = host
                .commands()
                .feedback(Text::literal(format!("AmbientSounds: volume {volume:.2}")));
        }
        "rescan" => {
            // The command runs on the server, which cannot read the client's
            // level: the next client tick owes the scan and the engine drops what
            // it had, as 0.1.0 reset it here.
            RESCAN.store(true, Ordering::Relaxed);
            let _ = host
                .commands()
                .feedback(Text::literal("AmbientSounds: rescan requested"));
        }
        other => {
            let _ = host.commands().feedback(Text::literal(format!(
                "usage: /ambientsounds status|engine|volume|rescan (not '{other}')"
            )));
        }
    }
    Flow::Cancel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_at_the_heightmap_value_is_open_and_below_it_is_not() {
        assert!(open_to_sky(201, Some(201)));
        assert!(open_to_sky(250, Some(201)));
        assert!(!open_to_sky(200, Some(201)));
        assert!(!open_to_sky(100, Some(201)));
    }

    #[test]
    fn a_column_without_a_chunk_is_not_open() {
        assert!(!open_to_sky(250, None));
    }

    #[test]
    fn exposure_means_the_open_air_and_counts_the_box() {
        let cells = [
            Cell {
                y: 30,
                height: Some(60),
                air: true,
            },
            Cell {
                y: 31,
                height: Some(60),
                air: true,
            },
            Cell {
                y: 62,
                height: Some(60),
                air: true,
            },
            Cell {
                y: 63,
                height: Some(60),
                air: true,
            },
            Cell {
                y: 30,
                height: Some(60),
                air: false,
            },
        ];
        let (sky_light, sky, air) = exposure(&cells).expect("the box holds air");
        assert!((sky_light - 7.5).abs() < 1e-9, "sky_light {sky_light}");
        assert!((sky - 0.5).abs() < 1e-9, "sky {sky}");
        assert!((air - 0.8).abs() < 1e-9, "air {air}");
    }

    #[test]
    fn a_box_with_no_air_has_no_exposure() {
        let cells = [Cell {
            y: 30,
            height: Some(31),
            air: false,
        }];
        assert_eq!(exposure(&cells), None);
    }

    #[test]
    fn the_engine_word_takes_every_spelling_the_old_setting_did() {
        for word in ["on", "true", "1"] {
            assert_eq!(engine_setting(word), Some(true), "{word}");
        }
        for word in ["off", "false", "0"] {
            assert_eq!(engine_setting(word), Some(false), "{word}");
        }
        assert_eq!(engine_setting("maybe"), None);
        assert_eq!(engine_setting(""), None);
    }

    #[test]
    fn the_volume_word_is_a_fraction() {
        assert_eq!(volume_setting("0"), Some(0.0));
        assert_eq!(volume_setting("1"), Some(1.0));
        assert_eq!(volume_setting("0.25"), Some(0.25));
        assert_eq!(volume_setting("1.5"), None);
        assert_eq!(volume_setting("-0.1"), None);
        assert_eq!(volume_setting("loud"), None);
    }
}
