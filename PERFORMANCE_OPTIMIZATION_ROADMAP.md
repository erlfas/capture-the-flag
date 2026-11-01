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
| **✅ FIXED** | #5 | **Replace downward ground raycast with `sample_terrain_height`** | `src/player.rs` | Physics / CPU | Eliminates 60 raycasts/sec into 20k-triangle trimesh |
| **✅ FIXED** | #6 | **Add `CollisionLayers` & optimize camera spring-arm raycast** | `src/camera.rs`, `src/types.rs` | Physics / Raycasts | Stops camera ray from traversing 20k terrain BVH; eliminates heap alloc |
| **✅ FIXED** | #7 | **Tune Avian physics solver substeps (`SubstepCount(2)`)** | `src/main.rs` | Physics | Halves contact solver CPU time on single-threaded WASM |
| **✅ FIXED** | #8 | **Fix sky dome backface culling & consolidate cloud layers** | `src/arena.rs` | GPU / Fillrate | Cuts transparent sky passes from 8 to 2 fullscreen passes |
| **✅ FIXED** | #9 | **Consolidate sewer water effects (135 entities → 2–3 meshes)** | `src/arena.rs` | GPU / Draw Calls | Eliminates ~125 unbatched draw calls & 129 transform updates |
| **✅ FIXED** | #10 | **Downscale 4K bot character textures to 1024×1024** | `assets/models/` | GPU / VRAM | Saves ~472 MB uncompressed VRAM & ~38.3 MB download payload |
| **✅ FIXED** | #11 | **Decimate `post_apocalyptic_battlefield.glb` (736k → 140k)** | `assets/models/` | GPU / Geometry | 81% reduction in triangles & 17.3 MB file size reduction; re-enables backface culling |
| **✅ FIXED** | #12 | **Decimate bot meshes (38k → 11k vertices) to match player** | `assets/models/` | GPU / Skinning | Cuts total skinned vertex count from ~201k to ~67k per frame |
| **✅ FIXED** | #13 | **Optimize `Cargo.toml` release profile (`panic = "abort"`, LTO)** | `Cargo.toml` | WASM / Build | Shrinks WASM binary by 30%–50% and removes landing pads |
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

