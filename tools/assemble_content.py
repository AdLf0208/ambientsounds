#!/usr/bin/env python3
"""Carry the two `.ogg` takes AmbientSounds' `cave` and `wind` beds play.

Reads the exact jar (`ports/_shared/licence-cache/AmbientSounds_NEOFORGE_v6.3.11_mc26.3.jar`,
LGPL-3.0-only) and writes, at the package root (the SDK layout: one flat `.hmod`
carries `assets/` beside `src/`):

  assets/ambientsounds/sounds/cave/cave1.ogg      the cave bed's take
  assets/ambientsounds/sounds/wind/light-wind.ogg the wind bed's take
  assets/ambientsounds/sounds.json                those two events, verbatim

The other 118 `.ogg` files (81.6 MB of the jar's 84.8 MB) are **not** carried:
they belong to the beds, animals and water sounds this slice does not play (see
`../notes.md`, gap list). The events are copied verbatim from the jar's own
`sounds.json` so the `stream: true` flags stay the mod's.

    python3 tools/assemble_content.py            # write assets/ at the package root
    python3 tools/assemble_content.py --check    # the carried files are the jar's
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CAMPAIGN = next(p for p in ROOT.parents if (p / "ports/_shared").is_dir())
JAR = CAMPAIGN / "ports/_shared/licence-cache/AmbientSounds_NEOFORGE_v6.3.11_mc26.3.jar"
PACKAGE = ROOT
NAMESPACE = "ambientsounds"

# The two sound events this slice plays (`regions/cave.ambience.json`'s `cave`
# files and `sound_collections/wind.json`'s `light-wind` files).
EVENTS = ("cave.cave1", "wind.light-wind")


def carried() -> dict[str, bytes]:
    """The files the package carries, keyed by their path inside it."""
    with zipfile.ZipFile(JAR) as jar:
        sounds = json.loads(jar.read(f"assets/{NAMESPACE}/sounds.json"))
        events = {}
        for name in EVENTS:
            if name not in sounds:
                raise SystemExit(f"{JAR.name} has no sound event {name}")
            # Each take must be a single streamed file: the pin's music layers
            # play one looping track per layer.
            entries = sounds[name]["sounds"]
            if len(entries) != 1 or not entries[0].get("stream"):
                raise SystemExit(f"{name} is not one streamed file: {entries}")
            events[name] = sounds[name]
        files = {
            f"assets/{NAMESPACE}/sounds.json":
                (json.dumps(events, indent=4, sort_keys=True) + "\n").encode(),
        }
        for name, event in events.items():
            for entry in event["sounds"]:
                # A sound event `ambientsounds:cave/cave1` is played from
                # `assets/ambientsounds/sounds/cave/cave1.ogg`.
                path = f"assets/{NAMESPACE}/sounds/{entry['name'].split(':')[1]}.ogg"
                files[path] = jar.read(path)
    return files


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true", help="verify, do not write")
    args = parser.parse_args()
    files = carried()
    if args.check:
        bad = 0
        for relative, expected in files.items():
            path = PACKAGE / relative
            if not path.is_file():
                print(f"missing: {relative}", file=sys.stderr)
                bad += 1
            elif path.read_bytes() != expected:
                print(f"differs from the jar: {relative}", file=sys.stderr)
                bad += 1
        extra = sorted(
            str(path.relative_to(PACKAGE))
            for path in (PACKAGE / "assets" / NAMESPACE).rglob("*.ogg")
            if str(path.relative_to(PACKAGE)) not in files
        )
        for relative in extra:
            print(f"carried but not in the slice: {relative}", file=sys.stderr)
            bad += 1
        if bad:
            return 1
        print(f"assets: {len(files) - 1} .ogg file(s) + sounds.json, identical to the jar")
        return 0
    for relative, content in files.items():
        path = PACKAGE / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
    total = sum(len(content) for relative, content in files.items() if relative.endswith(".ogg"))
    print(f"carried {len(files) - 1} .ogg file(s), {total} B, and {len(EVENTS)} sound events")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
