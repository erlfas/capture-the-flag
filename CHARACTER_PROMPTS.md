# Character T-Pose Generation Prompts (Tripo 3D Pipeline)

This document contains generation prompts for Google's image models (Imagen 3 / Gemini) to produce clean, symmetrical full-body **T-pose** reference images for all secondary characters in *Cyber-Athletic Coliseum*. These images are formatted specifically for ingestion into **Tripo 3D** to generate auto-rigged, animated 3D character meshes matching Anders (`assets/models/main_player.glb`).

---

## Technical Specifications for Tripo 3D Compatibility

- **Pose**: Symmetrical frontal **T-pose** (arms straight horizontal at 90°, palms down/forward, fingers relaxed & separated, legs straight and spaced apart, feet flat on ground).
- **Framing**: Full-body head-to-toe with generous margins on all sides (no cropped boots, hair, or hands).
- **Background**: Plain, solid neutral studio gray (`#808080`) or clean white, free of scenery, ground decals, or harsh cast shadows.
- **Visual Style**: Stylized 3D animated hero / cel-shaded comic aesthetic matching Anders's in-game look and portrait.
- **Props**: No held weapons, shields, or accessories that could fuse to the body mesh during 3D extraction.

### Recommended Negative Prompt
```text
cropped limbs, cut-off feet, cut-off head, dynamic action pose, walking, running, bent arms, holding weapons, holding guns, cluttered background, realistic photographic skin pores, harsh directional shadows, high depth of field blur, ground reflections
```

---

## 1. Morten (`Team Blå` — Vanguard / Flag Striker)

- **Role & Team**: Team Blue Vanguard / Striker
- **Visual Profile**: Male tactical operator, athletic muscular build, mid-30s, styled dark brown hair, prominent thick full facial beard and mustache, navy tactical jumpsuit, dark tactical plate carrier vest, tactical headset.

### Prompt:
```text
Full-body 3D character design of Morten, a male tactical athletic operator in a strict symmetrical T-pose, viewed straight-on at eye level. Athletic muscular build, mid-30s, styled dark brown hair, prominent thick full facial beard and mustache. Wearing a navy blue tactical jumpsuit with subtle cyan team piping, dark charcoal tactical plate carrier vest with chest magazine pouches, black utility belt, reinforced black knee pads, and sturdy black combat boots. Wearing a black single-ear tactical comms headset with a small boom microphone near his mouth. Symmetrical frontal orthographic perspective, arms stretched straight out horizontally at 90 degrees with fingers relaxed and spread, legs straight shoulder-width apart, feet facing forward. Isolated on a solid plain neutral light gray studio background with soft ambient diffuse lighting. Stylized 3D animated hero game asset, cel-shaded comic aesthetic, crisp edges, clean silhouette, no shadows, no background objects, entire body completely visible within frame from head to toe.
```

---

## 2. Jonas (`Team Blå` — Base Defender / Tactical Comms)

- **Role & Team**: Team Blue Defender & Comms
- **Visual Profile**: Male tactical operator, lean athletic build, late 20s, short neat dark crew cut, clean jaw with faint stubble, navy tactical jumpsuit, tactical harness/chest rig, utility pouches, tactical comms headset with boom mic.

### Prompt:
```text
Full-body 3D character design of Jonas, a male tactical defense and communications operator in a strict symmetrical T-pose, viewed straight-on at eye level. Lean athletic build, late 20s, short neat dark brown crew cut hair, clean-shaven with faint tactical stubble along a defined jawline. Wearing a navy blue tactical jumpsuit with cyan blue accents, dark matte tactical chest rig plate carrier with radio pouches and antennas, utility duty belt with tech pouches, black knee pads, and black combat boots. Wearing a black tactical headset with boom microphone. Symmetrical frontal orthographic perspective, arms extended straight out horizontally at 90 degrees with palms down and fingers spread, legs straight and slightly separated, feet pointing forward. Isolated on a solid plain neutral light gray studio background with soft ambient studio lighting. Stylized 3D animated hero game asset, cel-shaded clean aesthetic, crisp readable silhouette, no floor shadows, no props, entire body head to toe fully centered in frame.
```

---

## 3. Siri (`Team Rød` — Lead Striker / Flag Runner)

