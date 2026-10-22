# On-Screen Messages & HUD Text Reference (`SCREEN_MESSAGES.md`)

This document catalogs all on-screen messages, HUD labels, dynamic Capture the Flag (CTF) event alerts, ability indicators, overhead character nametags, and control prompts in **Cyber-Athletic Coliseum**, including their original Norwegian text and their English translations.

---

## 1. Top Match Scoreboard & Objective Subtitles ([`src/ui.rs`](src/ui.rs))

| Screen Location | Trigger / State | Original Text (Norwegian) | English Translation (Active in Game) |
| :--- | :--- | :--- | :--- |
| **Top-Left Score Box** | Always visible (`koth.blue_score`) | `BLÅ FLAGG: {score}` | `BLUE FLAG: {score}` |
| **Top-Center Timer** | Match countdown (`MM:SS`) | `{mins:02}:{secs:02}` | `{mins:02}:{secs:02}` |
| **Top-Center Subtitle** | Initial startup placeholder | `SONE ALPHA` | `ZONE ALPHA` |
| **Top-Center Subtitle** | Default / neutral flag state | `CAPTURE THE FLAG (FØRSTE TIL 5)` | `CAPTURE THE FLAG (FIRST TO 5)` |
| **Top-Center Subtitle** | Blue Team is carrying the Red Flag | `LEVER RØDT FLAGG TIL BLÅ BASE!` | `DELIVER RED FLAG TO BLUE BASE!` |
| **Top-Center Subtitle** | Blue Flag is away from Blue Base | `TAKLE FIENDEN FOR Å REDDE BLÅTT FLAGG!` | `TACKLE THE ENEMY TO SAVE BLUE FLAG!` |
| **Top-Right Score Box** | Always visible (`koth.red_score`) | `RØD FLAGG: {score}` | `RED FLAG: {score}` |

---

## 2. Prominent Center-Top Dynamic Status & Alert Banner ([`src/koth.rs`](src/koth.rs) & [`src/ui.rs`](src/ui.rs))

### A. Persistent Objective Headlines (when no short event notice is active)

| State | Original Text (Norwegian) | English Translation (Active in Game) |
| :--- | :--- | :--- |
| **Initial Resource Default** | `«STJEL DET RØDE FLAGGET PÅ ØSTPLATTFORMEN!»` | `«STEAL THE RED FLAG ON THE EAST PLATFORM!»` |
| **Neutral (Both Flags at Base)** | `«EROBRE RØDT FLAGG PÅ ØSTPLATTFORMEN OG BRING DET TIL BLÅ BASE!»` | `«CAPTURE THE RED FLAG ON THE EAST PLATFORM AND BRING IT TO BLUE BASE!»` |
| **Blue Carrying Red Flag** | `«{NAME} HAR RØDT FLAGG! FRIGJØR VEIEN TIL BLÅ BASE!»` | `«{NAME} HAS THE RED FLAG! CLEAR THE WAY TO BLUE BASE!»` |
| **Red Carrying Blue Flag** | `«VÅRT FLAGG ER STJÅLET AV {NAME}! TAKLE DEM!»` | `«OUR FLAG WAS STOLEN BY {NAME}! TACKLE THEM!»` |
| **Both Flags Stolen (Standoff)** | `«BEGGE FLAGG STJÅLET! {BLUE_NAME} [BLÅ] vs {RED_NAME} [RØD]»` | `«BOTH FLAGS STOLEN! {BLUE_NAME} [BLUE] vs {RED_NAME} [RED]»` |

### B. Timed Gameplay Event Notices (`koth.event_notice`)

| Gameplay Event | Duration | Original Text (Norwegian) | English Translation (Active in Game) |
| :--- | :--- | :--- | :--- |
| **Red Player Steals Blue Flag** | `4.0s` | `«VÅRT FLAGG ER STJÅLET AV {NAME}! TAKLE DEM!»` | `«OUR FLAG WAS STOLEN BY {NAME}! TACKLE THEM!»` |
| **Blue Player Steals Red Flag** | `4.0s` | `«{NAME} TOK DET RØDE FLAGGET! LØP TIL BLÅ BASE!»` | `«{NAME} TOOK THE RED FLAG! RUN TO BLUE BASE!»` |
| **Carrier Drops Blue Flag (Tackled)** | `3.2s` | `«{NAME} MISTET DET BLÅ FLAGGET! TA DET TILBAKE!»` | `«{NAME} DROPPED THE BLUE FLAG! RECOVER IT!»` |
| **Carrier Drops Red Flag (Tackled)** | `3.2s` | `«{NAME} MISTET DET RØDE FLAGGET!»` | `«{NAME} DROPPED THE RED FLAG!»` |
| **Blue Teammate Touch-Returns Blue Flag** | `3.5s` | `«{NAME} RETURNERTE VÅRT BLÅ FLAGG TIL BASEN!»` | `«{NAME} RETURNED OUR BLUE FLAG TO BASE!»` |
| **Red Teammate Touch-Returns Red Flag** | `3.5s` | `«{NAME} RETURNERTE DET RØDE FLAGGET!»` | `«{NAME} RETURNED THE RED FLAG!»` |
| **Blue Flag Auto-Returns (15s Timer)** | `3.0s` | `«BLÅTT FLAGG RETURNERT TIL BASEN!»` | `«BLUE FLAG RETURNERT TO BASE!»` / `«BLUE FLAG RETURNED TO BASE!»` |
| **Red Flag Auto-Returns (15s Timer)** | `3.0s` | `«RØDT FLAGG RETURNERT TIL BASEN!»` | `«RED FLAG RETURNED TO BASE!»` |
| **Blue Team Captures Red Flag (+1 Pt)** | `4.5s` | `«FLAGG EROBRET AV {NAME}! +1 BLÅTT POENG!»` | `«FLAG CAPTURED BY {NAME}! +1 BLUE POINT!»` |
| **Red Team Captures Blue Flag (+1 Pt)** | `4.5s` | `«{NAME} EROBRET VÅRT FLAGG! +1 RØDT POENG!»` | `«{NAME} CAPTURED OUR FLAG! +1 RED POINT!»` |

