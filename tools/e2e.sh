#!/usr/bin/env bash
# In-game check for AmbientSounds slice 1: the environment engine plays the
# `cave` bed underground, fades it out above ground and hands over to the storm
# `wind` bed in a thunderstorm, all on the mod's own cadence (a scan every 40
# client ticks, an aim every 4, an eased step every tick).
#
# **The 0.1.0 (`.hnmod`) runs are historical.** Logs in
# `ports/_shared/evidence/ambientsounds/` (`e2e.log`, `e2e-embedded.log`,
# `e2e-selftest.log`) were taken on a game built by the retired `tools/horizon.py`
# with the bundle compiled in. The 0.2.0 package is a flat SDK `.hmod`: it needs
# a game **built by the launcher** (`horizon-launcher native-build <port>
# --instance <i> --trust-native-code`), which is not installed in the campaign's
# environment, so no 0.2.0 leg has been run. Pass that binary as `GAME=`.
#
# What is asserted, and where it comes from:
#   * the classification, from `/ambientsounds status` (the port's own read of
#     sky exposure, depth, weather and biome);
#   * that the beds are handed to the game, from the same status line's
#     `layers=` (what the port pushed this tick) and `music_held=`;
#   * that vanilla's music is held back, from `vanilla=held` plus the
#     `music_received`/`music_held` the music hook recorded. The prepared 26.3
#     resources carry no vanilla sounds (see notes.md), so no vanilla track can
#     start here and `status.music` stays null in both states; the hook's own
#     input is the evidence that it ran and was overridden.
#
# Every mod command is typed with `ctl chat "/cmd"` because `ctl command` runs
# the console path and bypasses `run_mod_command`.
#
# Arenas:
#   * the cave: a sealed 9x11x9 stone shell at (400, 24..38, 500) in the
#     Overworld, hollowed to 7x7x7 air, the player standing at y=30 — no sky in
#     the box, the terrain's columns ~40 blocks above the eye;
#   * the surface platform: 21x21 stone at y=200 around (400, 200, 400), which is
#     exactly the height scan's reach (11 columns each way at 2 blocks), so every
#     sampled column reads 200 and the surface region sees the eye 1 block up;
#   * the empty level: a stone floor at (200, 100, 0) in the End, where the port
#     plays no bed at all (the dangling `bad-regions: ["end"]` fix).
#
# GAME=<launcher-built horizon-client>   the game carrying this package (required)
# CONTROL_GAME=<the same build without it>  optional counterfactual: the command
#                                            does not exist there
# SELFTEST=1 flips the cave assertion (it must FAIL), proving the run can fail.
set -Eeuo pipefail
export LC_ALL=C

SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}

CAMPAIGN=${CAMPAIGN:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CAMPAIGN/external/ambientsounds
CTL=${CTL:-$(command -v horizon-ctl || true)}
GAME=${GAME:-}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/ambientsounds-e2e.sock}
WORLD=AmbientSoundsE2E
CONTROL_WORLD=AmbientSoundsControl

CAVE_BED=ambientsounds:cave.cave1
WIND_BED=ambientsounds:wind.light-wind
CAVE_X=400
CAVE_Y=30
CAVE_Z=500
PLATFORM_X=400
PLATFORM_Y=201
PLATFORM_Z=400
END_X=200
END_Y=101

command -v jq >/dev/null || { echo "missing jq (every assertion parses a ctl reply)" >&2; exit 1; }
[ -n "$CTL" ] && [ -x "$CTL" ] || { echo "missing horizon-ctl: set CTL=<path>" >&2; exit 1; }
[ -n "$GAME" ] || {
    echo "missing GAME: this package is a native .hmod, so it needs a game" >&2
    echo "  built by 'horizon-launcher native-build' with it installed" >&2
    exit 1
}
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/ambientsounds-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
# The pin's mixer runs without a device; the census still lists the events
# holding channels, which is what the bed assertions read.
export HORIZON_NO_AUDIO=1
export HORIZON_LAUNCHER_NO_SELF_UPDATE=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 60 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 40 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch [WORLD] [GAME]
    local world=${1:-$WORLD}
    local game=${2:-$GAME}
    rm -f "$SOCK"
    cd "$SCRATCH"
    # No --dev-mod: the launcher built this game from the package, content and
    # code together, so nothing is loaded from a folder.
    "$CTL" --socket "$SOCK" launch --game "$game" --world "$world" \
        --seed 20261105 --creative --headless --wait --resources "$RES" >/dev/null
    ctl wait-ready 120
    # The window never takes focus headless, and a pause stops the music manager.
    ctl options-set pauseOnLostFocus false >/dev/null || true
    ctl resume >/dev/null 2>&1 || true
    ctl wait 40
}

