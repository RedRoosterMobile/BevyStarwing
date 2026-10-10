# Panther recording script

A calm, slightly cocky wingmate; urgent on damage, genuinely delighted on victory. Speak clearly, with short radio-call pacing. No talking/mouth animation is enabled; expressions still react to gameplay.

Record one line per file, with exactly these lowercase names:

| Filename | Trigger | Delivery | Line |
| --- | --- | --- | --- |
| `launch.wav` | Start / restart | Quiet confidence | “Keep your nose up. We're going in.” |
| `kill.wav` | Fighter destroyed | Quick encouragement | “Clean hit! Keep it up.” |
| `roll.wav` | Barrel roll | Playful approval | “Nice roll. Stay slippery.” |
| `hit.wav` | Shield damage, above two shields | Startled warning | “Whoa! Watch the crossfire!” |
| `low_shields.wav` | Damage leaving one or two shields | Urgent, controlled | “Shields critical! Fly careful!” |
| `victory.wav` | Survive the 90-second mission | Big relief, delighted | “Sector clear! That's how we fly!” |
| `defeat.wav` | Ship destroyed | Concerned, reassuring | “We've lost the ship. Let's try again.” |

## Export / install

- WAV, uncompressed signed 16-bit PCM, mono, 44.1 or 48 kHz.
- Aim for 1–3 seconds per line. Trim dead air, leave ~50 ms at each end. No clipping; peak around -3 dBFS.
- Record clean/dry. CRT scanlines are visual only; no radio filter is applied to audio by the game.
- Put recordings alongside this README in `assets/voice/panther/`, then rebuild (`cargo run` or `./build-web.sh`). `build.rs` embeds whichever named WAVs exist, so missing recordings are silent, not errors. Web releases remain self-contained.
- Text captions appear now, even without recordings. Voice cues have a four-second cooldown; damage/critical/results can interrupt less important lines. One voice plays at a time. Pause freezes voices and reactions; restart stops the old voice and replays launch if present.
- The kill/roll face lasts 1.6 seconds; hurt lasts 2.6 seconds. Results hold until restart. Expressions are independent of clip duration, and the talk morph always stays zero.
