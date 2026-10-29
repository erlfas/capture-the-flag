# 3D Character Model Integration & Animation Pipeline (`CHARACTER_MODEL_PIPELINE.md`)

This document summarizes the key technical lessons learned while integrating the Tripo 3D `.glb` character models (`main_player.glb`, `morten-blue.glb`, `jonas-blue.glb`, `siri-red.glb`, `lisa-red.glb`, `per-red.glb`) into **Cyber-Athletic Coliseum** (Bevy `0.15` + Avian3D + Trunk WebAssembly).

---

## 1. Raw Tripo 3D `.glb` Exports vs. Bevy 0.15 `GltfLoader` Requirements

When exporting auto-rigged Mixamo-style (`mixamorig:*`) characters from Tripo 3D in a T-pose, the raw `.glb` files have three characteristics that must be normalized before Bevy 0.15 can animate them cleanly:

1. **Matrix Transforms Instead of TRS (`translation`, `rotation`, `scale`)**:
   - Raw Tripo exports store 4×4 column-major `"matrix"` arrays on every node (`node_0`, `Armature`, `tripo_node_*`, and all `mixamorig:*` bones).
   - **Rule**: According to the glTF 2.0 specification and Bevy's `AnimationPlayer`, any node targeted by an animation channel (or in the animated skeleton hierarchy) **must** use decomposed `"translation"`, `"rotation"` (quaternion `[x, y, z, w]`), and `"scale"` properties rather than `"matrix"`.
2. **Unnamed Root Nodes Break `AnimationTargetId` Hashing**:
   - Raw Tripo exports often have an unnamed root node (`nodes[0] = {"children": [1]}`).
   - **Rule**: Bevy 0.15 computes `AnimationTargetId` by hashing the hierarchy path of `Name` components from the `AnimationPlayer` root down to each bone. Ensure every node in `nodes` has a non-empty `"name"` (e.g. `"Root_0"` if missing).
3. **Static T-Pose When Exported Without Embedded Clips**:
   - If a `.glb` is exported from Tripo with `animations: []`, Bevy will not attach an `AnimationPlayer` or `AnimationTarget` components, leaving the character frozen in a horizontal T-pose.
   - **Solution**: Bake `Idle` (`Animation(0)`) and `Run` (`Animation(1)`) directly into each character's `.glb` binary buffer and JSON chunks so every model is self-contained.

---

## 2. World-Space Delta Retargeting Across Characters

Even when two characters share the `mixamorig:*` bone names, auto-riggers can assign slightly different local bone roll axes (e.g., on `mixamorig:LeftShoulder` / `mixamorig:RightShoulder` or `mixamorig:Hips` parent wrappers). Copying raw local quaternions from one `.glb` to another can twist arms or shoulders.

Instead, use **World-Space Delta Retargeting**:

1. For each bone $b$ in the reference model (`main_player.glb`), compute the world-space rotational delta from its bind pose $R_{\text{bind}}^{\text{ref}}(b)$ at keyframe $k$:
   $$\Delta R_{\text{world}}(b, k) = R_{\text{anim}}^{\text{ref}}(b, k) \cdot \left(R_{\text{bind}}^{\text{ref}}(b)\right)^T$$
2. Apply that same world-space delta to the target character's T-pose bind orientation $R_{\text{bind}}^{\text{target}}(b)$:
   $$R_{\text{anim}}^{\text{target}}(b, k) = \Delta R_{\text{world}}(b, k) \cdot R_{\text{bind}}^{\text{target}}(b)$$
3. Convert back to the target bone's local space top-down using the parent's animated world rotation:
   $$R_{\text{local}}^{\text{target}}(b, k) = \left(R_{\text{anim}}^{\text{target}}(\text{parent}(b), k)\right)^T \cdot R_{\text{anim}}^{\text{target}}(b, k)$$
