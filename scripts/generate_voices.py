#!/usr/bin/env python3
"""
Cyber-Athletic Coliseum - Voice Generation Script
Connects to the ElevenLabs REST API to generate high quality character speech audio
for all lines specified in SCREEN_MESSAGES.md.

Outputs directly to:
    assets/voices/<character>/<line_id>.mp3
"""

import os
import sys
import time
import json
import argparse
import urllib.request
import urllib.error
from pathlib import Path

# ==============================================================================
# Voice Profiles & Character Mapping
# ------------------------------------------------------------------------------
# Replace voice IDs with your own custom cloned voice IDs or choose from ElevenLabs
# standard library voices:
#   - Adam:   pNInz6obpgDQGcFmaJgB (Commanding, confident, leader)
#   - Josh:   TxGEqnHWrfWFTfGW9XjX (Energetic, youthful striker)
#   - Daniel: onwK4e9ZLuTAKqWW03F9 (Calm, tactical defender)
#   - Rachel: 21m00Tcm4TlvDq8ikWAM (Fast, competitive striker)
#   - Nicole: piTKgcLEGmPE4e6mEKli (Quick, sharp scout)
#   - George: JBFqnCBsd6RMkjVDRZzb (Deep, heavy enforcer)
# ==============================================================================
CHARACTER_CONFIG = {
    "anders": {
        "display_name": "ANDERS",
        "team": "BLUE",
        "role": "Squad Leader",
        "voice_id": "pNInz6obpgDQGcFmaJgB",  # Adam
        "stability": 0.55,
        "similarity_boost": 0.75,
        "style": 0.15,
    },
    "morten": {
        "display_name": "MORTEN",
        "team": "BLUE",
        "role": "Vanguard / Flag Striker",
        "voice_id": "TxGEqnHWrfWFTfGW9XjX",  # Josh
        "stability": 0.45,
        "similarity_boost": 0.75,
        "style": 0.25,
    },
    "jonas": {
        "display_name": "JONAS",
        "team": "BLUE",
        "role": "Base Defender / Tactical Comms",
        "voice_id": "onwK4e9ZLuTAKqWW03F9",  # Daniel
        "stability": 0.65,
        "similarity_boost": 0.80,
        "style": 0.05,
    },
    "siri": {
        "display_name": "SIRI",
        "team": "RED",
        "role": "Lead Striker / Flag Runner",
        "voice_id": "21m00Tcm4TlvDq8ikWAM",  # Rachel
        "stability": 0.40,
        "similarity_boost": 0.75,
        "style": 0.20,
    },
    "lisa": {
        "display_name": "LISA",
        "team": "RED",
        "role": "Flanker / Scout",
        "voice_id": "piTKgcLEGmPE4e6mEKli",  # Nicole
        "stability": 0.45,
        "similarity_boost": 0.75,
        "style": 0.15,
    },
    "per": {
        "display_name": "PER",
        "team": "RED",
        "role": "Heavy Defender / Enforcer",
        "voice_id": "JBFqnCBsd6RMkjVDRZzb",  # George
        "stability": 0.60,
        "similarity_boost": 0.80,
        "style": 0.10,
    },
}

