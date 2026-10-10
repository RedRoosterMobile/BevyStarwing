# Starwing

A tiny Star Fox (SNES)-inspired rail shooter, made with Bevy 0.20.0 and original procedural geometry. No Nintendo assets, external models, or textures.

## Run

```sh
cd starwing # from the parent bevy directory
cargo run
```

`rust-toolchain.toml` selects Rust 1.99.0 (Bevy requires at least 1.97.1) and includes the WebAssembly target. Rustup installs this toolchain automatically without changing your global default. The first build downloads and compiles Bevy; expect several minutes. Native builds use Metal (macOS), Vulkan, or DirectX 12.

## Neon Sprawl

Fly down a cyan/magenta grid highway between scrolling triangulated wireframe mountains, under a striped orange-to-pink sun, violet sunset gradient, and stars. Obstacles stay inside the playable lane; decorative mountains remain outside it. The scenery uses shared unlit vertex-color materials and ordinary triangle ribbons, not native polygon-mode wireframe, custom shaders, or desktop-only post-processing.

For a WebGL2-compatible compile check:

```sh
rustup target add wasm32-unknown-unknown
cargo check --target wasm32-unknown-unknown --features web
```

## Web build / FTP upload

```sh
./build-web.sh
```

This produces `dist/` and `starwing-web.zip`. Extract the ZIP locally and upload **its contents** (`index.html`, `starwing.js`, `starwing_bg.wasm`) by FTP into any public folder, for example `public_html/games/starwing/`. Open `https://your-domain.example/games/starwing/` and click **Play Starwing**. All URLs are relative, so no domain or subfolder configuration is needed. The game assets and music are embedded in WASM.

Use a modern desktop browser with WebGL2 and a keyboard. Serve over HTTP(S), not by opening `index.html` directly from disk. No server-side code, routing rules, cross-origin isolation headers, or WASM MIME configuration is required. Browser sound may need a click after loading. Upload the three files together when updating (and refresh the browser cache). The script installs the matching `wasm-bindgen` CLI if needed; the initial release build takes several minutes. Web builds use a separate size-optimized profile with link-time optimization and omit debugging names. If Binaryen is installed (`brew install binaryen` on macOS), the script also runs `wasm-opt -Os`. Native build settings are unchanged. Enable gzip/Brotli on your web host if available to reduce download sizes further; the ZIP is only an upload archive.

## Controls

| Key | Action |
| --- | --- |
| WASD / arrows | Steer |
| Hold Space | Twin lasers |
| Left / right Shift | Barrel roll: brief immunity, 3-second cooldown |
| P | Pause / resume |
| R | Restart anytime |
| Esc | Quit |

Survive the 90-second canyon run. Shoot red fighters for 100 points each; avoid their aimed shots, floating rocks, and walls with narrow openings. Five shield hits end the run. Damage briefly makes you invulnerable (the ship flashes). Obstacles cannot be destroyed. Chromatic aberration is always visible, with exaggerated color splitting on hits. Hits briefly shake the camera: laser impacts are subtle, shield damage is stronger, and crashes/destruction give the largest jolt. Hit effects fade back to the resting camera and baseline aberration, freeze while paused, and reset on restart; immune hits do not trigger damage feedback. The course gets faster and waves more frequent as you progress.

## Soundtrack

The supplied ProTracker MOD **Space Walk** is preserved at `assets/music/ace_fi_-_space_walk.mod` and embedded in the executable. `xmrs` / `xmrsplayer` import its instruments, patterns, and effects and render one complete pass to stereo PCM in memory at startup. Bevy loops that audio at 25% volume beneath the sound effects. No converter, system tracker library, or pre-rendered audio file is required.

P pauses/resumes the soundtrack with the game. R resets gameplay without restarting the song; music continues on the results screen. To replace it, put a valid MOD at the same path and rebuild. Rendering is limited to ten minutes to bound memory and startup time; invalid or overlong music logs a warning and leaves gameplay playable. The track was supplied by the user; verify its redistribution rights before publishing the game.

## Explore the code

Gameplay lives in `src/main.rs`:

- `setup` / `ship_mesh`: meshes, materials, lighting, camera, and UI.
- `src/scenery.rs`: procedural sunset, starfield, grid highway, and seamless looping wireframe canyon.
- Components describe player, enemies, obstacles, projectiles, and sparks.
- `Flight`: mission progress, shields, timers, and reproducible wave randomness.
- `pilot`: steering, banking, rolls, and firing.
- `waves` / `travel`: incoming formations, scrolling scenery, enemy movement and fire.
- `combat`: collisions, scoring, damage, and explosions.
- `src/explosion.rs`: destroyed fighters emit a 72-particle 3D explosion: expanding fireball and shockwave, fast orange streaks, tumbling fragments with gravity, and growing translucent smoke. Shared meshes/materials need no new dependency. Particles follow the scrolling world, freeze on pause, finish on the results screen, and clear on restart.
- `controls` / `hud`: pause, restart, exit, and UI.
- `src/impact.rs`: severity-scaled camera shake and Bevy's built-in chromatic aberration, with no custom shader or extra dependency.
- `src/sound.rs`: original [sfxr/jsfxr-style](https://sfxr.me/) procedural sound effects. Pulse waves, pitch slides, filtered noise, and attack/decay envelopes become in-memory mono PCM WAVs at startup. Effects cover player/enemy lasers, explosions, shield hits, rolls, victory, and defeat. Bevy plays each once and removes the audio entity afterward; restart also stops active effects. No downloaded samples or JavaScript runtime are needed.

- `src/music.rs`: MOD rendering, looping soundtrack, and pause/resume.

The chained Update systems execute in order. Forward flight is simulated by bringing the world toward a player at Z=0. Shared mesh/material handles keep spawning cheap. This is a small learning prototype, not a faithful recreation: no bosses, Nintendo characters, or textures. Player collisions use simple box/sphere checks; lasers use swept segments so fast shots cannot skip targets. Simulation steps are capped at 50 ms, so the game slows down on very slow machines instead of jumping forward.

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Bevy uses the 0.20 release series; `Cargo.lock` records the exact resolved dependencies. Useful starting references: [Bevy examples](https://bevy.org/examples/) and the [0.20.0 3D scene example](https://github.com/bevyengine/bevy/blob/v0.20.0/examples/3d/3d_scene.rs).