- **Role & Team**: Team Red Lead Striker / Runner
- **Visual Profile**: Female tactical operator, agile athletic runner build, late 20s, dark brown hair in a sleek high ponytail, confident features, crimson red tactical jumpsuit, lightweight chest plate carrier, tactical headset with boom mic.

### Prompt:
```text
Full-body 3D character design of Siri, a female tactical sprint operator in a strict symmetrical T-pose, viewed straight-on at eye level. Agile athletic runner build, late 20s, dark brown hair pulled back into a sleek high ponytail held by a black tactical band, sharp confident facial features. Wearing a vibrant crimson red tactical athletic jumpsuit, dark charcoal lightweight plate carrier vest, dark tactical utility belt with small pouches, streamlined black knee pads, and sleek black combat boots. Wearing a black tactical comms headset with a slim boom microphone. Symmetrical frontal orthographic perspective, arms straight out horizontally at 90 degrees, palms facing down, fingers spread, legs straight with slight gap between thighs, feet flat and facing forward. Isolated on a solid plain neutral light gray studio background with soft diffuse ambient lighting. Stylized 3D animated character design, cel-shaded esports comic aesthetic, clean edges, zero background props, entire character completely in frame from top of hair to bottom of boots.
```

---

## 4. Lisa (`Team Rød` — Flanker / Scout)

- **Role & Team**: Team Red Flanker / Scout
- **Visual Profile**: Female tactical operator, nimble athletic build, mid-20s, bright golden blonde hair in a sporty high ponytail, crimson red tactical jumpsuit, tactical plate carrier, knee pads, combat boots, tactical comms headset.

### Prompt:
```text
Full-body 3D character design of Lisa, a female tactical scout operator in a strict symmetrical T-pose, viewed straight-on at eye level. Nimble athletic build, mid-20s, bright golden blonde hair tied in a sporty high ponytail, friendly yet focused expression. Wearing a crimson red tactical jumpsuit with subtle red piping, dark matte tactical plate carrier vest, tactical waist belt, black reinforced knee pads, and black combat boots. Wearing a black single-ear tactical headset with boom mic. Symmetrical frontal orthographic perspective, arms spread straight out horizontally at 90 degrees with hands open and fingers separated, legs standing straight shoulder-width apart, feet pointing forward. Isolated on a solid plain neutral light gray studio background with flat studio lighting. Stylized 3D animated character, Pixar and Overwatch inspired cel-shaded style, high-contrast readable silhouette, no shadows or clutter, full body completely visible from head to toe with margin around edges.
```

---

## 5. Per (`Team Rød` — Heavy Defender / Enforcer)

- **Role & Team**: Team Red Heavy Enforcer / Defender
- **Visual Profile**: Male tactical enforcer, broad-shouldered heavy muscular build, early 30s, dark buzz cut hair, rugged square chiseled jawline with 5 o'clock shadow, heavy crimson jumpsuit, reinforced heavy-duty plate carrier, heavy combat boots, tactical headset.

### Prompt:
```text
Full-body 3D character design of Per, a male heavy tactical enforcer in a strict symmetrical T-pose, viewed straight-on at eye level. Broad-shouldered heavy muscular powerhouse build, early 30s, dark buzz-cut hair, rugged square chiseled jawline with heavy 5 o'clock shadow stubble. Wearing a crimson red heavy tactical jumpsuit, heavy-duty dark charcoal reinforced tactical plate carrier with front ammo pouches and shoulder pads, thick tactical utility belt, heavy-duty black knee guards, and heavy reinforced black combat boots. Wearing a rugged tactical headset with boom microphone. Symmetrical frontal orthographic perspective, arms held straight out horizontally at 90 degrees with palms down, legs straight and spaced apart, feet flat facing forward. Isolated on a solid plain neutral light gray studio background with soft even lighting. Stylized 3D animated hero asset, cel-shaded comic aesthetic, distinct silhouette, no cast shadows, full figure completely visible inside frame from head to toe.
```

---

## Recommended Tripo 3D Import Settings

1. **Aspect Ratio for Generation**: Generate images at `1:1` (square) or `3:4` / `9:16` (vertical) so the full character from boots to head is framed without clipping.
2. **Preset**: Select the **Character / Rigging** preset in Tripo 3D.
3. **Format**: Export as `.glb` with embedded animations (`Idle`, `Run`), ready to replace the placeholder procedural meshes in [`src/ai.rs`](file:///C:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/src/ai.rs).