4. Ensure quaternion hemisphere continuity across keyframes ($\mathbf{q}_k \cdot \mathbf{q}_{k-1} \ge 0$, negating $\mathbf{q}_k$ if negative) to prevent 360° spin artifacts during `LINEAR` interpolation.

Why this works reliably:
- All Tripo T-pose exports share the same world orientation ($+Y$ up, $+Z$ forward, $\pm X$ horizontal arms).
- The resulting skinning rotation matrix $S^{\text{target}}(b, k) = R_{\text{anim}}^{\text{target}}(b, k) (R_{\text{bind}}^{\text{target}}(b))^T = \Delta R_{\text{world}}(b, k)$ is invariant to internal node wrapper or bone-roll differences.

---

## 3. Per-Character `AnimationGraph` in Bevy 0.15

- Do **not** share a single `Handle<AnimationGraph>` loaded from `main_player.glb` across different `.glb` files if their internal node wrapper names (e.g. `tripo_node_<uuid>` or `Root_0 -> node_0 -> RootNode -> Armature`) differ, because Bevy bakes the node path into each clip's `AnimationTargetId`.
- Instead, attach a `CharacterAnimClips` component per character entity in [`spawn_character_visuals`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/player.rs#L65-L116) that loads `GltfAssetLabel::Animation(0)` and `GltfAssetLabel::Animation(1)` from **that character's own `.glb` file**.

---

## 4. Bounding Box Normalization & Scale Matching

Tripo normalizes exported meshes into a $[-0.5, +0.5]$ unit cube along the **largest** dimension:
- For slender/athletic characters (`main_player.glb`, `siri-red.glb`, `lisa-red.glb`, `jonas-blue.glb`, `morten-blue.glb`), height ($Y$) is the largest dimension ($\approx 0.98\text{--}1.00$ units), so a uniform scale of `1.88`–`1.92` yields a $\sim 1.88\text{ m}$ tall character.
- For broad-shouldered characters with wide T-pose arm spans (`per-red.glb`), the arm span ($X \approx 0.98$) hits the unit-cube limit first, leaving the $Y$ height at $\approx 0.905$ units. Scaling `per-red.glb` by `2.08` compensates for this so Per stands at the proper $\sim 1.88\text{ m}$ height alongside the other operators.
- **Tip**: Always inspect the world-space $Y$ min/max of vertex positions when adding a new `.glb` so feet sit at `y = -0.90` relative to the physics capsule center and head height matches the rest of the roster.

---

## 5. Bevy ECS & Trunk / WSL Workflow Gotchas

1. **Bevy 15-Element Tuple `Bundle` Limit**:
   - `commands.spawn((...))` only implements `Bundle` for tuples up to **15 elements**. If adding components like `Visibility::default()` or `SweptCcd::default()` pushes a spawn tuple to 16 elements, chain `.insert(...)` after `.spawn((...))`.
2. **Parent `Visibility::default()` Required for `SceneRoot` Children**:
   - Any parent entity (such as a `Bot` or `Player` physics body) that spawns a `SceneRoot` child **must** have `Visibility::default()` on the parent entity, otherwise Bevy's visibility propagation will not render the child glTF scene.
3. **Asset Placement for Trunk (`assets/models/`)**:
   - Trunk bundles assets via `<link rel="copy-dir" data-trunk href="assets"/>` in [`index.html`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/index.html#L148), which maps `assets/models/<name>.glb` to Bevy's `"models/<name>.glb"` asset path. Keep source `.glb` files in `players/` synced to `assets/models/`, and avoid duplicating large `.glb` files into extra subdirectories inside `assets/` to keep WebAssembly bundle size lean.
4. **Building in WSL Avoids Windows File-Locking (`os error 5`)**:
   - Running `cargo` on Windows inside synced folders can trigger `Access is denied (os error 5)` on build-script executables. Always sync to the native Linux filesystem in WSL (`~/repos/cyber_coliseum_koth`) with `CARGO_TARGET_DIR=~/cargo_target` as documented in [`SERVER_GUIDE.md`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/SERVER_GUIDE.md).
