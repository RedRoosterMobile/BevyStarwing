# Panther monitor character

Stylized humanized panther head with nine flat-color PBR materials, no textures, and no exported lights/cameras. Designed for a front-facing monitor portrait; apply CRT effects in the game, not in this asset.

## Files

- `panther_neutral.glb`
- `panther_smile.glb`
- `panther_celebrate.glb`
- `panther_fear_scream_hurt.glb` — one shared alarm/pain expression
- `panther_talk.glb` — static open-mouth pose
- `panther_morphs.glb` — neutral base, four morph targets, and a one-second `Talk` animation (24 fps source, loop in the game)

The five pose files are baked standalone meshes. The morph file is the preferred option for smooth reactions and speech. All versions share topology/materials. Approx. 12,241 source vertices, nine material primitives; ~424 KiB per pose and ~1.29 MiB for the morph version.

## Morph weights

Target order (on every material primitive):

0. `smile`
1. `celebrate`
2. `fear_scream_hurt`
3. `talk`

Neutral is `[0, 0, 0, 0]`. Drive the selected target from 0 to 1, easing the previous target back to 0. For speech, oscillate target 3, or loop the exported `Talk` clip. Stop the talk clip when driving other expressions so it does not overwrite manual weights. Strong simultaneous weights may overlap facial components; use single-expression blends rather than setting everything to 1.

In Bevy, load the GLB's `Scene0` (not just its first material mesh). When animating manually, update the scene's morph weights across all mesh primitives. Orientation after glTF conversion: +Y up, face toward +Z. Approximate bounds: X ±1.2, Y -1.6…1.7, Z -0.8…1.45. Camera in front on +Z, aiming near Y = 0.1. Orthographic vertical span ~3.7 frames the full head/collar. A small amount of monitor lighting makes the dark fur readable; materials are not unlit/emissive.

## Editable source

- `blender/panther_character.blend`: neutral default, editable shape keys, Talk action, and `Panther_Presentation` scene with preview lighting/camera. Original default scene is preserved.
- `blender/build_panther.py`: additive MCP build script. Run only in a file without `Panther_Character`; the guard prevents accidental duplication.
- `blender/panther_*_preview.png`: rendered expression previews.

Verified GLB headers, texture absence, distinct baked poses, target order, and exported morph animation. Visually checked Blender renders. The game now uses `src/monitor.rs` to render the morph model into a bottom-left CRT monitor with reaction weights and captions. Talking is disabled. Native render-to-texture loading was visually verified. Chromium/WebGL2 browser startup, gameplay, reactions, fire/roll, pause/resume and restart were smoke-tested without render validation errors. The monitor uses a direct sRGB render target (no alternate views) and point lights for WebGL2 compatibility. Optional recording filenames live in `assets/voice/panther/README.md`.
