//! A layer-isolated 3D portrait, rendered into a bottom-left CRT UI panel.
use bevy::{
    asset::io::embedded::EmbeddedAssetRegistry,
    audio::Volume,
    camera::{RenderTarget, ScalingMode, visibility::RenderLayers},
    mesh::morph::MorphWeights,
    prelude::*,
    render::render_resource::TextureFormat,
    world_serialization::WorldInstanceReady,
};
use std::path::Path;

use crate::{Flight, combat, controls, setup as game_setup, sound::Sound};

const SCREEN: f32 = 208.0;
const PORTRAIT: &str = "embedded://panther/panther_morphs.glb";

pub struct MonitorPlugin;
impl Plugin for MonitorPlugin {
    fn build(&self, app: &mut App) {
        // Keep the portrait self-contained in both native and FTP/WASM builds.
        app.world()
            .resource::<EmbeddedAssetRegistry>()
            .insert_asset(
                Path::new(file!()).to_path_buf(),
                Path::new("panther/panther_morphs.glb"),
                &include_bytes!("../assets/characters/panther/panther_morphs.glb")[..],
            );
        app.init_resource::<Reaction>()
            .add_message::<Cue>()
            .add_systems(Startup, (setup, setup_voices).after(game_setup))
            .add_systems(
                Update,
                (react, play_voice, animate)
                    .chain()
                    .after(combat)
                    .after(controls),
            );
    }
}

#[derive(Component)]
struct Portrait;
#[derive(Component)]
struct Caption;
#[derive(Component)]
struct Sweep;

#[derive(Message, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(usize)]
enum Cue {
    #[default]
    Launch,
    Kill,
    Roll,
    Hit,
    LowShields,
    Victory,
    Defeat,
}
impl Cue {
    fn line(self) -> &'static str {
        match self {
            Self::Launch => "Keep your nose up. We're going in.",
            Self::Kill => "Clean hit! Keep it up.",
            Self::Roll => "Nice roll. Stay slippery.",
            Self::Hit => "Whoa! Watch the crossfire!",
            Self::LowShields => "Shields critical! Fly careful!",
            Self::Victory => "Sector clear! That's how we fly!",
            Self::Defeat => "We've lost the ship. Let's try again.",
        }
    }
    fn target(self) -> Option<usize> {
        match self {
            Self::Launch => None,
            Self::Kill | Self::Roll => Some(0),
            Self::Victory => Some(1),
            Self::Hit | Self::LowShields | Self::Defeat => Some(2),
        }
    }
    fn priority(self) -> u8 {
        match self {
            Self::Launch => 0,
            Self::Kill | Self::Roll => 1,
            Self::Hit => 2,
            Self::LowShields => 3,
            Self::Victory | Self::Defeat => 4,
        }
    }
}

#[derive(Resource)]
struct Reaction {
    cue: Cue,
    remaining: f32,
    weights: [f32; 4],
    elapsed: f32,
}
impl Default for Reaction {
    fn default() -> Self {
        Self {
            cue: Cue::Launch,
            remaining: 3.0,
            weights: [0.0; 4],
            elapsed: 0.0,
        }
    }
}
impl Reaction {
    fn trigger(&mut self, cue: Cue) {
        if self.remaining > 0.0 && cue.priority() < self.cue.priority() {
            return;
        }
        self.cue = cue;
        self.remaining = match cue {
            Cue::Launch => 3.0,
            Cue::Kill | Cue::Roll => 1.6,
            Cue::Hit | Cue::LowShields => 2.6,
            Cue::Victory | Cue::Defeat => f32::INFINITY,
        };
    }
    fn advance(&mut self, dt: f32) {
        self.remaining = (self.remaining - dt).max(0.0);
        let target = (self.remaining > 0.0).then(|| self.cue.target()).flatten();
        let ease = 1.0 - (-10.0 * dt).exp();
        for (i, weight) in self.weights.iter_mut().enumerate() {
            let goal = (target == Some(i)) as u8 as f32;
            *weight += (goal - *weight) * ease;
        }
        self.weights[3] = 0.0; // Talking is intentionally disabled.
    }
}