# Catalog of all lines from SCREEN_MESSAGES.md
VOICE_LINES = {
    "anders": [
        ("match_start", "Capture the Red Flag on the East Platform and bring it to Blue Base!"),
        ("took_flag", "I've got the Red Flag! Cover me back to Blue Base!"),
        ("dropped_flag", "I dropped the Red Flag! Someone grab it!"),
        ("returned_flag", "I returned our Blue Flag to base!"),
        ("captured_flag", "Flag captured! Plus one point for Blue Team!"),
        ("tackle_hit", "Target neutralized! Stay down!"),
        ("flag_stolen_alert", "Our flag was stolen! Tackle them!"),
        ("escort_carrier", "Our teammate has the Red Flag! Clear the way to Blue Base!"),
        ("victory", "Victory! Blue Team won Capture the Flag!"),
    ],
    "morten": [
        ("took_flag", "I took the Red Flag! Run to Blue Base!"),
        ("dropped_flag", "I lost the Red Flag! Pick it up!"),
        ("returned_flag", "Blue Flag returned to base! We're safe!"),
        ("captured_flag", "Touchdown! I captured the Red Flag for Blue Team!"),
        ("tackle_hit", "Boom! Direct tackle hit!"),
        ("escort_carrier", "Anders has the Red Flag! Clear the way to Blue Base!"),
    ],
    "jonas": [
        ("took_flag", "Red Flag secured! Heading back to Blue Base!"),
        ("dropped_flag", "Dropped the Red Flag! Recover it fast!"),
        ("returned_flag", "I recovered our Blue Flag! Base is secure!"),
        ("flag_auto_returned", "Blue Flag automatically returned to base!"),
        ("captured_flag", "Red Flag delivered! Point for Blue Team!"),
        ("tackle_hit", "Enemy stopped in their tracks!"),
        ("both_flags_stolen", "Both flags stolen! Protect our carrier and tackle theirs!"),
    ],
    "siri": [
        ("took_flag", "I stole the Blue Flag! Catch me if you can!"),
        ("dropped_flag", "Ugh, I dropped the Blue Flag! Recover it!"),
        ("returned_flag", "I returned the Red Flag to our base!"),
        ("captured_flag", "I captured the Blue Flag! Point for Red Team!"),
        ("tackle_hit", "Too slow! Took you down!"),
        ("victory", "Victory for Red Team! We own this arena!"),
    ],
    "lisa": [
        ("took_flag", "Blue Flag is mine! Sprinting to Red Base!"),
        ("dropped_flag", "I lost the Blue Flag! Don't let them touch it!"),
        ("returned_flag", "Red Flag is back at base!"),
        ("captured_flag", "Score! Another flag capture for Red Team!"),
        ("tackle_hit", "Caught you in the net!"),
    ],
    "per": [
        ("took_flag", "I have the Blue Flag! Clear the way to Red Base!"),
        ("dropped_flag", "Dropped the Blue Flag! Hold them back!"),
        ("returned_flag", "I secured the Red Flag! Back at base!"),
        ("flag_auto_returned", "Red Flag automatically returned to base!"),
        ("captured_flag", "Blue Flag delivered to Red Base! Point scored!"),
        ("tackle_hit", "Sit down! Nobody gets past me!"),
    ],
}


def get_api_key(explicit_key: str | None = None) -> str | None:
    """Gets API key from argument, environment, or Windows registry fallback."""
    if explicit_key:
        return explicit_key
    
    # Check current process environment
    env_val = os.environ.get("ELEVENLABS_API_KEY")
    if env_val:
        # If running on Windows, verify if the User environment has an updated key
        if sys.platform == "win32":
            try:
                import winreg
                with winreg.OpenKey(winreg.HKEY_CURRENT_USER, r"Environment") as k:
                    user_val, _ = winreg.QueryValueEx(k, "ELEVENLABS_API_KEY")
                    if user_val:
                        return user_val
            except Exception:
                pass
        return env_val

    # Fallback to Windows User/System Registry if not in process env
    if sys.platform == "win32":
        try:
            import winreg
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, r"Environment") as k:
                user_val, _ = winreg.QueryValueEx(k, "ELEVENLABS_API_KEY")
                if user_val:
                    return user_val
        except Exception:
            pass

def test_api_key(api_key: str):
    """Verifies that the provided API key is valid and lists active account voices."""
    url = "https://api.elevenlabs.io/v1/voices"
    req = urllib.request.Request(url, headers={"xi-api-key": api_key})
    try:
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            return True, data.get("voices", [])
    except urllib.error.HTTPError as e:
        return False, f"HTTP {e.code}: {e.reason}"
    except Exception as e:
        return False, str(e)
    """Verifies that the provided API key is valid and lists active account voices."""
    url = "https://api.elevenlabs.io/v1/voices"
    req = urllib.request.Request(url, headers={"xi-api-key": api_key})
    try:
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            return True, data.get("voices", [])
    except urllib.error.HTTPError as e:
        return False, f"HTTP {e.code}: {e.reason}"
    except Exception as e:
        return False, str(e)


def generate_speech(
    api_key: str,
    voice_id: str,
    text: str,
    output_path: Path,
    model_id: str = "eleven_turbo_v2_5",
    stability: float = 0.5,
    similarity_boost: float = 0.75,
    style: float = 0.1,
):
    """Sends text-to-speech request to ElevenLabs API and writes MP3 file."""
    url = f"https://api.elevenlabs.io/v1/text-to-speech/{voice_id}?output_format=mp3_44100_128"
    payload = {
        "text": text,
        "model_id": model_id,
        "voice_settings": {
            "stability": stability,
            "similarity_boost": similarity_boost,
            "style": style,
            "use_speaker_boost": True,
        },
    }
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=data,
        headers={
            "xi-api-key": api_key,
            "Content-Type": "application/json",
            "Accept": "audio/mpeg",
        },
    )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with urllib.request.urlopen(req) as response:
        with open(output_path, "wb") as f:
            f.write(response.read())