# `/ambientsounds status <tag>` -> STATUS, then `field KEY`. The tag comes back
# in the reply, so a stale line already in the chat window is never mistaken for
# this call's answer.
SEQ=0
status() {
    SEQ=$((SEQ + 1))
    local tag="t$SEQ"
    say "ambientsounds status $tag"
    STATUS=""
    for _ in $(seq 1 25); do
        ctl wait 2 >/dev/null
        STATUS=$(ctl messages 40 | jq -r '.chat[]' | grep -F " seq=$tag" | tail -1 || true)
        [ -n "$STATUS" ] && return 0
    done
    fail "no /ambientsounds status reply for $tag"
}
field() { printf '%s\n' "$STATUS" | tr ' ' '\n' | sed -n "s/^$1=//p" | head -1; }

expect() { # expect KEY VALUE
    local got
    got=$(field "$1")
    [ "$got" = "$2" ] || fail "$1=$got, expected $2 (line: $STATUS)"
}
expect_at_least() { # expect_at_least KEY MIN
    python3 -c "import sys; sys.exit(0 if float(sys.argv[1]) >= float(sys.argv[2]) else 1)" \
        "$(field "$1")" "$2" || fail "$1=$(field "$1") is below $2 (line: $STATUS)"
}
expect_above() { # expect_above KEY VALUE
    python3 -c "import sys; sys.exit(0 if float(sys.argv[1]) > float(sys.argv[2]) else 1)" \
        "$(field "$1")" "$2" || fail "$1=$(field "$1") is not above $2 (line: $STATUS)"
}
expect_below() { # expect_below KEY VALUE
    python3 -c "import sys; sys.exit(0 if float(sys.argv[1]) < float(sys.argv[2]) else 1)" \
        "$(field "$1")" "$2" || fail "$1=$(field "$1") is not below $2 (line: $STATUS)"
}
expect_value_above() { # expect_value_above VALUE MIN LABEL
    python3 -c "import sys; sys.exit(0 if float(sys.argv[1]) > float(sys.argv[2]) else 1)" "$1" "$2" \
        || fail "$3=$1 is not above $2"
}
expect_value_near() { # expect_value_near VALUE WANT TOLERANCE LABEL
    python3 -c "import sys; sys.exit(0 if abs(float(sys.argv[1]) - float(sys.argv[2])) <= float(sys.argv[3]) else 1)" "$1" "$2" "$3" \
        || fail "$4=$1 is not $2 (+-$3)"
}
expect_near() { # expect_near KEY VALUE TOLERANCE
    python3 -c "import sys; sys.exit(0 if abs(float(sys.argv[1]) - float(sys.argv[2])) <= float(sys.argv[3]) else 1)" \
        "$(field "$1")" "$2" "$3" || fail "$1=$(field "$1") is not $2 (+-$3) (line: $STATUS)"
}

