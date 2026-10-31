# Key Technical Lessons Learned — Cyber-Athletic Coliseum

This document summarizes the key engineering lessons learned while refining the 3D battlefield geometry, atmospheric sky & industrial horizon systems, 3rd-person action camera, and skeletal character animations.

---

## 1. 3D Mesh Decimation & UV-Seam Integrity (`post_apocalyptic_battlefield.glb`)

### The Problem: Polygon Gaps Along Texture Seams
- **Symptom**: Visible holes and missing polygon strips appeared across sandbags, cars, shipping containers, and terrain in `assets/models/post_apocalyptic_battlefield.glb`.
- **Root Cause**: A previous mesh simplification pass clustered vertices by **3D position only** (`(x, y, z)`) and discarded any triangle whose UV edge exceeded `0.02` in UV space to avoid texture stretching across UV chart boundaries. That deleted **37,937 triangles** along every UV seam, leaving **69,095 open boundary edges** and single-sided backface culling holes.

### The Fix: UV-Seam-Preserving 3D Vertex Clustering
- When simplifying a textured mesh from high-poly source geometry (`tripo_convert_2717f1af-db81-49e5-8c67-f8df9fd36fb6.fbx`, `997,462` vertices / `1,994,920` faces):
  1. **Include UV coordinates in the clustering key**: Quantize vertices by `(round(x / g_xyz), round(y / g_xyz), round(z / g_xyz), round(u / g_uv), round(v / g_uv))` (or preserve distinct UV vertices whenever `|ΔUV| > 0.008` at a shared 3D position).
  2. **Never delete triangles that span UV seams**: Because vertices on opposite sides of a UV seam keep their own UV coordinates while sharing identical 3D positions, seam triangles remain 100% watertight without any texture stretching.
  3. **Enable `"doubleSided": true` in the glTF material**: Prevents thin open structures (barbed-wire fences, container walls, watchtower struts) from disappearing when viewed from behind.
- **Result**: Open boundary edges dropped from **69,095** to **2** across `735,845` triangles (`27.17 MB`).

### Trimming Baked AI Concept-Art Backdrop Walls
- Image-to-3D models generated from 2D concept art often extrude the background skyline into a tall, unpainted clay-like rim around the outer edge of the mesh (`z < -16.0` and `|x| > 24.5` rising up to `y = 16.5m`).
- **Lesson**: Smoothly lower vertices outside the playable fence perimeter (`z < -16.0`, `x < -24.5`, `x > 24.2`) down to fence height (`y <= 4.2m..5.1m`) while keeping interior structures (watchtowers, flagpoles, ramps) untouched. This prevents the outer clay wall from blocking the 360° sky dome.

---

## 2. Painted 360° Sky Domes vs. 3D Primitive Clouds (`src/arena.rs`)

### The Problem: Unlit 3D Primitive Clouds Look Flat and Artificial
- **Symptom**: Spawning dozens of scaled 3D `Sphere` and `Capsule3d` primitives with flat unlit red/orange/blue materials created harsh geometric ovals in 3D perspective (`bad_skies.png`) that clashed with the painted aesthetic of the concept art.

### The Fix: Custom UV-Mapped 360° Panorama Dome + Pure Building-Free Cloud Layers
- **Use 2D painted artwork mapped onto a custom 360° sky dome mesh (`build_sky_dome_mesh`)**:
  1. **Seamless Horizontal Wrapping**: Resize high-resolution concept art (`art/scene/dark_skies.jpg`) with a 16%–18% width overlap and apply a cosine crossfade (`0.5 - 0.5 * cos(π * t)`) so `u = 0.0` and `u = 1.0` tile with zero visible seam.
  2. **Zenith & Ground Feathering**: Blend the top 10% (`v < 0.10`) into a uniform deep storm-charcoal color (matching a flat zenith cap) to avoid polar pinching at the top of the dome, and blend the bottom (`v > 0.88`) into dark horizon soot.
  3. **Strict Separation of Stationary Horizon vs. Moving Cloud Layers (`ScrollingSkyLayer`)**:
     - **Never include horizon buildings, chimneys, or ground in rotating cloud textures** (blending full-height `chimnies_smoke.jpg` into scrolling layers caused factories and wasteland dirt to float and drift across the sky).
     - Extract `storm_clouds_layer1.png` and `storm_clouds_layer2.png` **exclusively from the building-free upper cloud region (`y = 20..1020` of `1536`) of `art/scene/dark_skies.jpg`**, using luminance-contrast cloud density so dark storm billows are opaque while lighter rifts are transparent (allowing the warm amber sunset glow to shine through between moving clouds).
  4. **Calibrating Viewport Elevation (`y_horizon` to `y_mid`)**: In 3rd-person action view (`y ≈ 3.8m`), the visible sky above the compound fence (`y ≈ 4.5m..5.5m`) projects onto `y ≈ 5.0m..28.0m` at `radius = 77m..108m`. Mapping `v ∈ [0.22, 0.85]` across `[y_horizon, y_mid] = [3.2m, 27.0m]` places the richest rolling storm clouds directly inside the player's sky viewport while keeping the stationary skyline silhouette anchored low at the horizon (`y ≈ 2.6m..4.5m`).

