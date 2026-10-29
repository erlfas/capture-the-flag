# 3D Character Model Integration & Animation Pipeline (`CHARACTER_MODEL_PIPELINE.md`)

This document summarizes the key technical insights and lessons learned while integrating the Tripo 3D `.glb` character models (`main_player.glb`, `morten-blue.glb`, `jonas-blue.glb`, `siri-red.glb`, `lisa-red.glb`, `per-red.glb`) into **Cyber-Athletic Coliseum** (Bevy `0.15` + Avian3D + Trunk WebAssembly).

---

## 1. Why Characters Can Appear Invisible in Bevy 0.15 (Critical Failure Modes)

When a character's procedural underfoot selection ring (`Torus`) renders in-game but the 3D `.glb` model (`SceneRoot`) is completely invisible, check these three root causes in order:

### A. Embedded `image/jpeg` Textures Without Bevy's `"jpeg"` Feature (Most Common)
- **Symptom**: `main_player.glb` renders normally, while newly added Tripo `.glb` characters are completely invisible (only their ground rings and floating nameplates appear).
- **Root Cause**:
  - `main_player.glb` embeds a single PNG texture (`"mimeType": "image/png"`).
  - Raw Tripo 3D exports (`per-red.glb`, `siri-red.glb`, `jonas-blue.glb`, `morten-blue.glb`, `lisa-red.glb`) embed **JPEG** textures (`<name>_basecolor.jpg` and `<name>_normal.jpg` with `"mimeType": "image/jpeg"`).
  - In Bevy `0.15`, `default-features = true` only enables `png`, `hdr`, and `ktx2` on the underlying `image` crate—it **does not enable `jpeg`**!
  - When `bevy_gltf::GltfLoader` encounters an embedded `"image/jpeg"` buffer view without the `"jpeg"` feature compiled in, it aborts loading the entire `.glb` asset and never spawns the `SceneRoot` hierarchy.