# The layers this port handed the music manager on the last client tick, and the
# volume it gave the bed (`layers=<event>:<gain>,…` in the status line).
layer_gain() { # layer_gain EVENT -> sets GAIN
    GAIN=$(printf '%s\n' "$STATUS" | tr ' ' '\n' | sed -n "s/^layers=//p" | head -1 \
        | tr ',' '\n' | grep -F "$1:" | head -1 | sed 's/.*://' || true)
    [ -n "$GAIN" ] || fail "the port holds no layer $1 (layers=$(field layers))"
}
no_layer() { # no_layer EVENT
    local found
    found=$(printf '%s\n' "$STATUS" | tr ' ' '\n' | sed -n "s/^layers=//p" | head -1 \
        | tr ',' '\n' | grep -F "$1:" || true)
    [ -z "$found" ] || fail "the port still holds $1: $(field layers)"
}
# This box has no audio device, so the pin's SoundEngine is not loaded and no
# channel is ever created: the mixer's census stays empty for every sound. Print
# it, so the log shows the limitation rather than hiding it.
census_note() {
    echo "  (mixer census with no audio device: $(ctl status | jq -c '.sounds.events'))"
}

# Wait until KEY is at least VALUE, reading the status every 10 ticks.
wait_for() { # wait_for KEY VALUE
    for _ in $(seq 1 30); do
        status
        python3 -c "import sys; sys.exit(0 if float(sys.argv[1]) >= float(sys.argv[2]) else 1)" \
            "$(field "$1")" "$2" && return 0
        ctl wait 10 >/dev/null
    done
    fail "$1 never reached $2 (line: $STATUS)"
}

block_name() { ctl block "$1" "$2" "$3" | jq -r '.block // .name // empty'; }

# Wait until KEY is at most VALUE (a fade reaching zero).
wait_until_at_most() { # wait_until_at_most KEY VALUE
    for _ in $(seq 1 40); do
        status
        python3 -c "import sys; sys.exit(0 if float(sys.argv[1]) <= float(sys.argv[2]) else 1)" \
            "$(field "$1")" "$2" && return 0
        ctl wait 10 >/dev/null
    done
    fail "$1 never fell to $2 (line: $STATUS)"
}

wait_loaded() { # wait_loaded X Z
    for _ in $(seq 1 120); do
        ctl wait 5 >/dev/null
        if [ "$(ctl block "$1" 100 "$2" | jq -r '.loaded // false')" = true ]; then
            return 0
        fi
    done
    fail "the chunk at $1,$2 never loaded"
}

# The sealed cave: a stone shell with a 7x7x7 air room inside it.
build_cave() {
    ctl command "execute in minecraft:overworld run tp @s $CAVE_X.5 100 $CAVE_Z.5" >/dev/null
    wait_loaded "$CAVE_X" "$CAVE_Z"
    ctl command "execute in minecraft:overworld run fill $((CAVE_X - 4)) 24 $((CAVE_Z - 4)) $((CAVE_X + 4)) 38 $((CAVE_Z + 4)) minecraft:stone" >/dev/null
    ctl command "execute in minecraft:overworld run fill $((CAVE_X - 3)) 27 $((CAVE_Z - 3)) $((CAVE_X + 3)) 35 $((CAVE_Z + 3)) minecraft:air" >/dev/null
    ctl wait 10 >/dev/null
    ctl command "execute in minecraft:overworld run tp @s $CAVE_X.5 $CAVE_Y $CAVE_Z.5" >/dev/null
    # The scan runs once every 40 ticks (2 s); a status read sooner answers for
    # where the player stood before the move.
    ctl wait 60 >/dev/null
    [ "$(block_name "$CAVE_X" 37 "$CAVE_Z")" = "minecraft:stone" ] \
        || fail "the cave roof is not there: $(block_name "$CAVE_X" 37 "$CAVE_Z")"
    [ "$(block_name "$CAVE_X" "$CAVE_Y" "$CAVE_Z")" = "minecraft:air" ] \
        || fail "the cave room is not hollow: $(block_name "$CAVE_X" "$CAVE_Y" "$CAVE_Z")"
    [ "$(ctl light "$CAVE_X" "$CAVE_Y" "$CAVE_Z" "$CAVE_X" "$CAVE_Y" "$CAVE_Z" | jq -r '.sky // empty')" = 0 ] \
        || fail "the cave room is not dark: $(ctl light "$CAVE_X" "$CAVE_Y" "$CAVE_Z" "$CAVE_X" "$CAVE_Y" "$CAVE_Z" | jq -c .)"
}