fn portrait_image() -> Image {
    // WebGL2 cannot reinterpret a texture through an alternate sRGB view.
    Image::new_target_texture(256, 256, TextureFormat::Rgba8UnormSrgb, None)
}

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    main_camera: Query<Entity, With<IsDefaultUiCamera>>,
) {
    let image = images.add(portrait_image());
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            clear_color: Color::srgb(0.012, 0.035, 0.043).into(),
            ..default()
        },
        RenderTarget::Image(image.clone().into()),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 3.65,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.1, 0.25, 7.0).looking_at(Vec3::new(0.0, 0.08, 0.0), Vec3::Y),
        RenderLayers::layer(1),
        Msaa::Off,
    ));
    commands
        .spawn((
            Portrait,
            WorldAssetRoot(server.load(GltfAssetLabel::Scene(0).from_asset(PORTRAIT))),
            Transform::default(),
        ))
        .observe(portrait_ready);
    // WebGL2 allows just one directional light globally (used by the flight scene).
    for (position, intensity, color) in [
        (
            Vec3::new(-3.0, 4.0, 4.0),
            1_800_000.0,
            Color::srgb(0.70, 0.90, 1.0),
        ),
        (
            Vec3::new(3.0, 1.0, 3.0),
            1_200_000.0,
            Color::srgb(1.0, 0.78, 0.55),
        ),
    ] {
        commands.spawn((
            PointLight {
                intensity,
                color,
                range: 20.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y),
            RenderLayers::layer(1),
        ));
    }
    let Ok(camera) = main_camera.single() else {
        return;
    };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(22),
                bottom: px(18),
                width: px(228),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(2)),
                row_gap: px(6),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.045, 0.07, 0.08)),
            BorderColor::all(Color::srgb(0.22, 0.40, 0.39)),
            GlobalZIndex(10),
            UiTargetCamera(camera),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("*  PANTHER / COMMS 01"),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgb(0.35, 1.0, 0.72)),
            ));
            panel
                .spawn((
                    ImageNode::new(image),
                    Node {
                        width: px(SCREEN),
                        height: px(SCREEN),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                ))
                .with_children(|screen| {
                    // Plain UI strips are WebGL2-safe and stay on top of the live 3D image.
                    for y in (0..SCREEN as u32).step_by(4) {
                        screen.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: px(y),
                                left: px(0),
                                width: percent(100),
                                height: px(1),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.38)),
                        ));
                    }
                    screen.spawn((
                        Sweep,
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(0),
                            left: px(0),
                            width: percent(100),
                            height: px(10),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 1.0, 0.7, 0.07)),
                    ));
                    screen.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: percent(100),
                            height: percent(100),
                            border: UiRect::all(px(3)),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.0, 0.025, 0.025, 0.65)),
                    ));
                });
            panel.spawn((
                Caption,
                Text::new(Cue::Launch.line()),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.65, 1.0, 0.83)),
                Node {
                    min_height: px(40),
                    ..default()
                },
            ));
        });
}

fn portrait_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    roots: Query<(), With<Portrait>>,
    children: Query<&Children>,
) {
    if !roots.contains(ready.entity) {
        return;
    }
    // RenderLayers aren't inherited: every glTF descendant must be isolated.
    commands.entity(ready.entity).insert(RenderLayers::layer(1));
    for entity in children.iter_descendants(ready.entity) {
        commands
            .entity(entity)
            .insert((Portrait, RenderLayers::layer(1)));
    }
}

