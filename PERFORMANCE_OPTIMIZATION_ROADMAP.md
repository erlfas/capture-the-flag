# Cyber-Athletic Coliseum — Performance Optimization Roadmap
**Engine & Target**: Bevy `0.15.3` • Avian3D `0.2.1` • WebGL2 / WebAssembly (WASM)  
**Last Updated**: November 1, 2026

---

## 1. Status Overview

| Status | Item # | Optimization Task | Target Files | Category | Impact |
|:---:|:---:|:---|:---|:---:|:---|
| **✅ FIXED** | #1 | **Remove `SweptCcd` on all 6 characters** | `src/player.rs`, `src/ai.rs` | Physics | Eliminates continuous TOI checks against 20,160-triangle trimesh |
| **✅ FIXED** | #2 | **Cache skeletal bone entities & remove per-frame heap allocs** | `src/player.rs` | CPU / Anim | Eliminates ~600 string matches & 48–84 heap allocs per frame |
| **✅ FIXED** | #3 | **Guard UI `Text` & nametag mutations to stop Taffy layout thrashing** | `src/ui.rs` | UI / Layout | Stops full-tree flexbox recalculation & font re-rasterization |
| **✅ FIXED** | #4 | **Deduplicate `handle_fallout_respawns` from `Update`** | `src/arena.rs` | Systems | Halves terrain sampling & box collision clamping passes |
| **⏳ NOT FIXED** | #5 | **Replace downward ground raycast with `sample_terrain_height`** | `src/player.rs` | Physics / CPU | Eliminates 60 raycasts/sec into 20k-triangle trimesh |
| **⏳ NOT FIXED** | #6 | **Add `CollisionLayers` & optimize camera spring-arm raycast** | `src/camera.rs`, `src/types.rs` | Physics / Raycasts | Stops camera ray from traversing 20k terrain BVH; eliminates heap alloc |
| **⏳ NOT FIXED** | #7 | **Tune Avian physics solver substeps (`SubstepCount(2)`)** | `src/main.rs` | Physics | Halves contact solver CPU time on single-threaded WASM |
| **⏳ NOT FIXED** | #8 | **Fix sky dome backface culling & consolidate cloud layers** | `src/arena.rs` | GPU / Fillrate | Cuts transparent sky passes from 8 to 2 fullscreen passes |
| **⏳ NOT FIXED** | #9 | **Consolidate sewer water effects (135 entities → 2–3 meshes)** | `src/arena.rs` | GPU / Draw Calls | Eliminates ~125 unbatched draw calls & 129 transform updates |
| **⏳ NOT FIXED** | #10 | **Downscale 4K bot character textures to 1024×1024** | `assets/models/` | GPU / VRAM | Saves ~450 MB uncompressed VRAM & ~40 MB download payload |
| **⏳ NOT FIXED** | #11 | **Decimate `post_apocalyptic_battlefield.glb` (736k → 120k)** | `assets/models/` | GPU / Geometry | 80% reduction in vertex shading; re-enables backface culling |
| **⏳ NOT FIXED** | #12 | **Decimate bot meshes (38k → 10k vertices) to match player** | `assets/models/` | GPU / Skinning | Cuts skinned vertex count from ~200k to ~65k per frame |
| **⏳ NOT FIXED** | #13 | **Optimize `Cargo.toml` release profile (`panic = "abort"`, LTO)** | `Cargo.toml` | WASM / Build | Shrinks WASM binary by 30%–50% and removes landing pads |
| **⏳ NOT FIXED** | #14 | **Trim unused Bevy default features (`default-features = false`)** | `Cargo.toml` | WASM / Binary | Cuts ~25–35 MB of unused crates (gizmos, gilrs, tonemapping LUTs) |
| **⏳ NOT FIXED** | #15 | **Lazy-load character voice audio instead of 45-file preload** | `index.html` | Network / Audio | Eliminates startup network stampede & 45 browser audio decoders |

---

## 2. Details of Already-Fixed Items (Active in Current Build)

