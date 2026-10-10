# Game plan: three authored levels and readable obstacle encounters

## Purpose

Implement this plan one session at a time, including sessions that start with an empty conversation context. Work through the numbered sessions in order. Each session should leave a playable game, tests, and an updated checklist.

This is a plan, not an implementation. All boxes start unchecked.

## Instructions for a fresh session

1. Read this file, `README.md`, and the current code before editing. Run `git status --short`; preserve unrelated work.
2. Pick the first incomplete session. Implement only that session and its acceptance criteria. Earlier sessions are dependencies; later sessions are not extra scope.
3. Follow the current code if it has changed since this plan was written. Record any necessary plan adjustment here rather than silently changing the design.
4. Run the session's tests and the shared checks below. Only check off work supported by evidence; record any unavailable checks.
5. Append a short handoff entry with changed paths, checks/results, and the next task. Do not depend on chat history.

## Current code baseline

- Bevy rail shooter; most gameplay is in `src/main.rs`.
- World objects move toward the player at Z=0. `SPEED` is 24, increasing by `flight.elapsed * 0.08`.
- Player center ranges: X = -6.2 to 6.2, Y = -3.3 to 4.2; collision radius is 0.55.
- `waves()` generates three fighters on two of every three waves. Every third wave alternates between three random rocks and two full-height pillars with a sideways gap.
- `travel()` scrolls obstacles without animating them. `Obstacle { half }` supplies a fixed axis-aligned box.
- `combat()` uses box/sphere player collisions and swept laser/box tests. Obstacles block lasers and cannot be destroyed.
- One 90-second mission (`MISSION`), five shields, pause, restart, victory/defeat UI, sound, portrait reactions, and native/web builds already exist.
- `src/scenery.rs` supplies decorative scenery; decorations must not silently become collidable.

Recheck these facts against the workspace before implementing.

## Design boundaries

- At least three authored levels, played easy to hard. Introduce mechanics alone before combining them.
- No rotating collision boxes. No falling/tipping towers, swinging beams, rotating gates, or leaning structures.
- Solid hazards only translate along X/Y while scrolling along Z; their collision boxes stay axis-aligned.
- Walking robot limbs may animate cosmetically, but the body's gameplay collision stays axis-aligned.
- Prefer existing meshes, shared materials, sounds, projectile code, and collision helpers. No physics engine or generic level-editor framework.
- Every encounter has a reachable escape route with clearance for the ship radius. Never fill the entire playable cross-section.
- Telegraph movement with visible geometry/light/color, not color alone. Avoid surprise hazards appearing at the player.
- Moving obstacles remain indestructible. Robots are shootable enemies; decorations are not hazards.
- Pause freezes encounter timers and motion. Restart clears spawned objects and resets the current level deterministically.
- Keep native and WebGL2 behavior aligned. Embed level files so FTP deployment needs no extra asset-fetch setup.

## Level creation approach

Use three small JSON files under `assets/levels/`, embedded with `include_str!`. JSON is the source of truth; do not maintain a second ASCII format.

Each file should contain a version, ID, name, duration, starting speed, optional speed increase, and an ordered list of encounters. Each encounter has an `at` time in seconds, a named pattern, and only the parameters that implemented patterns actually need. Use a typed enum rather than arbitrary strings passed into gameplay logic. Add pattern variants as sessions implement them.

Illustrative shape (not a finalized schema):

```json
{
  "version": 1,
  "id": "01-outskirts",
  "name": "Outskirts",
  "duration": 90,
  "speed": 24,
  "speed_ramp": 0,
  "encounters": [
    { "at": 4, "pattern": "fighters", "count": 1, "x": 0, "y": 1 },
    { "at": 12, "pattern": "pillar_gap", "gap_x": -2, "gap_width": 4 }
  ]
}
```

`at` means **spawn time**, not time of contact. Spawn at a consistent forward distance. Author spacing with approach time in mind; at roughly Z=-100 and speed 24, contact is about four seconds later. Do not complete a mission while its final encounter is still approaching.

A shared encounter dispatcher replaces procedural `waves()` scheduling. Keep a cursor into the ordered encounter list; dispatch every due event exactly once, including when one update passes multiple event times. Use the existing deterministic RNG only for deliberately bounded variation, never to decide whether a route exists.