### C. End-of-Match Victory / Defeat Banners

| Match Outcome | Original Text (Norwegian) | English Translation (Active in Game) |
| :--- | :--- | :--- |
| **Blue Team Wins (5 Flags or Timer Expiry)** | `«SEIER! BLÅTT LAG VANT CAPTURE THE FLAG!»` | `«VICTORY! BLUE TEAM WON CAPTURE THE FLAG!»` |
| **Red Team Wins (5 Flags or Timer Expiry)** | `«NEDERLAG! RØDT LAG VANT CAPTURE THE FLAG!»` | `«DEFEAT! RED TEAM WON CAPTURE THE FLAG!»` |

---

## 3. Bottom-Left Operator Card & Bottom-Right Tactical Ability HUD ([`src/ui.rs`](src/ui.rs))

| HUD Element | State / Condition | Original Text (Norwegian) | English Translation (Active in Game) |
| :--- | :--- | :--- | :--- |
| **Operator Name** | Always visible | `ANDERS` | `ANDERS` |
| **Health Readout** | Dynamic (`health.current`) | `HP: {hp}` | `HP: {hp}` |
| **Stamina Readout** | Dynamic (`stamina.current`) | `STAMINA: {stamina}` | `STAMINA: {stamina}` |
| **Tackle / Net Slot (Ready)** | `cooldown <= 0.0` and `stamina >= 25.0` | `Kastbart nett (x2)` | `Throwable Net (x2)` |
| **Tackle / Net Slot (Cooldown)** | `cooldown > 0.0` | `Kastbart nett ({cd:.1}s)` | `Throwable Net ({cd:.1}s)` |
| **Tackle / Net Slot (Low Stamina)** | `cooldown <= 0.0` and `stamina < 25.0` | `Lav stamina for nett` | `Low Stamina for Net` |
| **Sprint Slot** | Always visible | `Hastighetsboost` | `Speed Boost` |

---

## 4. Floating 3D Overhead Character Nametags ([`src/ui.rs`](src/ui.rs) & [`src/types.rs`](src/types.rs))

| Character | Team | Original Overhead Tag (Norwegian) | English Overhead Tag (Active in Game) |
| :--- | :--- | :--- | :--- |
| **Anders** (Main Player) | Blue | `ANDERS [BLÅ]` | `ANDERS [BLUE]` |
| **Morten** (Ally Bot) | Blue | `MORTEN [BLÅ]` | `MORTEN [BLUE]` |
| **Jonas** (Ally Bot) | Blue | `JONAS [BLÅ]` | `JONAS [BLUE]` |
| **Siri** (Enemy Bot) | Red | `SIRI [RØD]` | `SIRI [RED]` |
| **Per** (Enemy Bot) | Red | `PER [RØD]` | `PER [RED]` |
| **Lisa** (Enemy Bot) | Red | `LISA [RØD]` | `LISA [RED]` |

---

## 5. Arena Zone Names ([`src/arena.rs`](src/arena.rs))

| Zone Index | Original Zone Name (Norwegian) | English Zone Name (Active in Game) |
| :--- | :--- | :--- |
| **Zone 0** | `Sentralsonen (Ruined Courtyard)` | `Central Zone (Ruined Courtyard)` |
| **Zone 1** | `Blå Base (Vestplattform)` | `Blue Base (West Platform)` |
| **Zone 2** | `Rød Base (Østplattform)` | `Red Base (East Platform)` |

---

## 6. Bottom Controls Bar ([`index.html`](index.html))

| Element | English Text (Active in Game) |
| :--- | :--- |
| **Controls Bar** | `WASD Move • MOUSE / Q, E Steer Camera • TAB / V Camera View • SHIFT Sprint • LMB / F Tackle • M Music` |