# The surface platform: 21x21, exactly the height scan's reach.
build_platform() {
    ctl command "execute in minecraft:overworld run tp @s $PLATFORM_X.5 250 $PLATFORM_Z.5" >/dev/null
    wait_loaded "$PLATFORM_X" "$PLATFORM_Z"
    ctl command "execute in minecraft:overworld run fill $((PLATFORM_X - 10)) 200 $((PLATFORM_Z - 10)) $((PLATFORM_X + 10)) 200 $((PLATFORM_Z + 10)) minecraft:stone" >/dev/null
    ctl wait 10 >/dev/null
    ctl command "execute in minecraft:overworld run tp @s $PLATFORM_X.5 $PLATFORM_Y $PLATFORM_Z.5" >/dev/null
    ctl wait 60 >/dev/null
    [ "$(block_name "$PLATFORM_X" 200 "$PLATFORM_Z")" = "minecraft:stone" ] \
        || fail "the platform is not there: $(block_name "$PLATFORM_X" 200 "$PLATFORM_Z")"
}

# The End's void: a stone floor, no weather, no bed.
build_void() {
    ctl command "execute in minecraft:the_end run tp @s $END_X.5 110 0.5" >/dev/null
    wait_loaded "$END_X" 0
    ctl command "execute in minecraft:the_end run fill $((END_X - 4)) 100 -4 $((END_X + 4)) 100 4 minecraft:stone" >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
    ctl wait 10 >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $END_X.5 $END_Y 0.5" >/dev/null
    ctl wait 60 >/dev/null
}

# The cave leg: the bed, its rise, and the engine playing it.
cave_leg() {
    echo "== A: underground the cave bed plays and its volume rises =="
    build_cave
    status
    [ "$(field dimension)" = "minecraft:overworld" ] || fail "dimension $(field dimension)"
    expect sky_light 0.00
    expect sky 0.00
    expect region_cave 1.000
    expect region_sky 0.000
    expect_below relative -10
    if [ "$SELFTEST" = 1 ]; then
        echo "(selftest: asserting the wind bed underground)"
        expect bed wind
    else
        expect bed cave
    fi
    echo "  $STATUS"
    local first second
    first=$(field current)
    ctl wait 20 >/dev/null
    status
    second=$(field current)
    echo "  current $first -> $second after 20 ticks"
    expect_above current "$first"
    wait_for current 0.500
    expect_near aimed 0.500 0.0001
    expect vanilla held
    expect music_held 1
    layer_gain "$CAVE_BED"
    expect_value_above "$GAIN" 0.400 "the cave layer gain"
    no_layer "$WIND_BED"
    echo "  the port holds $(field layers)"
    census_note
    case "$(field music_received)" in
        minecraft:music.*) ;;
        *) fail "the music hook saw '$(field music_received)', not a vanilla music event" ;;
    esac
    echo "  the music hook saw $(field music_received) and held it (music_held=$(field music_held))"
}

# The surface leg: the cave bed fades out and nothing takes its place in clear
# weather.
surface_leg() {
    echo "== B: above ground the cave bed fades out =="
    build_platform
    status
    expect region_cave 0.000
    expect_at_least sky_light 14.00
    expect_at_least region_surface 0.800
    expect_below relative 15
    # The aim is the new environment at once; the bed itself fades out over the
    # mod's 0.005 a tick, so it is still audible for a moment and vanilla's music
    # stays held until it is gone.
    expect_near aimed 0.000 0.0001
    local before
    before=$(field cave)
    ctl wait 20 >/dev/null
    status
    echo "  cave $before -> $(field cave) after 20 ticks"
    expect_below cave "$before"
    wait_until_at_most cave 0.000
    status
    expect bed none
    expect vanilla plays
    expect music_held 0
    no_layer "$CAVE_BED"
    no_layer "$WIND_BED"
    echo "  the cave bed faded to $(field cave) and the port holds no layer"
}