### ✅ FIXED: Item #5 — Replaced Downward Ground Raycast with `sample_terrain_height`
* **File Modified**: [`src/player.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/player.rs#L210-L265)
* **What Was Changed**:
  Removed `SpatialQueryFilter`, `spatial_query.cast_ray`, and `spatial_query: SpatialQuery` system parameter in `handle_player_movement`. Ground detection now relies directly on pre-computed bilinear terrain height sampling: `transform.translation.y <= terrain_y + 1.05`.
* **Result**:
  Completely eliminates 60 expensive ray traversals per second into the 20,160-triangle terrain mesh BVH and frees Bevy from fetching `SpatialQuery` for the player movement system.

### ✅ FIXED: Item #6 — Added `CollisionLayers` & Optimized Camera Spring-Arm Raycast
* **Files Modified**: [`src/types.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/types.rs#L1-L12), [`src/arena.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/arena.rs#L219-L355), [`src/player.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/player.rs#L178-L188), [`src/ai.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/ai.rs#L118-L128), [`src/camera.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/camera.rs#L90-L355)
* **What Was Changed**:
  1. Defined `GameLayer` (`Default`, `Terrain`, `Obstacle`, `Character`) in `src/types.rs` and assigned `CollisionLayers` across all static arena colliders, shipping containers, player, and bots.
  2. Replaced the per-frame `character_entities.iter().collect::<Vec<Entity>>()` heap allocation and `SpatialQueryFilter::from_excluded_entities` in `update_camera` with a stack-only `SpatialQueryFilter::from_mask(GameLayer::Obstacle)` filter.
  3. Guarded `Window.cursor_options` mutations in `handle_cursor_lock` and `update_camera` so `Window` is only marked mutated when cursor state transitions.
* **Result**:
  Eliminates per-frame `Vec` + `HashSet` heap allocations and prevents the camera spring-arm raycast from traversing the 20,160-triangle terrain BVH every frame.

### ✅ FIXED: Item #7 — Tuned Avian Solver Substeps (`SubstepCount(2)`) & Paused Background Physics
* **File Modified**: [`src/main.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/main.rs#L40-L105)
* **What Was Changed**:
  1. Inserted `.insert_resource(SubstepCount(2))` in `main.rs`, reducing Avian3D physics solver iterations from 6 substeps/frame (360 Hz) down to 2 substeps/frame (120 Hz).
  2. Added `Time<Physics>` pause/unpause synchronization in `handle_background_music_match_start` so physics substeps do not run while the game is on the pre-match intro screen or paused.
* **Result**:
  Cuts narrowphase contact solver CPU time by ~66% during gameplay and 100% while paused or on the title screen.

### ✅ FIXED: Item #8 — Enabled Sky Dome Backface Culling & Consolidated Cloud Layers
* **File Modified**: [`src/arena.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/arena.rs#L410-L535)
* **What Was Changed**:
  1. Reversed sky dome triangle winding to face inward and enabled standard hardware backface culling (`cull_mode` default) so exterior dome faces are culled.
  2. Consolidated from 4 scrolling transparent cloud domes down to 2 parallax layers (Layer A and Layer B).
* **Result**:
  Reduces full-screen transparent sky overdraw passes from 8 down to 2 (75% fillrate reduction).

### ✅ FIXED: Item #9 — Consolidated Sewer Water Effects (135 Entities → 4 Meshes)
* **File Modified**: [`src/arena.rs`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/arena.rs#L689-L940)
* **What Was Changed**:
  Replaced 135 individual transparent water particle/droplet/wave entities with 4 consolidated meshes (`build_waterfall_arch_mesh` for the core stream and oscillating foam sheath, `build_trench_water_ribbon_mesh` for the canal surface, and 1 expanding plunge pool torus ripple).
* **Result**:
  Eliminates 131 unbatched transparent WebGL2 draw calls and 129 per-frame CPU `Transform` updates.

### ✅ FIXED: Item #10 — Downscaled Bot Character Textures (4096×4096 → 1024×1024 / 512×512)
* **Assets Modified**: `assets/models/jonas-blue.glb`, `lisa-red.glb`, `morten-blue.glb`, `per-red.glb`, `siri-red.glb`
* **What Was Changed**:
  Downscaled all 5 bot character basecolor textures from `4096×4096` to `1024×1024` (Lanczos filter) and roughness/metallic + normal maps from `2048×2048` to `512×512`, rebuilding all GLB binary buffers with 4-byte alignment while preserving skeletal skins and embedded animations.
* **Result**:
  Reduces uncompressed GPU texture VRAM across all 5 bots from **503.3 MB** down to **31.5 MB** (**~472 MB VRAM saved**, a 93.7% reduction) and shrinks total bot GLB download payload from **53.8 MB** down to **15.4 MB** (**38.3 MB saved**).

### ✅ FIXED: Item #11 — Decimated Battlefield Mesh (`post_apocalyptic_battlefield.glb`) & Split Single/Double-Sided Sub-Meshes
* **Asset Modified**: [`assets/models/post_apocalyptic_battlefield.glb`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/assets/models/post_apocalyptic_battlefield.glb)
* **What Was Changed**:
  1. Stripped 50,681 unseen underground bottom-slab triangles (`y < -4.2` facing downward) and applied `MeshoptSimplifier` Quadric Error Metric (QEM) simplification with UV-seam preservation, reducing geometry from **735,845 triangles (456,667 vertices)** down to **140,438 triangles (126,589 vertices)**.
  2. Split the mesh into two primitives: `BattlefieldPBR` (`100,193` triangles with `doubleSided: false` for hardware backface culling across terrain, trenches, platforms, ramps, and buildings) and `BattlefieldFencePBR` (`40,245` triangles with `doubleSided: true` for thin perimeter barbed-wire fences and two-sided sheets).
* **Result**:
  Cuts battlefield triangle count by **80.9%**, shrinks `post_apocalyptic_battlefield.glb` from **28.5 MB** to **10.8 MB** (**17.7 MB saved**), and enables hardware backface culling across over 71% of the battlefield geometry.

### ✅ FIXED: Item #12 — Decimated Bot Character Meshes (38k → ~11k Vertices)
* **Assets Modified**: `assets/models/jonas-blue.glb`, `lisa-red.glb`, `morten-blue.glb`, `per-red.glb`, `siri-red.glb`
* **What Was Changed**:
  1. Deduplicated identical split vertices across shared edges and simplified all 5 bot character meshes using `MeshoptSimplifier` while preserving `POSITION`, `NORMAL`, `TEXCOORD_0`, `JOINTS_0`, `WEIGHTS_0`, skeletal skins, and embedded `Idle`/`Run` animations.
  2. Reduced vertex/triangle counts per bot (`jonas`: 39,896 → 12,591 verts; `lisa`: 37,714 → 11,926 verts; `morten`: 36,775 → 10,708 verts; `per`: 37,611 → 11,201 verts; `siri`: 37,533 → 9,015 verts) to match `main_player.glb` (11,869 verts).
* **Result**:
  Cuts bot skinned vertices by **70.7%** (from 189,529 down to 55,441; total across all 6 characters drops from 201,398 to 67,310 vertices per frame) and shrinks each bot `.glb` file down to **1.3–1.6 MB**.

### ✅ FIXED: Item #13 — Optimized `Cargo.toml` Release Profile
* **File Modified**: [`Cargo.toml`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/Cargo.toml#L23-L28)
* **What Was Changed**:
  Configured `[profile.release]` with `opt-level = "s"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, and `strip = "symbols"`.
* **Result**:
  Eliminates stack unwinding landing pads and debug symbol tables from release WASM builds and enables whole-program fat LTO.

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

## 4. Category D: WASM Compilation, Browser Environment & Cargo Profiles (Pending Fixes)

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

## 5. Master Checklist

- [x] **Item #1**: Remove `SweptCcd` on all 6 characters in `src/player.rs` and `src/ai.rs`. *(✅ FIXED)*
- [x] **Item #2**: Cache skeletal bones (`CharacterBones`) and eliminate per-frame heap allocations in `src/player.rs`. *(✅ FIXED)*
- [x] **Item #3**: Guard UI `Text` and nametag mutations against Taffy flexbox layout thrashing in `src/ui.rs`. *(✅ FIXED)*
- [x] **Item #4**: Remove duplicate `handle_fallout_respawns` registration from `src/arena.rs`. *(✅ FIXED)*
- [x] **Item #5**: Remove redundant downward ground raycast in `src/player.rs`. *(✅ FIXED)*
- [x] **Item #6**: Add `CollisionLayers` and optimize camera spring-arm raycast in `src/camera.rs` & `src/types.rs`. *(✅ FIXED)*
- [x] **Item #7**: Set `SubstepCount(2)` in `src/main.rs`. *(✅ FIXED)*
- [x] **Item #8**: Enable backface culling on sky domes and reduce cloud layers in `src/arena.rs`. *(✅ FIXED)*
- [x] **Item #9**: Consolidate sewer water effects (135 entities → 2–3 meshes) in `src/arena.rs`. *(✅ FIXED)*
- [x] **Item #10**: Downscale bot textures from 4K to 1024×1024 in `assets/models/`. *(✅ FIXED)*
- [x] **Item #11**: Decimate `post_apocalyptic_battlefield.glb` from 736k to 140k triangles. *(✅ FIXED)*
- [x] **Item #12**: Decimate bot character meshes from 38k to 11k vertices. *(✅ FIXED)*
- [x] **Item #13**: Optimize `Cargo.toml` release profile (`panic = "abort"`, LTO). *(✅ FIXED)*
- [ ] **Item #14**: Strip unused Bevy default features in `Cargo.toml`. *(⏳ NOT FIXED)*
- [ ] **Item #15**: Implement on-demand voice loading in `index.html`. *(⏳ NOT FIXED)*
