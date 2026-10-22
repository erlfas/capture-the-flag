# On-Screen Messages & Character Voice Script (`SCREEN_MESSAGES.md`)

This document catalogs all on-screen HUD messages, objective subtitles, and **character-differentiated voice callouts** in **Cyber-Athletic Coliseum**, including which character speaks each line, the exact spoken English transcript for voice generation, and the expected audio file path (`assets/voices/<character>/<line_id>.mp3`).

---

## 1. Character Roster & Voice Profiles

| Character ID | Display Name | Team | Role | Voice Folder Path |
| :--- | :--- | :--- | :--- | :--- |
| `anders` | **ANDERS** | `BLUE` | Main Player / Squad Leader | `assets/voices/anders/` |
| `morten` | **MORTEN** | `BLUE` | Vanguard / Flag Striker | `assets/voices/morten/` |
| `jonas` | **JONAS** | `BLUE` | Base Defender / Tactical Comms | `assets/voices/jonas/` |
| `siri` | **SIRI** | `RED` | Lead Striker / Flag Runner | `assets/voices/siri/` |
| `lisa` | **LISA** | `RED` | Flanker / Scout | `assets/voices/lisa/` |
| `per` | **PER** | `RED` | Heavy Defender / Enforcer | `assets/voices/per/` |

When any character speaks a line:
1. **Top Radio Banner (`ZoneStatusText`)** displays:
   `{SPEAKER} [{TEAM}]: «{Spoken English Line}»`
2. **3D Overhead Character Nametag (`CharacterNametag`)** above that character in the 3D arena highlights with:
   `[SPEAKING] {SPEAKER} [{TEAM}]`
3. **Voice Audio Path (`VoiceCallout::audio_asset_path`)** resolves to:
   `assets/voices/{character_id}/{line_id}.mp3`

---

## 2. Character-by-Character Voice Lines (For Voice Generation)

### A. Anders (`BLUE` — Main Player / Squad Leader)

| Line ID | Audio File Path | Trigger Event | Exact Spoken English Text (TTS Prompt) | On-Screen Banner Display |
| :--- | :--- | :--- | :--- | :--- |
| `match_start` | `assets/voices/anders/match_start.mp3` | Match start / neutral state | `"Capture the Red Flag on the East Platform and bring it to Blue Base!"` | `ANDERS [BLUE]: «Capture the Red Flag on the East Platform and bring it to Blue Base!»` |
| `took_flag` | `assets/voices/anders/took_flag.mp3` | Anders steals the Red Flag | `"I've got the Red Flag! Cover me back to Blue Base!"` | `ANDERS [BLUE]: «I've got the Red Flag! Cover me back to Blue Base!»` |
| `dropped_flag` | `assets/voices/anders/dropped_flag.mp3` | Anders is tackled & drops Red Flag | `"I dropped the Red Flag! Someone grab it!"` | `ANDERS [BLUE]: «I dropped the Red Flag! Someone grab it!»` |
| `returned_flag` | `assets/voices/anders/returned_flag.mp3` | Anders touch-returns Blue Flag | `"I returned our Blue Flag to base!"` | `ANDERS [BLUE]: «I returned our Blue Flag to base!»` |
| `captured_flag` | `assets/voices/anders/captured_flag.mp3` | Anders captures Red Flag at Blue Base | `"Flag captured! Plus one point for Blue Team!"` | `ANDERS [BLUE]: «Flag captured! Plus one point for Blue Team!»` |
| `tackle_hit` | `assets/voices/anders/tackle_hit.mp3` | Anders tackles an enemy | `"Target neutralized! Stay down!"` | `ANDERS [BLUE]: «Target neutralized! Stay down!»` |
| `flag_stolen_alert` | `assets/voices/anders/flag_stolen_alert.mp3` | Enemy is carrying Blue Flag | `"Our flag was stolen by {ENEMY}! Tackle them!"` | `ANDERS [BLUE]: «Our flag was stolen by {ENEMY}! Tackle them!»` |
| `escort_carrier` | `assets/voices/anders/escort_carrier.mp3` | Teammate (`Morten`/`Jonas`) has Red Flag | `"{ALLY} has the Red Flag! Clear the way to Blue Base!"` | `ANDERS [BLUE]: «{ALLY} has the Red Flag! Clear the way to Blue Base!»` |
| `victory` | `assets/voices/anders/victory.mp3` | Blue Team wins the match | `"Victory! Blue Team won Capture the Flag!"` | `ANDERS [BLUE]: «Victory! Blue Team won Capture the Flag!»` |