fn react(
    time: Res<Time>,
    flight: Res<Flight>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sounds: MessageReader<Sound>,
    mut reaction: ResMut<Reaction>,
    mut voices: MessageWriter<Cue>,
) {
    let restarted = keys.just_pressed(KeyCode::KeyR) || flight.elapsed < reaction.elapsed;
    if restarted {
        *reaction = Reaction::default();
        sounds.clear();
        voices.write(Cue::Launch);
    }
    reaction.elapsed = flight.elapsed;
    // Consume events even while paused; don't replay stale reactions on resume.
    for sound in sounds.read() {
        if flight.paused {
            continue;
        }
        let cue = match sound {
            Sound::Explosion => Some(Cue::Kill),
            Sound::Roll => Some(Cue::Roll),
            Sound::Hit if flight.health <= 2 => Some(Cue::LowShields),
            Sound::Hit => Some(Cue::Hit),
            Sound::Victory => Some(Cue::Victory),
            Sound::Defeat => Some(Cue::Defeat),
            _ => None,
        };
        if let Some(cue) = cue
            && (reaction.remaining == 0.0 || cue.priority() >= reaction.cue.priority())
        {
            reaction.trigger(cue);
            voices.write(cue);
        }
    }
    if !flight.paused {
        reaction.advance(time.delta_secs());
    }
}

#[derive(Component)]
struct VoicePlayback;

#[derive(Resource, Default)]
struct Voices {
    clips: [Option<Handle<AudioSource>>; 7],
    cooldown: f32,
    priority: u8,
}

fn setup_voices(
    mut commands: Commands,
    mut assets: ResMut<Assets<AudioSource>>,
    mut events: MessageWriter<Cue>,
) {
    let bytes: [Option<&'static [u8]>; 7] =
        include!(concat!(env!("OUT_DIR"), "/panther_voices.rs"));
    commands.insert_resource(Voices {
        clips: bytes.map(|clip| {
            clip.map(|bytes| {
                assets.add(AudioSource {
                    bytes: bytes.into(),
                })
            })
        }),
        ..default()
    });
    events.write(Cue::Launch);
}

fn play_voice(
    mut commands: Commands,
    time: Res<Time>,
    flight: Res<Flight>,
    mut events: MessageReader<Cue>,
    mut voices: ResMut<Voices>,
    playing: Query<(Entity, Option<&AudioSink>), With<VoicePlayback>>,
) {
    for (_, sink) in &playing {
        if let Some(sink) = sink {
            if flight.paused {
                sink.pause();
            } else {
                sink.play();
            }
        }
    }
    let cue = events.read().copied().max_by_key(|cue| cue.priority());
    if !flight.paused {
        voices.cooldown = (voices.cooldown - time.delta_secs()).max(0.0);
    }
    let Some(cue) = cue else { return };
    if cue == Cue::Launch {
        // Restart stops any old line even if the launch clip hasn't been recorded.
        voices.cooldown = 0.0;
        for (entity, _) in &playing {
            commands.entity(entity).despawn();
        }
    }
    if voices.cooldown > 0.0 && cue.priority() <= voices.priority {
        return;
    }
    let Some(clip) = voices.clips[cue as usize].clone() else {
        return;
    };
    if cue != Cue::Launch {
        for (entity, _) in &playing {
            commands.entity(entity).despawn();
        }
    }
    commands.spawn((
        VoicePlayback,
        crate::Transient,
        AudioPlayer::new(clip),
        PlaybackSettings {
            paused: flight.paused,
            ..PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.8))
        },
    ));
    voices.cooldown = 4.0;
    voices.priority = cue.priority();
}