---

## 3. 3rd-Person Action Camera & Wall-Occlusion Recovery (`src/camera.rs`)

### The Problem: Camera Blocked by Perimeter Walls & Fences
- **Symptom**: With a follow camera `6.5m` behind the character (`CameraMode::ThirdPersonAction`), backing up near a perimeter fence pushed the camera outside the `6.2m`-tall fence mesh, completely blocking the view of the player.

### The Fix: Three-Layer Anti-Occlusion System
1. **Inner-Perimeter XZ Clamping**: Clamp `desired_cam_pos.x` and `desired_cam_pos.z` inside `playable_min_x(z) + 0.55`, `playable_max_x(z) - 0.55`, and `[PLAYABLE_MIN_Z + 0.55, PLAYABLE_MAX_Z - 0.55]` so the camera never crosses to the outside of the compound fence.
2. **Physics Spring-Arm Raycast (`SpatialQuery`)**: Cast a ray from the character's upper chest (`translation + Vec3::Y * 0.85`) toward `desired_cam_pos` (excluding the player entity) and pull the camera in ahead of any interior obstacle hit (`hit.distance - 0.45`).
3. **Automatic Overhead Look-Down Elevation (`wall_compression`)**: When the horizontal distance between the clamped camera and the player is compressed (`horiz_dist < 6.5m`), compute `wall_compression = (1.0 - horiz_dist / 6.5)^1.15` and smoothly:
   - Lift the camera upward by up to **`+5.2m`** (`desired_cam_pos.y += wall_compression * 5.2`), and
   - Shift the look-at target slightly forward (`+1.85m`) and down (`-0.75m`) so the camera transitions into a clear **top-down / high-angle view** over the character's head whenever near a wall.

---

## 4. Skeletal Animation & Procedural Upper-Body Running Pose (`src/player.rs`)

### The Problem: Forearms and Hands Hanging Straight Down While Running
- **Symptom**: Even when the GLB `Run` animation (`Animation(1)`) was active, characters ran with their forearms and hands hanging nearly straight down toward their hips/thighs.
- **Root Cause**: Inspecting the binary keyframe tracks of `main_player.glb` and the bot `.glb` models revealed that `mixamorig:LeftForeArm` and `mixamorig:RightForeArm` had a nearly constant local rotation with only a **24° elbow bend** (`y = -0.60..-0.89` in model space) and wide outward flare (`x = ±0.45..0.51`).

### The Fix: Footstep-Synchronized Model-Space Bone Orientation in `PostUpdate`
- In Bevy `0.15`, procedural bone adjustments must run in `PostUpdate` **`.after(bevy::animation::animate_targets).before(TransformSystem::TransformPropagate)`**.
- Using `orient_bone_in_model_space` (which computes the parent chain rotation from the character root down to the bone and rotates the bone's local `+Y` axis toward a model-space target direction):
  1. **Contralateral Footstep Synchronization**: Read the animated model-space Z directions of `mixamorig:LeftUpLeg` and `mixamorig:RightUpLeg` (`leg_diff = l_dir.z - r_dir.z`). When the left leg swings forward (`+Z`), the right arm swings forward and the left arm swings back, keeping arm swing 100% locked to footfalls.
  2. **Athletic ~90° Elbow Bend**: Orient `mixamorig:LeftForeArm` and `mixamorig:RightForeArm` strongly forward (`z = 0.88`) and upward (`y = 0.24 + swing * 0.28`, tucked slightly inward `x = ∓0.12..0.22`) so the hands pump across the chest on the forward swing and stay bent beside the waist on the backswing.
  3. **Firm Wrist Alignment & Punch Blending**: Align `mixamorig:LeftHand` and `mixamorig:RightHand` with the forearm vector (`+0.06` Y pitch) so wrists never droop, and smoothly blend into the forward punch thrust (`punch_envelope`) when `Space` / `LMB` / `F` is pressed.
