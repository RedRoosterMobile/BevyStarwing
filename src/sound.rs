use bevy::{audio::Volume, prelude::*};
use std::f32::consts::TAU;

pub(super) const SAMPLE_RATE: u32 = 22_050;

#[derive(Message, Clone, Copy)]
#[repr(usize)]
pub enum Sound {
    Laser,
    EnemyLaser,
    Explosion,
    Hit,
    Roll,
    Victory,
    Defeat,
}

const SOUNDS: [Sound; 7] = [
    Sound::Laser,
    Sound::EnemyLaser,
    Sound::Explosion,
    Sound::Hit,
    Sound::Roll,
    Sound::Victory,
    Sound::Defeat,
];

#[derive(Resource)]
pub struct Sounds([Handle<AudioSource>; 7]);

pub fn setup(mut commands: Commands, mut assets: ResMut<Assets<AudioSource>>) {
    commands.insert_resource(Sounds(SOUNDS.map(|sound| assets.add(synthesize(sound)))));
}

pub fn playback(mut commands: Commands, sounds: Res<Sounds>, mut events: MessageReader<Sound>) {
    for &sound in events.read() {
        commands.spawn((
            super::Transient,
            AudioPlayer::new(sounds.0[sound as usize].clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.45)),
        ));
    }
}

// sfxr-style pitch slides, pulse waves, filtered noise, and a short attack/decay envelope.
// Original synthesis, not a port of jsfxr and not downloaded samples.
fn synthesize(sound: Sound) -> AudioSource {
    let (duration, start_hz, end_hz, gain): (f32, f32, f32, f32) = match sound {
        Sound::Laser => (0.10, 1400.0, 180.0, 0.22),
        Sound::EnemyLaser => (0.16, 650.0, 100.0, 0.16),
        Sound::Explosion => (0.55, 1700.0, 60.0, 0.65),
        Sound::Hit => (0.24, 240.0, 55.0, 0.5),
        Sound::Roll => (0.38, 220.0, 1200.0, 0.3),
        Sound::Victory => (0.64, 523.25, 1046.5, 0.3),
        Sound::Defeat => (0.75, 360.0, 45.0, 0.5),
    };
    let count = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut phase = 0.0_f32;
    let mut rng = 0x12345678_u32;
    let mut filtered_noise = 0.0;
    for i in 0..count {
        let t = i as f32 / SAMPLE_RATE as f32;
        let progress = i as f32 / (count - 1) as f32;
        let mut hz = start_hz * (end_hz / start_hz).powf(progress);
        let mut envelope = (t / 0.004).min(1.0) * (1.0 - progress).powf(1.5);
        if matches!(sound, Sound::Victory) {
            let note = (progress * 4.0).min(3.0) as usize;
            hz = [523.25, 659.25, 783.99, 1046.5][note];
            let note_time = t % 0.16;
            envelope = (note_time / 0.004).min(1.0) * ((0.16 - note_time) / 0.04).min(1.0) * 0.8;
        }
        if matches!(sound, Sound::Roll) {
            hz *= 1.0 + 0.12 * (t * TAU * 18.0).sin();
        }
        phase = (phase + hz / SAMPLE_RATE as f32).fract();
        let pulse = if phase < 0.5 { 1.0 } else { -1.0 };
        let triangle = 1.0 - 4.0 * (phase - 0.5).abs();
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = (rng >> 8) as f32 / 8388608.0 - 1.0;
        let cutoff = (TAU * hz / SAMPLE_RATE as f32).min(1.0);
        filtered_noise += cutoff * (noise - filtered_noise);
        let wave = match sound {
            Sound::Explosion => filtered_noise,
            Sound::Hit | Sound::Defeat => pulse * 0.45 + filtered_noise * 0.55,
            Sound::Roll | Sound::Victory => triangle,
            _ => pulse,
        };
        // The final 4 ms fade also prevents clicks at the end of the victory fanfare.
        envelope *= ((duration - t) / 0.004).clamp(0.0, 1.0);
        samples.push((wave * envelope * gain * i16::MAX as f32) as i16);
    }
    AudioSource {
        bytes: wav(&samples, 1).into(),
    }
}

pub(super) fn wav(samples: &[i16], channels: u16) -> Vec<u8> {
    let size = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + size as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * u32::from(channels) * 2).to_le_bytes());
    bytes.extend_from_slice(&(channels * 2).to_le_bytes()); // block alignment
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&size.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::audio::{Decodable, PlaybackMode};

    #[test]
    fn all_effects_decode_and_have_safe_levels_and_fades() {
        for sound in SOUNDS {
            let source = synthesize(sound);
            let decoded: Vec<_> = source.decoder().collect();
            assert_eq!(decoded.len(), (source.bytes.len() - 44) / 2);
            assert!(decoded.len() >= 2205);
            let peak = decoded
                .iter()
                .map(|&sample| (sample as f32).abs())
                .fold(0.0, f32::max);
            assert!(peak > 100.0 && peak < i16::MAX as f32 * 0.7);
            assert!((decoded[0] as f32).abs() < 100.0);
            assert!((*decoded.last().unwrap() as f32).abs() < 100.0);
        }
    }

    #[test]
    fn effects_spawn_once_and_clean_up_after_playback() {
        let mut app = App::new();
        app.insert_resource(Assets::<AudioSource>::default())
            .add_message::<Sound>()
            .add_systems(Startup, setup)
            .add_systems(Update, playback);
        app.update();
        app.world_mut().write_message(Sound::Laser);
        app.world_mut().write_message(Sound::Explosion);
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&AudioPlayer>()
                .iter(app.world())
                .count(),
            2
        );
        for settings in app
            .world_mut()
            .query::<&PlaybackSettings>()
            .iter(app.world())
        {
            assert!(matches!(settings.mode, PlaybackMode::Despawn));
        }
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&AudioPlayer>()
                .iter(app.world())
                .count(),
            2
        );
    }
}