---

### B. Morten (`BLUE` — Vanguard / Flag Striker)

| Line ID | Audio File Path | Trigger Event | Exact Spoken English Text (TTS Prompt) | On-Screen Banner Display |
| :--- | :--- | :--- | :--- | :--- |
| `took_flag` | `assets/voices/morten/took_flag.mp3` | Morten steals the Red Flag | `"I took the Red Flag! Run to Blue Base!"` | `MORTEN [BLUE]: «I took the Red Flag! Run to Blue Base!»` |
| `dropped_flag` | `assets/voices/morten/dropped_flag.mp3` | Morten is tackled & drops Red Flag | `"I lost the Red Flag! Pick it up!"` | `MORTEN [BLUE]: «I lost the Red Flag! Pick it up!»` |
| `returned_flag` | `assets/voices/morten/returned_flag.mp3` | Morten touch-returns Blue Flag | `"Blue Flag returned to base! We're safe!"` | `MORTEN [BLUE]: «Blue Flag returned to base! We're safe!»` |
| `captured_flag` | `assets/voices/morten/captured_flag.mp3` | Morten captures Red Flag at Blue Base | `"Touchdown! I captured the Red Flag for Blue Team!"` | `MORTEN [BLUE]: «Touchdown! I captured the Red Flag for Blue Team!»` |
| `tackle_hit` | `assets/voices/morten/tackle_hit.mp3` | Morten tackles an enemy | `"Boom! Direct tackle hit!"` | `MORTEN [BLUE]: «Boom! Direct tackle hit!»` |
| `escort_carrier` | `assets/voices/morten/escort_carrier.mp3` | Anders is carrying Red Flag | `"Anders has the Red Flag! Clear the way to Blue Base!"` | `MORTEN [BLUE]: «Anders has the Red Flag! Clear the way to Blue Base!»` |

---

### C. Jonas (`BLUE` — Base Defender / Tactical Comms)

| Line ID | Audio File Path | Trigger Event | Exact Spoken English Text (TTS Prompt) | On-Screen Banner Display |
| :--- | :--- | :--- | :--- | :--- |
| `took_flag` | `assets/voices/jonas/took_flag.mp3` | Jonas steals the Red Flag | `"Red Flag secured! Heading back to Blue Base!"` | `JONAS [BLUE]: «Red Flag secured! Heading back to Blue Base!»` |
| `dropped_flag` | `assets/voices/jonas/dropped_flag.mp3` | Jonas is tackled & drops Red Flag | `"Dropped the Red Flag! Recover it fast!"` | `JONAS [BLUE]: «Dropped the Red Flag! Recover it fast!»` |
| `returned_flag` | `assets/voices/jonas/returned_flag.mp3` | Jonas touch-returns Blue Flag | `"I recovered our Blue Flag! Base is secure!"` | `JONAS [BLUE]: «I recovered our Blue Flag! Base is secure!»` |
| `flag_auto_returned` | `assets/voices/jonas/flag_auto_returned.mp3` | Dropped Blue Flag auto-returns (15s) | `"Blue Flag automatically returned to base!"` | `JONAS [BLUE]: «Blue Flag automatically returned to base!»` |
| `captured_flag` | `assets/voices/jonas/captured_flag.mp3` | Jonas captures Red Flag at Blue Base | `"Red Flag delivered! Point for Blue Team!"` | `JONAS [BLUE]: «Red Flag delivered! Point for Blue Team!»` |
| `tackle_hit` | `assets/voices/jonas/tackle_hit.mp3` | Jonas tackles an enemy | `"Enemy stopped in their tracks!"` | `JONAS [BLUE]: «Enemy stopped in their tracks!»` |
| `both_flags_stolen` | `assets/voices/jonas/both_flags_stolen.mp3` | Both flags are carried simultaneously | `"Both flags stolen! Protect {BLUE_CARRIER} and tackle {RED_CARRIER}!"` | `JONAS [BLUE]: «Both flags stolen! Protect {BLUE_CARRIER} and tackle {RED_CARRIER}!»` |

