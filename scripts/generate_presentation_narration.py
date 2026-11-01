#!/usr/bin/env python3
"""
Generates the 10 deep-bass cinematic narrator MP3 clips for the
Cyber-Athletic Coliseum Web Video Presentation using ElevenLabs.
"""

import json
import sys
import time
import urllib.request
from pathlib import Path

# Reuse API key loader from generate_voices.py
sys.path.insert(0, str(Path(__file__).resolve().parent))
import generate_voices as gv

# ElevenLabs "Brian" - Deep, Resonant, Cinematic Bass Narrator
NARRATOR_VOICE_ID = "nPczCjzI2devNBz1zQrb"
MODEL_ID = "eleven_multilingual_v2"

SCENES = [
    (
        "scene_01",
        "Welcome to the Cyber-Athletic Coliseum. A high-voltage, three-versus-three Capture the Flag arena, engineered from the ground up in Rust, Bevy, and WebAssembly.",
    ),
    (
        "scene_02",
        "Inside the sixty-by-forty-two meter Delta Sector Compound, Blue Squad and Red Squad clash across elevated platforms, central ruins, and a flooded sewer trench. First team to five captures rules the arena.",
    ),
    (
        "scene_pillars",
        "Three core technologies power the engine. Bevy drives the Entity-Component-System, WebGL2 rendering, and skeletal animations. Avian 3D handles 120-hertz capsule physics and spatial raycasts, while wasm-bindgen bridges Rust directly with browser JavaScript and HTML5 audio.",
    ),
    (
        "scene_03",
        "The runtime pairs a lightweight HTML5 host with a compiled WebAssembly core. JavaScript manages the cinematic intro and low-latency audio pools, while Bevy drives the WebGL2 canvas through a wasm-bindgen bridge.",
    ),
    (
        "scene_04",
        "Every gameplay domain is isolated as a modular Bevy plugin. Arena, Capture the Flag, Player, AI, Combat, Camera, and UI systems operate on shared Entity-Component data with zero tight coupling.",
    ),
    (
        "scene_05",
        "Each frame executes a deterministic pipeline. Input and AI feed into combat and scoring chains, followed by Avian 3D physics substeps, boundary clamping, and procedural skeletal bone overrides.",
    ),
    (
        "scene_06",
        "Flags transition dynamically between base, carrier, and dropped states. Meanwhile, a five-tier radio arbitrator filters competing combat events, ensuring only the highest-priority tactical callout hits the airwaves.",
    ),
    (
        "scene_07",
        "To sustain sixty frames per second in the browser, expensive terrain raycasts are replaced by an O-of-one bilinear heightfield. A three-layer camera spring-arm prevents wall clipping and elevates smoothly near perimeter fences.",
    ),
    (
        "scene_08",
        "Character models combine world-space retargeted animations with procedural PostUpdate inverse kinematics—locking arm swing to footfalls and driving explosive kinetic shockwave punches.",
    ),
    (
        "scene_09",
        "Five autonomous bots evaluate a real-time tactical decision tree—scaling from Rookie to Mastermind to lead targets, screen allied flag carriers, and route across elevated platform ramps.",
    ),
    (
        "scene_10",
        "Compilation syncs seamlessly from Windows to native WSL2 storage. Trunk and wasm-bindgen package the optimized binary, verified by exact process monitors and served live to your browser.",
    ),
]


def main():
    api_key = gv.get_api_key()
    if not api_key:
        print("[ERROR] ELEVENLABS_API_KEY not found.")
        sys.exit(1)

    out_dir = Path(__file__).resolve().parent.parent / "presentation" / "audio"
    out_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 68)
    print("  Generating Deep Bass Cinematic Narration (Voice: Brian)")
    print(f"  Output Directory: {out_dir}")
    print("=" * 68)

    manifest = []
    for idx, (scene_id, text) in enumerate(SCENES, start=1):
        out_path = out_dir / f"{scene_id}.mp3"
        print(f"  [{idx:02d}/10] {scene_id}.mp3 -> \"{text[:55]}...\"")
        gv.generate_speech(
            api_key=api_key,
            voice_id=NARRATOR_VOICE_ID,
            text=text,
            output_path=out_path,
            model_id=MODEL_ID,
            stability=0.62,
            similarity_boost=0.82,
            style=0.22,
        )
        size_kb = out_path.stat().st_size / 1024.0
        print(f"          Saved ({size_kb:.1f} KB)")
        manifest.append({"id": scene_id, "file": f"audio/{scene_id}.mp3", "text": text})
        time.sleep(0.25)

    manifest_path = out_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print("=" * 68)
    print(f"  All 10 narration tracks generated successfully! Manifest: {manifest_path}")
    print("=" * 68)


if __name__ == "__main__":
    main()
