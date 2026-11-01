# Cyber-Athletic Coliseum: Game Setup Prompt

This prompt can be used to scaffold and initialize the foundation of the game from scratch.

***

```markdown
Create a 3D physics-based 3v3 King of the Hill (KotH) tactical brawler called "Cyber-Athletic Coliseum" built in Rust with the Bevy Engine (v0.15) and Avian3D (v0.2), compiled to WebAssembly (wasm32-unknown-unknown) and bundled with Trunk for browser play via WebGL2.

### 1. Game Concept & Rules
- **Setting & Vibe**: Comic/cel-shaded daytime tactical compound.
- **Teams (3v3)**:
  - Team Blå (Allies): Player ("Anders") + 2 AI bots ("Morten", "Jonas").
  - Team Rød (Opponents): 3 AI bots ("Siri", "Lisa", "Per").
- **Core Loop**:
  - Teams battle over 3 elevated flag pedestals / capture zones (West Base, Center Depot, East Base).
  - Every 45 seconds, the active zone rotates.
  - Capturing and uncontested holding of the active zone awards points.
  - If both teams are inside the zone simultaneously, scoring is paused ("FLAGGSONE OMKJEMPET!").
  - First team to reach 100 points wins.
- **Combat Mechanics**:
  - Non-lethal kinetic shoving / tackles (high physical impulse knockback).
  - Players/bots can be shoved off elevated capture platforms to disrupt capture progress.
  - Sprinting consumes stamina; tackling has a cooldown and brief recovery stun.

---

### 2. Project & Dependency Configuration

1. **Cargo.toml**:
   - `bevy = { version = "0.15.3", default-features = true, features = ["webgl2", "mp3", "jpeg"] }`
   - `avian3d = "0.2.1"`
   - `console_error_panic_hook = "0.1"`
   - Wasm-specific target dependencies: `getrandom` (v0.2 with `js` feature, v0.3 with `wasm_js`), `wasm-bindgen = "0.2"`.
   - Build profiles:
     - `[profile.dev]` with `opt-level = 1` and `[profile.dev.package."*"] opt-level = 3` (ensures physics & rendering remain smooth in debug).
     - `[profile.release]` with `opt-level = "s"` and `lto = "thin"`.

2. **Trunk.toml & index.html**:
   - Set Trunk to serve on port `8080` binding to `0.0.0.0`.
   - Configure `index.html` with full-window canvas `#bevy`, meta tags to disable mobile pinch-zoom and default touch behaviors.

---

### 3. Modular Architecture (Plugin-Based ECS)

Scaffold `src/` into modular plugins registered in `src/main.rs`:

1. **`src/types.rs`**:
   - `enum Team { Bla, Rod }`
   - `struct Character { team: Team, name: String, is_player: bool }`
   - `struct Stamina { current: f32, max: f32, regen_rate: f32 }`
   - `struct TackleAbility { cooldown: Timer, is_ready: bool, force: f32 }`
   - `struct StaggerState { timer: Timer }`
   - `struct HillZone { id: usize, name: String, center: Vec3, radius: f32 }`
   - `struct MatchState { team_bla_score: f32, team_rod_score: f32, active_hill: usize, rotation_timer: Timer, is_contested: bool, game_over: bool }`

2. **`src/arena.rs` (`ArenaPlugin`)**:
   - Build a tactical yard arena (compound floor with colliders, shipping containers as cover/ramps, perimeter fences/fallout boundaries).
   - Spawn the 3 hill pedestals with visual ground marker rings.
   - Configure daytime sunlight (`DirectionalLight` with shadows) and ambient light.

3. **`src/player.rs` (`PlayerPlugin`)**:
   - Avian3D `RigidBody::Dynamic`, `Collider::capsule`, `LockedAxes::ROTATION_LOCKED`.
   - WASD movement relative to camera yaw.
   - Left Shift for sprint boost (drains `Stamina`).
   - Spacebar for jump.
   - Ground check and friction tuning.

4. **`src/camera.rs` (`CameraPlugin`)**:
   - Dual-camera toggle with `V` or `Tab`:
     - Tactical Isometric view (fixed high-angle overview following player).
     - 3rd-person follow/action view with mouse look.
   - Mouse scroll wheel zoom in/out with smooth interpolation.

5. **`src/combat.rs` (`CombatPlugin`)**:
   - Tackle ability (`LMB` or `F` for player): casts a forward cone/sphere check or uses physics impulses (`ExternalImpulse`) to knock away opponents.
   - Apply brief stagger / velocity damping to tackled victims.

6. **`src/ai.rs` (`AiPlugin`)**:
   - Bot state machine for the 5 AI characters (2 Blå, 3 Rød):
     - Seek active hill zone.
     - Engage/tackle closest enemy within range if contested.
     - Hold ground and stay within zone bounds.

7. **`src/koth.rs` (`KothPlugin`)**:
   - Track entities inside the active `HillZone` radius.
   - Determine zone control:
     - Blå only -> accrue Blå points.
     - Rød only -> accrue Rød points.
     - Both -> mark `is_contested = true` and halt scoring.
   - Update 45-second timer to advance the active hill sequentially.

8. **`src/ui.rs` (`UiPlugin`)**:
   - Screen-space HUD:
     - Top match bar: Team Blå score vs Team Rød score, rotation countdown timer.
     - Center alert banner for "FLAGGSONE OMKJEMPET!" (Contested) or "NY FLAGGSONE AKTIVERT".
     - Bottom-left player vitals: stamina bar and tackle cooldown indicator.
   - 3D billboard / overhead nametags with team-tinted text above each character.

---

### 4. Implementation Guidelines
- Use primitive meshes (capsules for operators, boxes for containers, cylinders for pads) with PBR materials for an immediate, self-contained playable build.
- Ensure the physics simulation (`avian3d`) uses realistic Earth gravity (`Vec3::new(0.0, -9.81, 0.0)`).
- Ensure `AssetMetaCheck::Never` is configured in `AssetPlugin` to avoid HTTP 404s on `.meta` files in browser Wasm environments.
```