---

### D. Siri (`RED` — Lead Striker / Flag Runner)

| Line ID | Audio File Path | Trigger Event | Exact Spoken English Text (TTS Prompt) | On-Screen Banner Display |
| :--- | :--- | :--- | :--- | :--- |
| `took_flag` | `assets/voices/siri/took_flag.mp3` | Siri steals the Blue Flag | `"I stole the Blue Flag! Catch me if you can!"` | `SIRI [RED]: «I stole the Blue Flag! Catch me if you can!»` |
| `dropped_flag` | `assets/voices/siri/dropped_flag.mp3` | Siri is tackled & drops Blue Flag | `"Ugh, I dropped the Blue Flag! Recover it!"` | `SIRI [RED]: «Ugh, I dropped the Blue Flag! Recover it!»` |
| `returned_flag` | `assets/voices/siri/returned_flag.mp3` | Siri touch-returns Red Flag | `"I returned the Red Flag to our base!"` | `SIRI [RED]: «I returned the Red Flag to our base!»` |
| `captured_flag` | `assets/voices/siri/captured_flag.mp3` | Siri captures Blue Flag at Red Base | `"I captured the Blue Flag! Point for Red Team!"` | `SIRI [RED]: «I captured the Blue Flag! Point for Red Team!»` |
| `tackle_hit` | `assets/voices/siri/tackle_hit.mp3` | Siri tackles a Blue player | `"Too slow! Took you down!"` | `SIRI [RED]: «Too slow! Took you down!»` |
| `victory` | `assets/voices/siri/victory.mp3` | Red Team wins the match | `"Victory for Red Team! We own this arena!"` | `SIRI [RED]: «Victory for Red Team! We own this arena!»` |

---

### E. Lisa (`RED` — Flanker / Scout)

| Line ID | Audio File Path | Trigger Event | Exact Spoken English Text (TTS Prompt) | On-Screen Banner Display |
| :--- | :--- | :--- | :--- | :--- |
| `took_flag` | `assets/voices/lisa/took_flag.mp3` | Lisa steals the Blue Flag | `"Blue Flag is mine! Sprinting to Red Base!"` | `LISA [RED]: «Blue Flag is mine! Sprinting to Red Base!»` |
| `dropped_flag` | `assets/voices/lisa/dropped_flag.mp3` | Lisa is tackled & drops Blue Flag | `"I lost the Blue Flag! Don't let them touch it!"` | `LISA [RED]: «I lost the Blue Flag! Don't let them touch it!»` |
| `returned_flag` | `assets/voices/lisa/returned_flag.mp3` | Lisa touch-returns Red Flag | `"Red Flag is back at base!"` | `LISA [RED]: «Red Flag is back at base!»` |
| `captured_flag` | `assets/voices/lisa/captured_flag.mp3` | Lisa captures Blue Flag at Red Base | `"Score! Another flag capture for Red Team!"` | `LISA [RED]: «Score! Another flag capture for Red Team!»` |
| `tackle_hit` | `assets/voices/lisa/tackle_hit.mp3` | Lisa tackles a Blue player | `"Caught you in the net!"` | `LISA [RED]: «Caught you in the net!»` |

---

### F. Per (`RED` — Heavy Defender / Enforcer)