Validation should reject unsupported versions/patterns, non-finite or invalid values, out-of-order times, events beyond level duration, and invalid dimensions/gaps. Do not silently substitute a random level for bad data. Reuse an existing JSON dependency if suitable; otherwise add only `serde`/`serde_json` as needed.

## Ordered implementation sessions: easy to hard

Foundational scheduling comes before the harder gameplay changes that depend on it. Keep each session's scope bounded.

### Session 1 — Static colored building encounters

- [ ] Add a small shared palette for static building blocks; keep collision dimensions consistent with the visible geometry.
- [ ] Add reusable spawn functions for a low block, tall roadside building, overhead bridge, and stepped cluster, made from axis-aligned cuboids.
- [ ] Arrange them as deliberate passages, not randomly overlapping boxes. Add a few examples to the current mission without increasing enemy density.
- [ ] Keep static building colors visually distinct from future moving machinery.
- [ ] Add focused geometry/collision tests for an open passage and a blocked position.
- [ ] Playtest: visible gaps are actually flyable, buildings block lasers, and pause/restart still work.

**Done when:** the current mission has readable, varied static building encounters without new movement or collision primitives.

### Session 2 — File-driven encounters and a genuinely easy first level

- [ ] Add `src/level.rs` (or an equally small module following existing conventions), the typed JSON model, parser, and validator.
- [ ] Add embedded `assets/levels/01-outskirts.json`. Support only fighters, rocks, pillar gaps, and Session 1 building patterns for now.
- [ ] Replace random wave timing with the authored encounter cursor. Move duration/speed settings out of the global one-mission assumptions.
- [ ] Author Level 1: 90 seconds, constant speed initially, isolated one/two-fighter encounters, generous gaps, sparse rocks/buildings, and recovery stretches.
- [ ] Start with approximately 6–8 encounters, at least 8 seconds apart, then tune from playtesting. No moving hazards and no simultaneous obstacle/enemy introduction.
- [ ] Leave a quiet final stretch long enough for the last encounter to pass the player.
- [ ] Test parsing, bad-file rejection, once-only dispatch, multiple due events, pause, restart cursor reset, and end-of-level timing.
- [ ] Document the actual schema and add/edit workflow in `README.md`, with one working encounter example.

**Done when:** changing the embedded JSON and rebuilding changes the easy level without editing spawning code. The procedural wave scheduler is no longer generating extra encounters behind it.

### Session 3 — Three-level progression using existing mechanics

- [ ] Add embedded `02-industrial.json` and `03-core.json`, initially using only completed patterns.
- [ ] Track the current level independently of its elapsed time and encounter cursor.
- [ ] Display level name, number, and correct per-level progress in the HUD.
- [ ] On success, show a level-complete screen with an explicit continue action; advance once, never during an unresolved collision frame.
- [ ] Start each new level with five shields. Carry score across successful levels; R retries the current level and restores its entry score. Defeat retries the current level, not Level 1.
- [ ] After Level 3, show campaign victory and an explicit action to start a new campaign with score reset. Keep R's meaning clear on all result screens.
- [ ] Clear transient enemies, obstacles, bolts, particles, damage feedback, and level timers on transition. Preserve intentional music behavior and keep portrait victory/defeat reactions correct.
- [ ] Author Level 2 with moderately denser existing patterns; Level 3 with harder but reachable combinations. Do not simply multiply enemy counts or speed.
- [ ] Test advance, retry score restoration, final victory, campaign reset, and no objects leaking between levels.

**Done when:** all three levels can be completed in order, retried independently, and restarted as a campaign. Later sessions enrich Levels 2/3 rather than create the campaign from scratch.

### Session 4 — Rising ground barricades: sideways dodging

- [ ] Add the smallest motion state needed for a warning, rise, hold, and retract cycle; use encounter-local time or approach distance, not global sine phases.
- [ ] Spawn a visible ground socket/base before a barricade rises. Use translation only; the visible solid and its hitbox must agree during the full motion.
- [ ] Author alternating left/right occupied lanes with an open sideways route. Make raised blocks tall enough that flying over is not the universal solution.
- [ ] Configure activation by approach distance so changing level speed preserves useful warning time; specify and tune a minimum visible warning.
- [ ] Add an isolated introduction in Level 2, followed by a spaced left/right sequence. Add a tighter sequence to Level 3 only after testing.
- [ ] Test motion boundaries, moving hitbox positions, pause/restart, and nonzero safe-lane clearance.