fn animate(
    time: Res<Time>,
    flight: Res<Flight>,
    reaction: Res<Reaction>,
    mut morphs: Query<&mut MorphWeights, With<Portrait>>,
    mut captions: Query<&mut Text, With<Caption>>,
    mut sweeps: Query<&mut Node, With<Sweep>>,
    mut sweep_phase: Local<f32>,
) {
    for mut morph in &mut morphs {
        if morph.weights().len() == 4 {
            morph.weights_mut().copy_from_slice(&reaction.weights);
        }
    }
    for mut text in &mut captions {
        let line = if flight.paused {
            "LINK ON HOLD / PAUSED"
        } else if reaction.remaining > 0.0 {
            reaction.cue.line()
        } else {
            "COMMS ONLINE / STAY SHARP"
        };
        if text.0 != line {
            text.0 = line.into();
        }
    }
    if !flight.paused {
        *sweep_phase = (*sweep_phase + time.delta_secs() * 24.0) % SCREEN;
    }
    for mut node in &mut sweeps {
        node.top = px(*sweep_phase);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn portrait_texture_does_not_require_webgl_view_formats() {
        let image = portrait_image();
        assert_eq!(
            image.texture_descriptor.format,
            TextureFormat::Rgba8UnormSrgb
        );
        assert!(image.texture_descriptor.view_formats.is_empty());
        assert!(image.texture_view_descriptor.is_none());
        assert_eq!(image.texture_descriptor.sample_count, 1);
    }

    #[test]
    fn damage_overrides_kills_and_talk_never_activates() {
        let mut r = Reaction::default();
        r.trigger(Cue::Kill);
        r.advance(0.1);
        assert!(r.weights[0] > 0.5);
        r.trigger(Cue::Hit);
        r.trigger(Cue::Kill);
        assert_eq!(r.cue, Cue::Hit);
        r.advance(1.0);
        assert!(r.weights[2] > 0.99);
        assert_eq!(r.weights[3], 0.0);
        r.advance(3.0);
        assert!(r.weights.iter().all(|w| *w < 0.001));
    }

    #[test]
    fn optional_voices_debounce_interrupt_and_clean_up_on_restart() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .init_resource::<Flight>()
            .init_resource::<Voices>()
            .add_message::<Cue>();
        app.world_mut().write_message(Cue::Kill);
        app.world_mut().run_system_once(play_voice).unwrap();
        assert_eq!(
            app.world_mut()
                .query::<&AudioPlayer>()
                .iter(app.world())
                .count(),
            0
        );
        app.world_mut().resource_mut::<Voices>().clips =
            std::array::from_fn(|_| Some(Handle::default()));
        app.world_mut().run_system_once(play_voice).unwrap();
        let first = app
            .world_mut()
            .query_filtered::<Entity, With<VoicePlayback>>()
            .single(app.world())
            .unwrap();
        app.world_mut().run_system_once(play_voice).unwrap();
        assert!(app.world().get_entity(first).is_ok());
        app.world_mut().resource_mut::<Messages<Cue>>().clear();
        app.world_mut().write_message(Cue::Hit);
        app.world_mut().run_system_once(play_voice).unwrap();
        assert!(app.world().get_entity(first).is_err());
        assert_eq!(
            app.world_mut()
                .query::<&AudioPlayer>()
                .iter(app.world())
                .count(),
            1
        );
        app.world_mut().resource_mut::<Messages<Cue>>().clear();
        app.world_mut().resource_mut::<Voices>().clips[Cue::Launch as usize] = None;
        app.world_mut().write_message(Cue::Launch);
        app.world_mut().run_system_once(play_voice).unwrap();
        assert_eq!(
            app.world_mut()
                .query::<&AudioPlayer>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn victory_and_defeat_stick_until_restart_and_pause_freezes() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(100));
        app.insert_resource(time)
            .init_resource::<Flight>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Reaction>()
            .add_message::<Sound>()
            .add_message::<Cue>();
        for sound in [Sound::Victory, Sound::Defeat] {
            *app.world_mut().resource_mut::<Reaction>() = Reaction::default();
            app.world_mut().write_message(sound);
            app.world_mut().run_system_once(react).unwrap();
            let r = app.world().resource::<Reaction>();
            assert!(r.remaining.is_infinite());
            let weights = r.weights;
            app.world_mut().resource_mut::<Flight>().paused = true;
            app.world_mut().run_system_once(react).unwrap();
            assert_eq!(app.world().resource::<Reaction>().weights, weights);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyR);
            app.world_mut().run_system_once(react).unwrap();
            assert_eq!(app.world().resource::<Reaction>().weights, [0.0; 4]);
            assert_eq!(app.world().resource::<Reaction>().cue, Cue::Launch);
            *app.world_mut().resource_mut::<ButtonInput<KeyCode>>() = ButtonInput::default();
            app.world_mut().resource_mut::<Flight>().paused = false;
        }
    }
}