### ✅ FIXED: Item #1 — Removed `SweptCcd` on All 6 Characters
* **Files Modified**: [`src/player.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/player.rs#L173-L178), [`src/ai.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/ai.rs#L125-L130)
* **What Was Changed**:
  Removed `.insert(SweptCcd::default())` from both `spawn_player` and `spawn_single_bot`.
* **Result**:
  Eliminated iterative root-finding Time-of-Impact (TOI) checks against the 20,160-triangle terrain trimesh 360 times per second. Standard discrete capsule collision is handled in $O(1)$ time, and boundary tunneling is already prevented by 20m–30m perimeter boundary cuboids and hard boundary clamping in `handle_fallout_respawns`.

### ✅ FIXED: Item #2 — Cached Skeletal Bones & Stack-Allocated Parent Rotation
* **File Modified**: [`src/player.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/player.rs#L37-L660)
* **What Was Changed**:
  1. Added a `CharacterBones` component. Bone entities (`mixamorig:RightArm`, etc.) are discovered once upon scene spawn and cached directly on the character entity.
  2. Eliminated `children_query.iter_descendants` and ~600 string matching lookups (`name.as_str()`) on every frame in `apply_character_punch_pose`.
  3. Replaced `let mut chain = Vec::with_capacity(8);` in `model_space_parent_rot` with a stack-allocated fixed array `[Entity::PLACEHOLDER; 16]`, completely removing 48–84 heap allocations per frame during procedural bone FK calculation.
* **Result**:
  Eliminates recurring per-frame GC pressure and single-threaded WASM allocator lock contention.

### ✅ FIXED: Item #3 — Guarded UI `Text`, Nametag & Node Mutations Against Taffy Thrashing
* **File Modified**: [`src/ui.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/ui.rs#L1094-L1770)
* **What Was Changed**:
  1. Added equality checks (`if **text != new_text`) across all HUD text elements (`blue_text`, `red_text`, `timer_text`, `name_text`, `status_text`, `tackle_text`) and pause overlay text elements before invoking `DerefMut`.
  2. Guarded overhead nametag projections so `Node.left` and `Node.top` only mutate when moving by more than a 0.5px viewport threshold.
  3. Guarded nametag HP and Stamina bar `Node.width` (`Val::Percent(pct)`) and all menu `Node.display` / fill bars to only mutate on actual value changes.
* **Result**:
  Prevents Bevy 0.15's Taffy Flexbox engine from invalidating the entire layout tree, re-measuring text bounds, and re-rasterizing font glyphs on every frame.

### ✅ FIXED: Item #4 — Deduplicated `handle_fallout_respawns` Registration
* **File Modified**: [`src/arena.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/arena.rs#L50-L63)
* **What Was Changed**:
  Removed duplicate `handle_fallout_respawns` from `Update`, keeping it exclusively in `PostUpdate.after(PhysicsSet::Sync)`.
* **Result**:
  Halves per-frame boundary trigonometry, box container ejection calculations, and bilinear `sample_terrain_height` terrain lookups across all 6 characters.

---

## 3. Category A: UI & ECS Systems (Pending Fixes)

### ⏳ NOT FIXED: Guard Material Mutation in `update_hologram_visuals`
* **Target File**: [`src/koth.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/koth.rs#L980-L1005)
* **Status**: **NOT FIXED**
* **Root Cause**:
  Calls `materials.get_mut(mat_handle)` unconditionally every frame to adjust emissive and base color, continuously forcing Bevy's renderer to re-upload uniform buffer objects (UBOs) to the GPU.
* **Fix**:
  Track delivery state changes with a local boolean or change detection, and only mutate the material on state transitions.

### ⏳ NOT FIXED: Time-Slice AI Decision Logic
* **Target File**: [`src/ai.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/ai.rs#L185-L498)
* **Status**: **NOT FIXED**
* **Root Cause**:
  All 5 bots evaluate all behaviors (target search, priority math, trigonometry, uphill slope lookahead) every frame (60–120 Hz) on a single thread. Also allocates a heap vector every frame (`let char_snapshots: Vec<...> = ...collect()`).
* **Fix**:
  - Run the tactical decision tree (target selection, ramp selection) on a 10 Hz fixed timer (`0.1s`).
  - Keep frame-rate updates strictly for smooth velocity steering and terrain following.
  - Replace dynamic `Vec` heap allocation with a fixed stack array `[(Entity, Vec3, Vec3, Team); 6]`.

---

## 4. Category B: Physics, Colliders & Spatial Queries (Pending Fixes)

### ⏳ NOT FIXED: Item #5 — Remove Redundant Downward Ground Raycast
* **Target File**: [`src/player.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/player.rs#L253-L258)
* **Status**: **NOT FIXED**
* **Root Cause**:
  `player.rs` fires a downward raycast into the 20,160-triangle terrain mesh every frame to check `is_grounded`, even though the exact bilinear height is computed on the preceding line:
  ```rust
  let terrain_y = crate::terrain_heights::sample_terrain_height(transform.translation.x, transform.translation.z);
  let is_grounded = spatial_query.cast_ray(transform.translation, Dir3::NEG_Y, 1.07, true, &filter).is_some()
      || transform.translation.y <= terrain_y + 1.02;
  ```
* **Fix**:
  Remove `spatial_query.cast_ray` completely and rely directly on `transform.translation.y <= terrain_y + 1.05`, eliminating 60 expensive trimesh BVH ray traversals per second.

### ⏳ NOT FIXED: Item #6 — Add `CollisionLayers` & Fix Camera Spring-Arm Raycast
* **Target Files**: [`src/camera.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/camera.rs#L322-L337), [`src/types.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/types.rs)
* **Status**: **NOT FIXED**
* **Root Cause**:
  1. The spring-arm raycast in `camera.rs` allocates a new heap `Vec<Entity>` every frame (`character_entities.iter().collect()`).
  2. Because there are no collision layers, the raycast tests against the 20,160-triangle terrain mesh rather than just interior solid obstacles (shipping containers and walls).
* **Fix**:
  Define explicit `PhysicsLayer` groups (`Terrain`, `Obstacle`, `Character`):
  ```rust
  #[derive(PhysicsLayer, Default)]
  pub enum GameLayer {
      #[default]
      Default,
      Terrain,
      Obstacle,
      Character,
  }
  ```
  Change the camera raycast filter to `SpatialQueryFilter::from_mask([GameLayer::Obstacle])`. This eliminates the per-frame `Vec` heap allocation and skips the 20k terrain trimesh BVH.

### ⏳ NOT FIXED: Item #7 — Tune Avian Solver Substeps (`SubstepCount(2)`)
* **Target File**: [`src/main.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/main.rs#L40-L43)
* **Status**: **NOT FIXED**
* **Root Cause**:
  Avian defaults to 6 substeps per fixed step (360 solver ticks/sec at 60 Hz). For an arcade capture-the-flag game with simple capsules and static cuboids/trimesh, 6 substeps is unnecessary.
* **Fix**:
  Insert `SubstepCount(2)` in `main.rs`. Reduces contact solver and narrowphase CPU load by ~50%–65%.

---

## 5. Category C: Rendering, Shaders & 3D Assets (Pending Fixes)

### ⏳ NOT FIXED: Item #8 — Fix Sky Dome Backface Culling & Consolidate Cloud Layers
* **Target File**: [`src/arena.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/arena.rs#L468-L563)
* **Status**: **NOT FIXED**
* **Root Cause**:
  4 scrolling cloud layers use `cull_mode: None` and `AlphaMode::Blend`. The GPU renders both front and back faces of all 4 domes, producing **8 full-screen transparency passes** across the entire sky viewport. In WebGL2, transparent passes cannot use early-Z rejection and force continuous framebuffer read-modify-writes.
* **Fix**:
  1. Change `cull_mode: None` to `cull_mode: Some(bevy::render::render_resource::Face::Front)` so the exterior faces outside the dome are culled.
  2. Consolidate from 4 scrolling cloud layers down to **2 layers** (Layer A and Layer B). Cuts overdraw passes from 8 to 2 (75% fillrate reduction).

### ⏳ NOT FIXED: Item #9 — Consolidate Sewer Water Draw Calls (135 Entities → 2–3 Meshes)
* **Target File**: [`src/arena.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/arena.rs#L718-L1075)
* **Status**: **NOT FIXED**
* **Root Cause**:
  `spawn_sewer_water_effects` spawns 135 separate 3D entities (48 stream parcels, 28 splash droplets, 32 arch capsules, 6 ripples, 14 waves). Each uses `AlphaMode::Blend`, which Bevy sorts back-to-front and **cannot batch**. This creates 135 individual WebGL2 draw calls, and `animate_sewer_water` updates 129 `Transform` components on the CPU every frame.
* **Fix**:
  Replace individual particle entities with a scrolling UV waterfall strip and a rotating ripple decal/quad, reducing draw calls from 135 to 2–3 and eliminating 129 per-frame CPU transform updates.

### ⏳ NOT FIXED: Item #10 — Downscale Bot Character Textures (4096×4096 → 1024×1024)
* **Target Assets**: `assets/models/jonas-blue.glb`, `lisa-red.glb`, `morten-blue.glb`, `per-red.glb`, `siri-red.glb`
* **Status**: **NOT FIXED**
* **Root Cause**:
  Each bot contains 4096×4096 basecolor textures (67.1 MB uncompressed VRAM each = 335.5 MB total) plus 2048×2048 normal and roughness maps. Total texture VRAM exceeds **565 MB**, causing severe GPU memory pressure and cache thrashing on characters only 200–300 pixels high.
* **Fix**:
  Downscale bot basecolor, roughness, and normal maps to 1024×1024 (or 512×512 for normal maps). Shrinks VRAM from 565 MB to ~110 MB and reduces each bot file from ~11 MB to ~2 MB.

### ⏳ NOT FIXED: Item #11 — Decimate Battlefield Mesh (`post_apocalyptic_battlefield.glb`)
* **Target Asset**: [`assets/models/post_apocalyptic_battlefield.glb`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/assets/models/post_apocalyptic_battlefield.glb)
* **Status**: **NOT FIXED**
* **Root Cause**:
  Contains **735,845 triangles** and **456,667 vertices** with `"doubleSided": true`. WebGL2 vertex shaders are overwhelmed, and disabled backface culling forces all interior/underground faces to rasterize.
* **Fix**:
  1. Decimate geometry using Quadric Error Metric (QEM) simplification down to **120k–150k triangles**.
  2. Split the thin barbed-wire fences into a separate sub-mesh with `doubleSided: true`, allowing 95% of the terrain and buildings to use standard single-sided backface culling (`doubleSided: false`).

### ⏳ NOT FIXED: Item #12 — Decimate Bot Character Meshes (38k → 10k Vertices)
* **Target Assets**: `assets/models/*.glb`
* **Status**: **NOT FIXED**
* **Root Cause**:
  Each bot model has ~38,000 vertices with 65 skeletal bones (`JOINTS_0`, `WEIGHTS_0`). In WebGL2, evaluating skinning matrices for ~200,000 vertices every frame consumes substantial vertex shading time.
* **Fix**:
  Simplify bot meshes to match `main_player.glb` (~11k vertices, ~9k triangles) and merge duplicate vertices along UV seams.

---

## 6. Category D: WASM Compilation, Browser Environment & Cargo Profiles (Pending Fixes)

### ⏳ NOT FIXED: Item #13 — Optimize `Cargo.toml` Release Profile
* **Target File**: [`Cargo.toml`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/Cargo.toml#L23-L25)
* **Status**: **NOT FIXED**
* **Root Cause**:
  Release profile uses `lto = "thin"` without `panic = "abort"`, `codegen-units = 1`, or `strip = "symbols"`. Leaves landing pads, unwind tables, and symbol tables in the compiled WASM binary.
* **Fix**:
  ```toml
  [profile.release]
  opt-level = "s"
  lto = "fat"
  codegen-units = 1
  panic = "abort"
  strip = "symbols"
  ```

### ⏳ NOT FIXED: Item #14 — Strip Unused Bevy Default Features
* **Target File**: [`Cargo.toml`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/Cargo.toml#L7-L8)
* **Status**: **NOT FIXED**
* **Root Cause**:
  `default-features = true` compiles unused crates: `multi_threaded`, `tonemapping_luts` (with `ktx2` & `zstd`), `smaa_luts`, `bevy_gilrs`, `bevy_gizmos`, `bevy_sprite`, `bevy_mesh_picking_backend`, `default_font`, `vorbis`, and `hdr`.
* **Fix**:
  Set `default-features = false` and explicitly specify only the required Bevy features (`bevy_core_pipeline`, `bevy_render`, `bevy_pbr`, `bevy_gltf`, `bevy_ui`, `bevy_text`, `bevy_animation`, `bevy_audio`, `webgl2`, `mp3`, `jpeg`, `png`). Reduces binary size by ~25–35 MB.

### ⏳ NOT FIXED: Item #15 — Lazy-Load Character Voices in `index.html`
* **Target File**: [`index.html`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/index.html#L322-L335)
* **Status**: **NOT FIXED**
* **Root Cause**:
  `preloadGameAudio()` creates **45 `Audio` elements** with `preload = "auto"` synchronously during script initialization, saturating browser network connections and allocating 45 media decoders while loading the WASM binary.
* **Fix**:
  Preload only `punch.mp3`; load character voice clips on demand via memoized lazy loading.

---

## 7. Master Checklist

- [x] **Item #1**: Remove `SweptCcd` on all 6 characters in `src/player.rs` and `src/ai.rs`. *(✅ FIXED)*
- [x] **Item #2**: Cache skeletal bones (`CharacterBones`) and eliminate per-frame heap allocations in `src/player.rs`. *(✅ FIXED)*
- [x] **Item #3**: Guard UI `Text` and nametag mutations against Taffy flexbox layout thrashing in `src/ui.rs`. *(✅ FIXED)*
- [x] **Item #4**: Remove duplicate `handle_fallout_respawns` registration from `src/arena.rs`. *(✅ FIXED)*
- [ ] **Item #5**: Remove redundant downward ground raycast in `src/player.rs`. *(⏳ NOT FIXED)*
- [ ] **Item #6**: Add `CollisionLayers` and optimize camera spring-arm raycast in `src/camera.rs` & `src/types.rs`. *(⏳ NOT FIXED)*
- [ ] **Item #7**: Set `SubstepCount(2)` in `src/main.rs`. *(⏳ NOT FIXED)*
- [ ] **Item #8**: Enable backface culling on sky domes and reduce cloud layers in `src/arena.rs`. *(⏳ NOT FIXED)*
- [ ] **Item #9**: Consolidate sewer water effects (135 entities → 2–3 meshes) in `src/arena.rs`. *(⏳ NOT FIXED)*
- [ ] **Item #10**: Downscale bot textures from 4K to 1024×1024 in `assets/models/`. *(⏳ NOT FIXED)*
- [ ] **Item #11**: Decimate `post_apocalyptic_battlefield.glb` from 736k to 120k triangles. *(⏳ NOT FIXED)*
- [ ] **Item #12**: Decimate bot character meshes from 38k to 10k vertices. *(⏳ NOT FIXED)*
- [ ] **Item #13**: Optimize `Cargo.toml` release profile (`panic = "abort"`, LTO). *(⏳ NOT FIXED)*
- [ ] **Item #14**: Strip unused Bevy default features in `Cargo.toml`. *(⏳ NOT FIXED)*
- [ ] **Item #15**: Implement on-demand voice loading in `index.html`. *(⏳ NOT FIXED)*