- **Fix**: Ensure `"jpeg"` is explicitly included in [`Cargo.toml`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/Cargo.toml#L7):
  ```toml
  bevy = { version = "0.15.3", default-features = true, features = ["webgl2", "mp3", "jpeg"] }
  ```

### B. Missing `Visibility::default()` on the Parent Physics Entity
- **Root Cause**: In Bevy `0.15`, spawning a `SceneRoot` as a child of a physics entity (`Bot` or `Player`) requires the parent entity to have `Visibility::default()` (alongside `Transform`), otherwise `InheritedVisibility` does not propagate to the spawned glTF meshes.
- **Fix**: Always insert `Visibility::default()` on the parent character entity inside [`spawn_character_visuals`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/player.rs#L65-L116).

### C. Bevy 15-Element Tuple `Bundle` Limit
- **Root Cause**: `commands.spawn((...))` only implements `Bundle` for tuples up to **15 elements**. Adding a 16th component directly inside the tuple fails to compile with `is not a Bundle`.
- **Fix**: Keep `.spawn((...))` at 15 or fewer components and chain `.insert(SweptCcd::default())` (or additional components) afterwards.

---

## 2. Raw Tripo 3D `.glb` Exports vs. Bevy 0.15 Animation Requirements

When exporting auto-rigged Mixamo-style (`mixamorig:*`) characters from Tripo 3D in a T-pose, the raw `.glb` files have three structural traits that must be normalized for skeletal animation:

1. **Matrix Transforms Instead of Decomposed TRS (`translation`, `rotation`, `scale`)**:
   - Raw Tripo exports store 4×4 column-major `"matrix"` arrays on every node (`node_0`, `Armature`, `tripo_node_*`, and all `mixamorig:*` bones).
   - **Rule**: According to the glTF 2.0 specification and Bevy's `AnimationPlayer`, any node targeted by an animation channel (or in the animated skeleton hierarchy) **must** use decomposed `"translation"`, `"rotation"` (quaternion `[x, y, z, w]`), and `"scale"` properties rather than `"matrix"`.
2. **Unnamed Root Nodes Break `AnimationTargetId` Path Hashing**:
   - Raw Tripo exports include an unnamed scene root node (`nodes[0] = {"children": [1]}`).
   - **Rule**: Bevy `0.15` computes `AnimationTargetId` by hashing the hierarchy path of `Name` components from the `AnimationPlayer` ancestor down to each bone. Ensure every node in `nodes` has a non-empty `"name"` (e.g. `"Root_0"` if missing).
3. **Static T-Pose When Exported Without Embedded Clips**:
   - If a `.glb` is exported from Tripo with `animations: []`, Bevy will not attach an `AnimationPlayer` or `AnimationTarget` components, leaving the character frozen in a horizontal T-pose.
   - **Solution**: Bake `Idle` (`Animation(0)`) and `Run` (`Animation(1)`) directly into each character's `.glb` binary buffer and JSON chunks so every model is self-contained.

---

## 3. World-Space Delta Retargeting Across Characters

Even when two characters share identical `mixamorig:*` bone names, auto-riggers can assign different local bone roll axes (e.g., on `mixamorig:LeftShoulder` / `mixamorig:RightShoulder` or `mixamorig:Hips` parent wrappers). Copying raw local quaternions from one `.glb` to another twists arms or shoulders.

Instead, use **World-Space Delta Retargeting**:

1. For each bone $b$ in the reference model (`main_player.glb`), compute the world-space rotational delta from its bind pose $R_{\text{bind}}^{\text{ref}}(b)$ at keyframe $k$:
   $$\Delta R_{\text{world}}(b, k) = R_{\text{anim}}^{\text{ref}}(b, k) \cdot \left(R_{\text{bind}}^{\text{ref}}(b)\right)^T$$
2. Apply that same world-space delta to the target character's T-pose world bind orientation $R_{\text{bind}}^{\text{target}}(b)$:
   $$R_{\text{anim}}^{\text{target}}(b, k) = \Delta R_{\text{world}}(b, k) \cdot R_{\text{bind}}^{\text{target}}(b)$$
3. Convert back to the target bone's local space top-down using the parent's animated world rotation:
   $$R_{\text{local}}^{\text{target}}(b, k) = \left(R_{\text{anim}}^{\text{target}}(\text{parent}(b), k)\right)^T \cdot R_{\text{anim}}^{\text{target}}(b, k)$$
4. Enforce quaternion hemisphere continuity across keyframes ($\mathbf{q}_k \cdot \mathbf{q}_{k-1} \ge 0$, negating $\mathbf{q}_k$ if negative) to prevent 360° spin artifacts during `LINEAR` interpolation.
5. For `mixamorig:Hips` translation (vertical run bobbing $\Delta \mathbf{t}_{\text{world}}(k)$), transform the world delta into `Hips`'s parent local space:
   $$\mathbf{t}_{\text{local}}^{\text{target}}(\text{Hips}, k) = \mathbf{t}_{\text{local, rest}}^{\text{target}}(\text{Hips}) + \left(M_{\text{parent}}^{\text{target}}(\text{Hips})\right)_{3\times 3}^{-1} \Delta \mathbf{t}_{\text{world}}(k)$$

---

## 4. Per-Character `AnimationGraph` in Bevy 0.15

- Do **not** share a single `Handle<AnimationGraph>` loaded from `main_player.glb` across different `.glb` files when their internal wrapper node names (e.g. `tripo_node_<uuid>` or `Root_0 -> node_0 -> RootNode -> Armature`) differ, because Bevy bakes the full node path into each clip's `AnimationTargetId`.
- Instead, attach a [`CharacterAnimClips`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/player.rs#L21-L26) component per character entity in [`spawn_character_visuals`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/player.rs#L65-L116) that loads `GltfAssetLabel::Animation(0)` and `GltfAssetLabel::Animation(1)` from **that character's own `.glb` file**.

---

## 5. Bounding Box Normalization & Scale Matching

Tripo normalizes exported meshes into a $[-0.5, +0.5]$ unit cube along the **largest** dimension:
- For slender/athletic characters (`main_player.glb`, `siri-red.glb`, `lisa-red.glb`, `jonas-blue.glb`, `morten-blue.glb`), height ($Y$) is the largest dimension ($\approx 0.98\text{--}1.00$ units), so a uniform scale of `1.88`–`1.92` yields a $\sim 1.88\text{ m}$ tall character.
- For broad-shouldered characters with wide T-pose arm spans (`per-red.glb`), the arm span ($X \approx 0.98$) hits the unit-cube limit first, leaving the $Y$ height at $\approx 0.905$ units. Scaling `per-red.glb` by `2.08` compensates for this so Per stands at the proper $\sim 1.88\text{ m}$ height alongside the other operators.
- Always place the `SceneRoot` at `Transform::from_xyz(0.0, -0.90, 0.0)` so the model's feet (`y = 0.0` in local glTF space) rest cleanly on the ground plane inside the `Collider::capsule(0.35, 1.2)`.

---

## 6. Quick Diagnostic One-Liner for New `.glb` Models

Before testing a new `.glb` model in the browser, run this Python snippet to verify its texture formats, node names/TRS decomposition, animations, and skinned bounds:

```powershell
python -c "import struct, json; data = open('assets/models/per-red.glb', 'rb').read(); jlen = struct.unpack('<I', data[12:16])[0]; d = json.loads(data[20:20+jlen]); print('Images:', [img.get('mimeType') for img in d.get('images', [])]); print('Anims:', [a.get('name') for a in d.get('animations', [])]); print('Unnamed nodes:', [i for i, n in enumerate(d['nodes']) if not n.get('name')]); print('Matrix nodes:', [i for i, n in enumerate(d['nodes']) if 'matrix' in n])"
```

Expected healthy output for Bevy `0.15`:
- `Images`: `'image/png'` or `'image/jpeg'` (with `"jpeg"` feature enabled in `Cargo.toml`)
- `Anims`: `['Idle', 'Run']`
- `Unnamed nodes`: `[]`
- `Matrix nodes`: `[]`

---

## 7. Trunk & WSL Build Workflow

1. **Asset Placement**: Keep source `.glb` files in `players/` copied to `assets/models/`. Trunk bundles `assets/` via `<link rel="copy-dir" data-trunk href="assets"/>` in [`index.html`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/index.html#L148). Avoid duplicating `.glb` files into extra subdirectories inside `assets/`.
2. **Sync & Build in WSL**: Running `cargo` on Windows inside synced folders can trigger `Access is denied (os error 5)` on build-script executables. Always sync to the native Linux filesystem in WSL (`~/repos/cyber_coliseum_koth`) and run `CARGO_TARGET_DIR=~/cargo_target trunk build` (or `trunk serve`) as documented in [`SERVER_GUIDE.md`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/SERVER_GUIDE.md).