**Done when:** barricades visibly warn, rise, and require a fair sideways dodge; they never spawn invisible blocking collision in the flight path.

### Session 5 — Vertical gates: high/low timing

- [ ] Add translating floor/ceiling slabs with warning, closing, opening, and hold phases, reusing Session 4 motion logic where it genuinely fits.
- [ ] Design a changing vertical opening: first high, then low. Leave a ship-sized escape route at every relevant contact phase; avoid a fully sealed wall spanning all X.
- [ ] Synchronize each pair from one encounter clock. Stagger successive gates to make timing readable rather than arbitrary.
- [ ] Introduce one slow gate in Level 2 before an alternating high/low sequence. Use faster or combined sequences in Level 3, not Level 1.
- [ ] Test top/bottom collider alignment, opening clearance, synchronized phases, and reachable movement between consecutive gates.
- [ ] Playtest approach cues and depth readability from the actual flight camera.

**Done when:** players can read and time a high/low route without relying on barrel-roll immunity or guessing invisible phase changes.

### Session 6 — Sideways sliding shutters

- [ ] Add paired left/right translating walls whose opening moves laterally. Reuse existing gate pieces, palette, and timing rather than adding a separate machinery framework.
- [ ] Keep an adequately wide opening throughout the encounter; use tall walls so this is a lateral tracking challenge.
- [ ] Introduce a slow shutter encounter in Level 2; add changing openings in Level 3.
- [ ] Test minimum gap width, motion limits, collider alignment, and traversal time between consecutive openings.
- [ ] Playtest shutters adjacent to vertical gates; remove combinations that hide the next required route.

**Done when:** moving openings are readable, trackable, and distinct from static pillar gaps.

### Session 7 — Walking ground robots that shoot

- [ ] Build a low-poly robot from existing primitive meshes/materials with cosmetic walking animation and an axis-aligned body hitbox.
- [ ] Give it a ground lane, left-to-right/right-to-left patrol, a short firing stop, aimed shots, and resume behavior. Reuse hostile bolts and sound.
- [ ] Separate its behavior from airborne fighter steering; do not attach both motion systems to one entity.
- [ ] Make it shootable for a documented score value; reuse explosion and hit feedback. Extend laser target handling without losing nearest-hit ordering or obstacle occlusion.
- [ ] Add an isolated robot introduction in Level 2. Add robots beside known building/barricade patterns in Level 3, with a clear dodge route.
- [ ] Test patrol bounds, shooting window, pause/reset, destruction/scoring once, nearest-hit ordering, and robots behind walls not taking blocked shots.

**Done when:** ground robots visibly walk, pause to fire, and can be destroyed while all existing fighter combat still works.

### Session 8 — Final authored difficulty and campaign playtest

- [ ] Rebalance all JSON files after the new mechanics exist: Level 1 remains easy; Level 2 teaches; Level 3 combines and challenges.
- [ ] Level 1 / Outskirts: sparse fighters, colored static buildings, wide pillar gaps, no moving machinery or robots.
- [ ] Level 2 / Industrial: isolated introductions to rising barricades, vertical gates, shutters, and robots, separated by recovery stretches.
- [ ] Level 3 / Core: alternating sideways/vertical routes, robot fire beside familiar machinery, and denser static passages. Avoid simultaneous introductions or unavoidable bullet/obstacle traps.
- [ ] Confirm the first warning for each mechanic is visible from the camera, and the next route is not hidden behind the current obstacle.
- [ ] Check actual combined encounters for reachable escape routes; per-pattern validation alone cannot prove a whole level fair.
- [ ] Play through all levels natively and in the browser, including pause during motion, retry after death, and campaign restart.
- [ ] Update `README.md` with progression, new hazards, result-screen controls, and level-authoring instructions.

**Done when:** three distinct, fair levels offer increasing variety and difficulty without requiring rotating collision boxes.

## Shared completion checks for each session

Run and record:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo check --target wasm32-unknown-unknown --features web
```

For gameplay changes, also record a native playtest. For changes to embedded data or rendering, build/playtest the browser version when possible. Compile success is not evidence of visual readability or fair difficulty. If an environment/toolchain prevents a check, leave that acceptance item unchecked and record the blocker.

## Handoff log

Append one entry per implementation session:

```text
Session/date:
Completed checkboxes:
Changed paths:
Tests and playtests (actual results):
Unchecked items/blockers:
Design/schema decisions made:
Next session/task:
```
