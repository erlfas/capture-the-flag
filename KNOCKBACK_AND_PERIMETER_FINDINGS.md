# Punch Knockback, Vertical Launch & Fence Perimeter — Key Findings

This document summarizes the root causes identified for unrealistic punch knockback distances, characters launching high into the air, and characters getting pushed beyond the barbed-wire fence perimeter, along with the fixes implemented across [`src/combat.rs`](src/combat.rs), [`src/arena.rs`](src/arena.rs), [`src/player.rs`](src/player.rs), and [`src/ai.rs`](src/ai.rs).

---

## 1. Characters Pushed Way Too Far Away (vs. Losing Punching Power)

### Root Causes
1. **Excessive Base Knockback + Uncapped Momentum Stacking**:
   - `KNOCKBACK_FORCE` (`18.5 m/s`) combined with 45% of the tackler's lunge speed (`TACKLE_IMPULSE = 21.5 m/s`) produced initial knockback speeds up to **`28.2 m/s`** (more than 3× normal running speed).
2. **Zero Drag During Stagger (`1.05s`)**:
   - While `stagger.duration > 0.0`, both `handle_player_movement` and `update_bot_behaviors` returned early without applying any horizontal deceleration to `LinearVelocity`. Combined with `UPWARD_LIFT = 3.8 m/s` lifting characters into the air, a single punch could send characters flying **20–25 meters** across the arena.
3. **Why Over-Damping or `FixedPostUpdate` Position Sync Kills Punch Power**:
   - In Avian3D, `PhysicsSchedule` integrates `Position` during `FixedPostUpdate`, and `Position` is only synced to `Transform` later in `PostUpdate` (`PhysicsSet::Sync`). Running boundary/position sync in `FixedPostUpdate` (or overwriting `pos.0` unconditionally from `transform.translation` when no boundary clamp occurred) resets `Position` back to the stale pre-physics `Transform` every physics tick, canceling out knockback and lunge movement.
   - Similarly, setting `UPWARD_LIFT = 0.0` with no terrain step-gliding during stagger causes the knocked-back capsule to snag immediately on uneven 0.5m trimesh edges.

### Fixes Implemented
- Tuned `KNOCKBACK_FORCE` to **`17.2 m/s`** with a capped forward momentum bonus (`<= +3.2 m/s`), `TACKLE_IMPULSE` to **`19.5 m/s`**, `UPWARD_LIFT` to **`2.1 m/s`**, and stagger duration to **`0.86s`**.
- Added smooth horizontal drag (`lerp` decay at `1.75 * dt`) plus uphill terrain step-gliding during `stagger.duration > 0.0` in `update_cooldowns_and_stagger` so punched characters are knocked back a consistent, impactful **~6–8 meters** without snagging on terrain or flying across the map.
- Ensured `handle_fallout_respawns` only writes to `Position` when an actual perimeter clamp occurs, and only runs in `Update` and `PostUpdate.after(PhysicsSet::Sync)`.
- Added a procedural skeletal hand/arm punching animation (`apply_character_punch_pose` in [`src/player.rs`](src/player.rs), driven by `TackleAbility::punch_anim_timer`) that runs in `PostUpdate` after `bevy::animation::animate_targets` and before `TransformSystem::TransformPropagate`, driving `mixamorig:RightArm`/`LeftArm` and `mixamorig:RightForeArm`/`LeftForeArm` forward in a rapid two-handed punch thrust while leaving `mixamorig:Spine2` untouched (since `Spine2` has no animation channel in the GLB clips and would otherwise accumulate rotation across frames).

---

## 2. Characters Launched Far Up Into the Air

### Root Causes
1. **Physics Slope & Capsule Deflection ("Ski Jump Effect")**:
   - Moving a capsule at `20–28 m/s` with `UPWARD_LIFT = 3.8 m/s` into any angled triangle of the 3D terrain trimesh (ramps, stairs, rubble, sandbags, or trench walls) or into another capsule's hemisphere caused Avian3D's contact solver to deflect horizontal momentum into `LinearVelocity.y` (`+15 to +25 m/s`).
2. **Uncapped Slope-Climbing Vertical Velocity**:
   - Uphill step-climbing logic in `player.rs` and `ai.rs` allowed `climb_vy` up to `12.5–13.5 m/s` (more than 2× `JUMP_IMPULSE = 5.4 m/s`), launching characters skyward when cresting short steps or rubble piles.

### Fixes Implemented
- Reduced `UPWARD_LIFT` from `3.8 m/s` to **`2.1 m/s`** (just enough to unstick boots from trimesh friction without lofting into the air).
- Capped slope-climbing `climb_vy` to **`4.2 m/s`** in `player.rs` and `ai.rs`.
- Enforced vertical velocity ceilings (`lin_vel.y <= 2.4 m/s` while staggered, `<= 5.8 m/s` otherwise) and horizontal speed caps (`<= 19.8 m/s` while staggered) in `handle_fallout_respawns`.

---

## 3. Characters Pushed Beyond the Fence Perimeter

### Root Causes
1. **Perspective-Tapered 3D Fence Geometry vs. Axis-Aligned Bounds**:
   - Inspection of `assets/models/post_apocalyptic_battlefield.glb` revealed that the compound's West and East barbed-wire fences taper inward toward the North edge due to perspective foreshortening:
     - **West Fence**: runs from `x ≈ -22.3` (at `z = -15.0`) out to `x ≈ -24.7` (at `z = +11.0`).
     - **East Fence**: runs from `x ≈ +21.4` (at `z = -15.0`) out to `x ≈ +24.0` (at `z = +11.0`).
     - **North Fence**: inner face sits along `z ≈ -15.5` (with rubble around `z ≈ -14.6`).
   - Because the previous bounds were axis-aligned constants (`PLAYABLE_MIN_X = -24.2`, `PLAYABLE_MAX_X = 23.6`, `PLAYABLE_MIN_Z = -16.2`), the northern half of the West and East boundaries lay **1.5–2.2 meters outside** the actual 3D barbed-wire fences.

### Fixes Implemented
- Replaced fixed X bounds with Z-dependent boundary functions [`playable_min_x(z)`](src/arena.rs) and [`playable_max_x(z)`](src/arena.rs) that follow the exact inward taper of the West and East barbed-wire fences and platform railings.
- Tightened `PLAYABLE_MIN_Z` to **`-14.6`** and `PLAYABLE_MAX_Z` to **`16.0`**.
- Rotated and positioned the 30m-tall `RigidBody::Static` perimeter barrier colliders in `setup_arena` to match the angled West and East fence lines, and updated `check_tackle_hits`, `handle_body_pushing`, and `handle_fallout_respawns` to deflect and clamp against `playable_min_x(z)` and `playable_max_x(z)`.