# The weather leg: a thunderstorm at ground level hands over to the storm wind.
weather_leg() {
    echo "== C: in a thunderstorm the wind bed takes over =="
    ctl command "weather thunder 10000" >/dev/null
    status
    for _ in $(seq 1 30); do
        status
        if [ "$(field rain)" = "1.00" ] && [ "$(field thunder)" = "1.00" ]; then
            break
        fi
        ctl wait 10 >/dev/null
    done
    expect rain 1.00
    expect thunder 1.00
    expect_at_least region_storm 0.800
    expect bed wind
    wait_for wind 0.400
    expect_near wind 0.400 0.001
    expect vanilla held
    layer_gain "$WIND_BED"
    expect_value_near "$GAIN" 0.400 0.010 "the wind layer gain"
    no_layer "$CAVE_BED"
    echo "  $STATUS"
    ctl command "weather clear 10000" >/dev/null
    ctl wait 40 >/dev/null
}

# The empty-level leg: an environment with no bed stays silent.
void_leg() {
    echo "== D: control, an environment with no bed stays silent =="
    build_void
    status
    [ "$(field dimension)" = "minecraft:the_end" ] || fail "dimension $(field dimension)"
    expect region_cave 0.000
    expect region_sky 0.000
    expect bed none
    ctl wait 40 >/dev/null
    status
    expect current 0.000
    expect vanilla plays
    no_layer "$CAVE_BED"
    no_layer "$WIND_BED"
    echo "  the End: bed=$(field bed), no layer held"
}

# The disabled-engine control: nothing plays and vanilla keeps its music.
disabled_leg() {
    echo "== E: control, a disabled engine plays nothing =="
    ctl command "execute in minecraft:overworld run tp @s $CAVE_X.5 100 $CAVE_Z.5" >/dev/null
    ctl command "execute in minecraft:overworld run tp @s $CAVE_X.5 $CAVE_Y $CAVE_Z.5" >/dev/null
    ctl wait 60 >/dev/null
    say "ambientsounds engine off"
    ctl wait 6 >/dev/null
    status
    expect engine off
    expect_near aimed 0.000 0.0001
    wait_until_at_most cave 0.000
    status
    expect bed none
    expect vanilla plays
    no_layer "$CAVE_BED"
    echo "  engine off in the same cave: $(field bed), cave=$(field cave), no layer held"
    say "ambientsounds engine on"
    wait_for current 0.500
    status
    expect bed cave
    layer_gain "$CAVE_BED"
    echo "  engine on again: $(field bed) at $(field current), the layer is back at $GAIN"
}

# The 0.1.0 audio-census limit is unchanged: a box with no audio device plays no
# instance, so the beds are read from the port's own layer record and the hook's
# output rather than from `sounds.events` (see notes.md, "Verification limits").
#
# CONTROL_GAME: the same build without this package. `/ambientsounds` does not
# exist there, so the readouts above are the package's own and not the game's.
control_game_leg() {
    echo "== control: a build without the package has no /ambientsounds =="
    launch "$CONTROL_WORLD" "$CONTROL_GAME"
    build_cave
    say "ambientsounds status t0"
    ctl wait 10 >/dev/null
    if ctl messages 40 | jq -r '.chat[]' | grep -F " seq=t0" >/dev/null; then
        fail "the control game answered /ambientsounds: the readouts are not the package's"
    fi
    echo "  the control game answered no /ambientsounds status (no ' seq=t0' line)"
    ctl quit >/dev/null 2>&1 || true
    rm -f "$SOCK"
}

echo "== launch (Overworld arenas, launcher-built game) =="
if [ -n "$CONTROL_GAME" ]; then control_game_leg; fi
launch
ctl options-set musicFrequency CONSTANT >/dev/null || true

cave_leg
surface_leg
weather_leg
void_leg
disabled_leg

echo "PASS: underground the cave bed is held at 0.500 while vanilla's music is held back, above ground it fades to 0.000 and the layer is dropped, a thunderstorm hands over to the wind bed at 0.400, the End holds no bed, and a disabled engine holds nothing"