def main():
    parser = argparse.ArgumentParser(
        description="Generate character voice audio from SCREEN_MESSAGES.md via ElevenLabs"
    )
    parser.add_argument(
        "--api-key",
        default=os.environ.get("ELEVENLABS_API_KEY"),
        help="ElevenLabs API key (or set ELEVENLABS_API_KEY env var)",
    )
    parser.add_argument(
        "--character",
        choices=list(CHARACTER_CONFIG.keys()),
        help="Generate lines only for a specific character",
    )
    parser.add_argument(
        "--line",
        help="Generate only a specific line ID (e.g. 'match_start' or 'took_flag')",
    )
    parser.add_argument(
        "--model",
        default="eleven_turbo_v2_5",
        help="ElevenLabs TTS Model ID (e.g. eleven_turbo_v2_5, eleven_multilingual_v2)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print lines, character assignments, and target file paths without calling API",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Regenerate audio files even if they already exist",
    )
    parser.add_argument(
        "--list-voices",
        action="store_true",
        help="Test connection and list available voice IDs in your account",
    )

    args = parser.parse_args()
    api_key = get_api_key(args.api_key)

    # Voice listing check
    if args.list_voices:
        if not api_key:
            print("[ERROR] ELEVENLABS_API_KEY is not set. Provide --api-key or set env var.")
            sys.exit(1)
        ok, res = test_api_key(api_key)
        if ok:
            print(f"[OK] Connected! Found {len(res)} available voices:")
            for v in res:
                labels = v.get("labels") or {}
                gender = labels.get("gender", "unknown")
                accent = labels.get("accent", "")
                print(f"  • {v['name']} ({gender}, {accent}): {v['voice_id']}")
        else:
            print(f"[ERROR] Failed to query voices: {res}")
        return

    # Check API key if not in dry-run
    if not args.dry_run and not api_key:
        print("[ERROR] ElevenLabs API key is missing.")
        print("  Set the ELEVENLABS_API_KEY environment variable or pass --api-key <KEY>.")
        print("  Run with --dry-run to preview voice lines without an API key.")
        sys.exit(1)

    project_root = Path(__file__).resolve().parent.parent
    assets_dir = project_root / "assets" / "voices"

    characters_to_run = [args.character] if args.character else list(VOICE_LINES.keys())
    total_lines = 0
    generated_count = 0
    skipped_count = 0

    print("=" * 70)
    print("  Cyber-Athletic Coliseum - Voice Line Generator (ElevenLabs)")
    print(f"  Target directory: {assets_dir}")
    print(f"  Model ID:         {args.model}")
    print(f"  Mode:             {'DRY-RUN (Simulated)' if args.dry_run else 'LIVE API GENERATION'}")
    print("=" * 70)

    for char_id in characters_to_run:
        cfg = CHARACTER_CONFIG[char_id]
        lines = VOICE_LINES[char_id]
        print(f"\n[{cfg['display_name']} - {cfg['team']}] Voice ID: {cfg['voice_id']}")

        for line_id, text in lines:
            if args.line and args.line != line_id:
                continue

            total_lines += 1
            out_file = assets_dir / char_id / f"{line_id}.mp3"

            if out_file.exists() and not args.force and not args.dry_run:
                print(f"  [SKIPPED] {line_id} -> already exists: {out_file.name}")
                skipped_count += 1
                continue

            print(f"  • {line_id:18} -> {out_file.relative_to(project_root)}")
            print(f"    Text: \"{text}\"")

            if not args.dry_run:
                try:
                    generate_speech(
                        api_key=api_key,
                        voice_id=cfg["voice_id"],
                        text=text,
                        output_path=out_file,
                        model_id=args.model,
                        stability=cfg["stability"],
                        similarity_boost=cfg["similarity_boost"],
                        style=cfg["style"],
                    )
                    generated_count += 1
                    time.sleep(0.3)
                except urllib.error.HTTPError as e:
                    err_msg = e.read().decode("utf-8", errors="ignore")
                    print(f"    [FAIL] HTTP Error {e.code}: {e.reason} - {err_msg}")
                except Exception as e:
                    print(f"    [FAIL] Error: {e}")

    print("\n" + "=" * 70)
    if args.dry_run:
        print(f"Dry-run complete. Total lines cataloged: {total_lines}")
        print("To generate audio files, run without --dry-run using your valid API key.")
    else:
        print(f"Done! Generated: {generated_count}, Skipped: {skipped_count}, Total: {total_lines}")
    print("=" * 70)


if __name__ == "__main__":
    main()
