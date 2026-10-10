use bevy::{audio::Volume, prelude::*};
use xmrs::core::module::Module;
use xmrsplayer::xmrsplayer::XmrsPlayer;

use super::sound::{SAMPLE_RATE, wav};

const TRACK: &[u8] = include_bytes!("../assets/music/ace_fi_-_space_walk.mod");

#[derive(Component)]
pub struct Music;

pub fn setup(mut commands: Commands, mut assets: ResMut<Assets<AudioSource>>) {
    match render(TRACK) {
        Ok(source) => {
            info!(
                "Space Walk: {:.1}s of ProTracker music ready",
                (source.bytes.len() - 44) as f32 / (SAMPLE_RATE * 4) as f32
            );
            commands.spawn((
                Music,
                AudioPlayer::new(assets.add(source)),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(0.25)),
            ));
        }
        Err(error) => warn!("Music unavailable: {error}"),
    }
}

pub fn pause(flight: Res<super::Flight>, sinks: Query<&AudioSink, With<Music>>) {
    for sink in &sinks {
        if flight.paused {
            sink.pause();
        } else {
            sink.play();
        }
    }
}

// Render once, then let Bevy loop the PCM. The small original MOD stays embedded;
// no system tracker library, converter, or large pre-rendered asset is needed.
fn render(bytes: &[u8]) -> Result<AudioSource, String> {
    let module = Module::load(bytes).map_err(|error| format!("Invalid MOD: {error:?}"))?;
    let mut player = XmrsPlayer::new(&module, SAMPLE_RATE, 0);
    player.set_max_loop_count(1);
    let mut samples = Vec::new();
    // Bound memory/time if the embedded track is ever replaced with a looping/broken module.
    for _ in 0..SAMPLE_RATE * 600 {
        let Some((left, right)) = player.sample(true) else {
            if samples.is_empty() {
                return Err("The MOD contains no audio".into());
            }
            return Ok(AudioSource {
                bytes: wav(&samples, 2).into(),
            });
        };
        samples.extend_from_slice(&[left, right]);
    }
    Err("MOD exceeded the ten-minute render limit".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::audio::{Decodable, PlaybackMode};

    #[test]
    fn supplied_protracker_mod_renders_full_stereo_track_and_loops() {
        assert_eq!(&TRACK[1080..1084], b"M.K.");
        let mut app = App::new();
        app.insert_resource(Assets::<AudioSource>::default())
            .add_systems(Startup, setup);
        app.update();
        let (player, settings) = app
            .world_mut()
            .query_filtered::<(&AudioPlayer, &PlaybackSettings), With<Music>>()
            .single(app.world())
            .unwrap();
        assert!(matches!(settings.mode, PlaybackMode::Loop));
        assert_eq!(settings.volume, Volume::Linear(0.25));
        let source = app
            .world()
            .resource::<Assets<AudioSource>>()
            .get(&player.0)
            .unwrap();
        assert_eq!(
            u16::from_le_bytes(source.bytes[22..24].try_into().unwrap()),
            2
        );
        let duration = (source.bytes.len() - 44) as f32 / (SAMPLE_RATE * 4) as f32;
        assert!(duration > 90.0 && duration < 600.0, "duration={duration}");
        assert!(
            source
                .decoder()
                .take(SAMPLE_RATE as usize * 4)
                .any(|sample| sample != 0.0)
        );
        assert!(
            app.world_mut()
                .query_filtered::<Entity, (With<Music>, With<super::super::Transient>)>()
                .iter(app.world())
                .next()
                .is_none(),
            "restarting must not remove the music"
        );
    }

    #[test]
    fn invalid_module_is_rejected() {
        assert!(render(b"not a tracker module").is_err());
    }
}
