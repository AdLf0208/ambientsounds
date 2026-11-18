//! AmbientSounds for Horizon — slice 1: the environment engine and two beds.
//!
//! AmbientSounds (see `FINDINGS.md`) decides what the player hears from where
//! they are: a slow scan of the terrain, the sky exposure, the weather and the
//! biome, then the mod's own region conditions easing a bed's volume up or down.
//! This slice carries the `cave` and the `wind` bed and the regions that decide
//! them (`cave`, `cave.ambience`, `surface`, `sky`, `storm.close`), plays them
//! through the game's music layers, and holds vanilla's own music back while a
//! bed is audible.
//!
//! [`runtime`] is the pure engine — the conditions, the numbers from the jar's
//! `basic/engine.json`, the cadence, the easing and the ducking — with its own
//! tests and no `mod_api` types; it is byte-identical to the retired `.hnmod`'s
//! `runtime.rs`. [`support`] is the glue: the world reads and the `/ambientsounds`
//! harness surface. This file is the one `impl Hooks`.
//!
//! The two `.ogg` takes and their `sounds.json` entries travel in the same
//! package (`assets/ambientsounds/`): one `.hmod` is content and code alike.
pub mod runtime;
pub mod support;

use mod_api::{
    self, ClientHost, ClientLevelLoadEvent, Flow, Hooks, MusicEvent, MusicLayersEvent,
    RegisterCommandsEvent, Registrar, RegistrationContext, RunModCommandEvent, ServerHost,
};

/// The mod keeps its state in `support`'s statics, which outlive this value.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the two settings the Options > Mods page shows, which is also
    /// what lets this mod read them back (`Host::setting` answers a registered
    /// key only).
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        support::declare_settings(registrar);
    }

    /// Vanilla's music choice, once a client tick: a bed playing hands back no
    /// track and volume 0 (`SelectMusicEvent`).
    fn music(&self, host: &dyn ClientHost, event: &mut MusicEvent<'_>) {
        support::music(host, event);
    }

    /// Once a client tick: the engine's cadence, and the beds as looping layers.
    fn music_layers(&self, host: &dyn ClientHost, event: &mut MusicLayersEvent<'_>) {
        support::music_layers(host, event);
    }

    /// A level was made or changed: the scan starts over.
    fn client_level_load(&self, _host: &mut dyn ClientHost, _event: &mut ClientLevelLoadEvent<'_>) {
        support::client_level_load();
    }

    /// `/ambientsounds status|engine|volume|rescan`: the surface the harness
    /// reads the bed, the volumes and the scan with.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        support::declare_commands(event.commands);
    }

    /// A player ran one of the commands above: answer it and take it.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        support::run_command(host, event)
    }
}