| Line ID | Audio File Path | Trigger Event | Exact Spoken English Text (TTS Prompt) | On-Screen Banner Display |
| :--- | :--- | :--- | :--- | :--- |
| `took_flag` | `assets/voices/per/took_flag.mp3` | Per steals the Blue Flag | `"I have the Blue Flag! Clear the way to Red Base!"` | `PER [RED]: «I have the Blue Flag! Clear the way to Red Base!»` |
| `dropped_flag` | `assets/voices/per/dropped_flag.mp3` | Per is tackled & drops Blue Flag | `"Dropped the Blue Flag! Hold them back!"` | `PER [RED]: «Dropped the Blue Flag! Hold them back!»` |
| `returned_flag` | `assets/voices/per/returned_flag.mp3` | Per touch-returns Red Flag | `"I secured the Red Flag! Back at base!"` | `PER [RED]: «I secured the Red Flag! Back at base!»` |
| `flag_auto_returned` | `assets/voices/per/flag_auto_returned.mp3` | Dropped Red Flag auto-returns (15s) | `"Red Flag automatically returned to base!"` | `PER [RED]: «Red Flag automatically returned to base!»` |
| `captured_flag` | `assets/voices/per/captured_flag.mp3` | Per captures Blue Flag at Red Base | `"Blue Flag delivered to Red Base! Point scored!"` | `PER [RED]: «Blue Flag delivered to Red Base! Point scored!»` |
| `tackle_hit` | `assets/voices/per/tackle_hit.mp3` | Per tackles a Blue player | `"Sit down! Nobody gets past me!"` | `PER [RED]: «Sit down! Nobody gets past me!»` |

---

## 3. Non-Spoken HUD Readouts & Labels ([`src/ui.rs`](src/ui.rs), [`index.html`](index.html))

| Screen Location | Original Text (Norwegian) | English Text (Active in Game) |
| :--- | :--- | :--- |
| **Top-Left Score Box** | `BLÅ FLAGG: {score}` | `BLUE FLAG: {score}` |
| **Top-Center Timer** | `{mins:02}:{secs:02}` | `{mins:02}:{secs:02}` |
| **Top-Center Subtitle (Neutral)** | `CAPTURE THE FLAG (FØRSTE TIL 5)` | `CAPTURE THE FLAG (FIRST TO 5)` |
| **Top-Center Subtitle (Delivering)** | `LEVER RØDT FLAGG TIL BLÅ BASE!` | `DELIVER RED FLAG TO BLUE BASE!` |
| **Top-Center Subtitle (Defending)** | `TAKLE FIENDEN FOR Å REDDE BLÅTT FLAGG!` | `TACKLE THE ENEMY TO SAVE BLUE FLAG!` |
| **Top-Right Score Box** | `RØD FLAGG: {score}` | `RED FLAG: {score}` |
| **Bottom-Left Operator Card** | `ANDERS` / `HP: {hp}` / `STAMINA: {stamina}` | `ANDERS` / `HP: {hp}` / `STAMINA: {stamina}` |
| **Bottom-Right Net Slot (Ready)** | `Kastbart nett (x2)` | `Throwable Net (x2)` |
| **Bottom-Right Net Slot (Cooldown)** | `Kastbart nett ({cd:.1}s)` | `Throwable Net ({cd:.1}s)` |
| **Bottom-Right Net Slot (Low Stamina)** | `Lav stamina for nett` | `Low Stamina for Net` |
| **Bottom-Right Sprint Slot** | `Hastighetsboost` | `Speed Boost` |
| **3D Overhead Nametag (Default)** | `{NAME} [BLÅ]` / `{NAME} [RØD]` | `{NAME} [BLUE]` / `{NAME} [RED]` |
| **3D Overhead Nametag (Active Speaker)** | *(New)* | `[SPEAKING] {NAME} [BLUE/RED]` |
| **Bottom Controls Bar** | *(English)* | `WASD Move • MOUSE / Q, E Steer Camera • TAB / V Camera View • SHIFT Sprint • LMB / F Tackle • M Music` |
