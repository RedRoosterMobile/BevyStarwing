use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use std::f32::consts::TAU;

mod explosion;
mod impact;
mod monitor;
mod music;
mod scenery;
mod sound;
use sound::Sound;

const SPEED: f32 = 24.0;
const MISSION: f32 = 90.0;
const SHIP_RADIUS: f32 = 0.55;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "STARWING — a low-poly rail shooter".into(),
                resolution: (960, 720).into(),
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#game".into()),
                #[cfg(target_arch = "wasm32")]
                fit_canvas_to_parent: true,
                #[cfg(target_arch = "wasm32")]
                prevent_default_event_handling: true,
                ..default()
            }),
            // Exit with the window intact; avoid tearing down its render surface mid-frame.
            close_when_requested: false,
            ..default()
        }))
        .add_plugins(monitor::MonitorPlugin)
        .insert_resource(ClearColor(Color::srgb(0.018, 0.004, 0.055)))
        .init_resource::<Flight>()
        .add_message::<Sound>()
        .add_systems(Startup, (setup, sound::setup, music::setup))
        .add_systems(
            Update,
            (
                controls,
                pilot,
                waves,
                travel,
                combat,
                explosion::animate,
                impact::animate,
                hud,
                sound::playback,
                music::pause,
            )
                .chain(),
        )
        .run();
}

#[derive(Resource)]
struct Flight {
    elapsed: f32,
    health: u8,
    score: u32,
    paused: bool,
    ended: bool,
    shot: f32,
    invulnerable: f32,
    roll: f32,
    roll_cooldown: f32,
    steering: Quat,
    impact: f32,
    spawn: f32,
    wave: u32,
    rng: u32,
}

impl Default for Flight {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            health: 5,
            score: 0,
            paused: false,
            ended: false,
            shot: 0.0,
            invulnerable: 0.0,
            roll: 0.0,
            roll_cooldown: 0.0,
            steering: Quat::IDENTITY,
            impact: 0.0,
            spawn: 0.4,
            wave: 0,
            rng: 12345,
        }
    }
}

impl Flight {
    fn random(&mut self, min: f32, max: f32) -> f32 {
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        min + (max - min) * (self.rng >> 8) as f32 / 16777216.0
    }
    fn running(&self) -> bool {
        !self.paused && !self.ended
    }
    fn damage(&mut self) -> bool {
        if self.invulnerable > 0.0 {
            return false;
        }
        self.health = self.health.saturating_sub(1);
        self.invulnerable = 1.3;
        self.ended = self.health == 0;
        true
    }
}

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Transient;
#[derive(Component)]
struct Scenery;
#[derive(Component)]
struct Hud;
#[derive(Component)]
struct Enemy {
    home: Vec2,
    phase: f32,
    shot: f32,
}
#[derive(Component)]
struct Obstacle {
    half: Vec3,
}
#[derive(Component)]
struct Bolt {
    velocity: Vec3,
    previous: Vec3,
    friendly: bool,
}
#[derive(Component)]
struct Spark {
    velocity: Vec3,
    life: f32,
}

#[derive(Resource)]
struct Art {
    cube: Handle<Mesh>,
    ship: Handle<Mesh>,
    rock: Handle<Mesh>,
    hull: Handle<StandardMaterial>,
    blue: Handle<StandardMaterial>,
    red: Handle<StandardMaterial>,
    stone: Handle<StandardMaterial>,
    laser: Handle<StandardMaterial>,
    hostile_laser: Handle<StandardMaterial>,
}

// Flat triangles, deliberately reminiscent of the original cartridge's tiny polygon budget.
fn ship_mesh() -> Mesh {
    let points = [
        [0.0, 0.0, -1.6],
        [-0.45, 0.0, 0.8],
        [0.45, 0.0, 0.8],
        [0.0, 0.5, 0.3],
        [-1.7, -0.12, 0.9],
        [1.7, -0.12, 0.9],
        [0.0, -0.25, 0.5],
    ];
    let triangles = [
        [0, 1, 3],
        [0, 3, 2],
        [1, 2, 3],
        [0, 4, 1],
        [0, 2, 5],
        [0, 6, 4],
        [0, 5, 6],
        [4, 6, 1],
        [5, 2, 6],
        [1, 6, 2],
    ];
    let positions: Vec<_> = triangles.iter().flatten().map(|&i| points[i]).collect();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.compute_flat_normals();
    mesh
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let mut material = |color: Color| {
        mats.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 1.0,
            ..default()
        })
    };
    let art = Art {
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        ship: meshes.add(ship_mesh()),
        rock: meshes.add(Sphere::new(1.0).mesh().ico(0).unwrap()),
        hull: material(Color::srgb(0.72, 0.82, 0.9)),
        blue: material(Color::srgb(0.04, 0.8, 1.0)),
        red: material(Color::srgb(1.0, 0.06, 0.32)),
        stone: material(Color::srgb(0.34, 0.045, 0.42)),
        laser: material(Color::srgb(0.05, 1.0, 0.9)),
        hostile_laser: material(Color::srgb(1.0, 0.3, 0.08)),
    };
    commands
        .spawn((
            Player,
            Mesh3d(art.ship.clone()),
            MeshMaterial3d(art.hull.clone()),
            Transform::default(),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(art.cube.clone()),
                MeshMaterial3d(art.blue.clone()),
                Transform::from_xyz(0.0, 0.32, 0.1).with_scale(Vec3::new(0.35, 0.28, 0.65)),
            ));
            for x in [-0.28, 0.28] {
                parent.spawn((
                    Mesh3d(art.cube.clone()),
                    MeshMaterial3d(art.laser.clone()),
                    Transform::from_xyz(x, 0.0, 0.85).with_scale(Vec3::new(0.18, 0.18, 0.3)),
                ));
            }
        });
    let camera_pose =
        Transform::from_xyz(0.0, 3.0, 12.0).looking_at(Vec3::new(0.0, 0.0, -28.0), Vec3::Y);
    commands.spawn((
        Camera3d::default(),
        IsDefaultUiCamera,
        camera_pose,
        impact::ImpactCamera::new(camera_pose),
        bevy::post_process::effect_stack::ChromaticAberration {
            intensity: impact::CHROMATIC_BASELINE,
            ..default()
        },
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 14000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.4, 0.0)),
    ));
    scenery::setup(&mut commands, &mut meshes, &mut mats);
    commands.spawn((
        Hud,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(Color::srgb(0.25, 1.0, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            top: px(18),
            left: px(22),
            ..default()
        },
    ));
    commands.spawn((
        Text::new(
            "WASD / arrows: steer   SPACE: fire\nSHIFT: roll   P: pause\nR: restart   ESC: quit",
        ),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(18),
            right: px(22),
            ..default()
        },
    ));
    commands.insert_resource(explosion::setup(&mut meshes, &mut mats));
    commands.insert_resource(art);
}

fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut flight: ResMut<Flight>,
    mut commands: Commands,
    transient: Query<Entity, With<Transient>>,
    mut player: Query<(&mut Transform, &mut Visibility), With<Player>>,
    mut exit: MessageWriter<AppExit>,
    mut close_requests: MessageReader<bevy::window::WindowCloseRequested>,
) {
    if keys.just_pressed(KeyCode::Escape) || close_requests.read().count() > 0 {
        exit.write(AppExit::Success);
        return;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        for entity in &transient {
            commands.entity(entity).despawn();
        }
        *flight = Flight::default();
        if let Ok((mut transform, mut visibility)) = player.single_mut() {
            *transform = Transform::default();
            *visibility = Visibility::Inherited;
        }
    }
    if keys.just_pressed(KeyCode::KeyP) && !flight.ended {
        flight.paused = !flight.paused;
    }
}

fn pilot(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut flight: ResMut<Flight>,
    mut player: Query<(&mut Transform, &mut Visibility), With<Player>>,
    mut commands: Commands,
    art: Res<Art>,
    mut sounds: MessageWriter<Sound>,
) {
    if !flight.running() {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    flight.elapsed += dt;
    if flight.elapsed >= MISSION {
        flight.ended = true;
        sounds.write(Sound::Victory);
        return;
    }
    flight.shot = (flight.shot - dt).max(0.0);
    flight.invulnerable = (flight.invulnerable - dt).max(0.0);
    flight.roll = (flight.roll - dt).max(0.0);
    flight.roll_cooldown = (flight.roll_cooldown - dt).max(0.0);
    let Ok((mut transform, mut visibility)) = player.single_mut() else {
        return;
    };
    let axis = |positive, alternative, negative, other| {
        (keys.pressed(positive) || keys.pressed(alternative)) as i32 as f32
            - (keys.pressed(negative) || keys.pressed(other)) as i32 as f32
    };
    let direction = Vec2::new(
        axis(
            KeyCode::KeyD,
            KeyCode::ArrowRight,
            KeyCode::KeyA,
            KeyCode::ArrowLeft,
        ),
        axis(
            KeyCode::KeyW,
            KeyCode::ArrowUp,
            KeyCode::KeyS,
            KeyCode::ArrowDown,
        ),
    )
    .normalize_or_zero();
    transform.translation.x = (transform.translation.x + direction.x * dt * 9.0).clamp(-6.2, 6.2);
    transform.translation.y = (transform.translation.y + direction.y * dt * 9.0).clamp(-3.3, 4.2);
    if (keys.just_pressed(KeyCode::ShiftLeft) || keys.just_pressed(KeyCode::ShiftRight))
        && flight.roll_cooldown == 0.0
    {
        sounds.write(Sound::Roll);
        flight.roll = 0.7;
        flight.roll_cooldown = 3.0;
        flight.invulnerable = flight.invulnerable.max(0.7);
    }
    let roll = if flight.roll > 0.0 {
        (1.0 - flight.roll / 0.7) * TAU
    } else {
        0.0
    };
    flight.steering = animate_steering(flight.steering, direction, dt);
    transform.rotation = Quat::from_rotation_z(roll) * flight.steering;
    *visibility = if flight.invulnerable > 0.0
        && flight.roll == 0.0
        && ((flight.elapsed * 18.0) as u32).is_multiple_of(2)
    {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    if keys.pressed(KeyCode::Space) && flight.shot == 0.0 {
        sounds.write(Sound::Laser);
        flight.shot = 0.16;
        for x in [-0.45, 0.45] {
            spawn_bolt(
                &mut commands,
                &art,
                transform.translation + Vec3::new(x, 0.0, -1.3),
                Vec3::NEG_Z * 85.0,
                true,
            );
        }
    }
}

fn animate_steering(current: Quat, direction: Vec2, dt: f32) -> Quat {
    let target = Quat::from_euler(
        EulerRot::XYZ,
        direction.y * 0.12,
        -direction.x * 0.12,
        -direction.x * 0.35,
    );
    current.slerp(target, 1.0 - (-10.0 * dt).exp()).normalize()
}

fn spawn_bolt(commands: &mut Commands, art: &Art, position: Vec3, velocity: Vec3, friendly: bool) {
    commands.spawn((
        Transient,
        Bolt {
            velocity,
            previous: position,
            friendly,
        },
        Mesh3d(art.cube.clone()),
        MeshMaterial3d(if friendly {
            art.laser.clone()
        } else {
            art.hostile_laser.clone()
        }),
        Transform::from_translation(position)
            .looking_to(velocity, Vec3::Y)
            .with_scale(Vec3::new(0.12, 0.12, if friendly { 1.5 } else { 0.65 })),
    ));
}

fn waves(time: Res<Time>, mut flight: ResMut<Flight>, mut commands: Commands, art: Res<Art>) {
    if !flight.running() {
        return;
    }
    flight.spawn -= time.delta_secs().min(0.05);
    if flight.spawn > 0.0 {
        return;
    }
    flight.wave += 1;
    flight.spawn = (1.7 - flight.elapsed * 0.008).max(0.85);
    if !flight.wave.is_multiple_of(3) {
        let center = flight.random(-3.8, 3.8);
        let y = flight.random(-1.8, 3.0);
        for i in 0..3 {
            let home = Vec2::new(center + (i as f32 - 1.0) * 1.6, y);
            let phase = flight.random(0.0, TAU);
            commands.spawn((
                Transient,
                Enemy {
                    home,
                    phase,
                    shot: 1.6 + i as f32 * 0.4,
                },
                Mesh3d(art.ship.clone()),
                MeshMaterial3d(art.red.clone()),
                Transform::from_xyz(home.x, home.y, -100.0 - i as f32 * 4.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
                    .with_scale(Vec3::splat(0.7)),
            ));
        }
    } else if flight.wave.is_multiple_of(2) {
        // Two pillars leave a clearly visible opening to fly through.
        let gap = flight.random(-3.0, 3.0);
        for (x, width) in [
            ((-9.0 + gap - 1.8) * 0.5, gap + 7.2),
            ((gap + 1.8 + 9.0) * 0.5, 7.2 - gap),
        ] {
            let half = Vec3::new(width * 0.5, 5.5, 0.6);
            commands.spawn((
                Transient,
                Obstacle { half },
                Mesh3d(art.cube.clone()),
                MeshMaterial3d(art.stone.clone()),
                Transform::from_xyz(x, 0.0, -100.0).with_scale(half * 2.0),
            ));
        }
    } else {
        for _ in 0..3 {
            let position = Vec3::new(
                flight.random(-6.0, 6.0),
                flight.random(-3.0, 3.0),
                flight.random(-110.0, -95.0),
            );
            let half = Vec3::splat(flight.random(0.8, 1.6));
            commands.spawn((
                Transient,
                Obstacle { half },
                Mesh3d(art.rock.clone()),
                MeshMaterial3d(art.stone.clone()),
                Transform::from_translation(position).with_scale(half),
            ));
        }
    }
}

// Explicit disjoint filters let Bevy safely borrow each moving object's Transform.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn travel(
    time: Res<Time>,
    flight: Res<Flight>,
    art: Res<Art>,
    mut commands: Commands,
    mut sounds: MessageWriter<Sound>,
    player: Query<&Transform, With<Player>>,
    mut scenery: Query<&mut Transform, (With<Scenery>, Without<Player>)>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy), (Without<Player>, Without<Scenery>)>,
    mut obstacles: Query<
        (Entity, &mut Transform),
        (
            With<Obstacle>,
            Without<Player>,
            Without<Scenery>,
            Without<Enemy>,
        ),
    >,
    mut bolts: Query<
        (Entity, &mut Transform, &mut Bolt),
        (
            Without<Player>,
            Without<Scenery>,
            Without<Enemy>,
            Without<Obstacle>,
        ),
    >,
    mut sparks: Query<
        (Entity, &mut Transform, &mut Spark),
        (
            Without<Player>,
            Without<Scenery>,
            Without<Enemy>,
            Without<Obstacle>,
            Without<Bolt>,
        ),
    >,
) {
    if !flight.running() {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let speed = SPEED + flight.elapsed * 0.08;
    for mut t in &mut scenery {
        t.translation.z += speed * dt;
        if t.translation.z > 14.0 {
            t.translation.z -= 176.0;
        }
    }
    for (entity, mut t, mut enemy) in &mut enemies {
        t.translation.z += (speed + 3.0) * dt;
        t.translation.x = enemy.home.x + (flight.elapsed * 1.8 + enemy.phase).sin() * 0.9;
        t.translation.y = enemy.home.y + (flight.elapsed * 1.3 + enemy.phase).cos() * 0.4;
        let direction = Vec2::new(
            (flight.elapsed * 1.8 + enemy.phase).cos(),
            -(flight.elapsed * 1.3 + enemy.phase).sin(),
        );
        let facing = Quat::from_rotation_y(std::f32::consts::PI);
        t.rotation = animate_steering(t.rotation * facing.inverse(), direction, dt) * facing;
        enemy.shot -= dt;
        if enemy.shot <= 0.0 && t.translation.z > -65.0 && t.translation.z < -8.0 {
            enemy.shot = 2.2;
            if let Ok(player) = player.single() {
                let velocity = (player.translation - t.translation).normalize_or_zero() * 38.0;
                sounds.write(Sound::EnemyLaser);
                spawn_bolt(&mut commands, &art, t.translation, velocity, false);
            }
        }
        if t.translation.z > 10.0 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut t) in &mut obstacles {
        t.translation.z += speed * dt;
        if t.translation.z > 12.0 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut t, mut bolt) in &mut bolts {
        bolt.previous = t.translation;
        t.translation += bolt.velocity * dt;
        if t.translation.z < -150.0
            || t.translation.z > 16.0
            || t.translation.x.abs() > 25.0
            || t.translation.y.abs() > 25.0
        {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut t, mut spark) in &mut sparks {
        spark.life -= dt;
        t.translation += (spark.velocity + Vec3::Z * speed) * dt;
        t.scale *= 1.0 - dt * 2.0;
        if spark.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn intersects_box(center: Vec3, half: Vec3, point: Vec3, radius: f32) -> bool {
    let closest = point.clamp(center - half, center + half);
    closest.distance_squared(point) <= radius * radius
}

// Sweep fast lasers through the whole frame so they cannot tunnel through a target.
fn segment_box(start: Vec3, end: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
    let delta = end - start;
    let low = center - half;
    let high = center + half;
    let (mut enter, mut leave) = (0.0_f32, 1.0_f32);
    for axis in 0..3 {
        if delta[axis].abs() < 1e-6 {
            if start[axis] < low[axis] || start[axis] > high[axis] {
                return None;
            }
        } else {
            let a = (low[axis] - start[axis]) / delta[axis];
            let b = (high[axis] - start[axis]) / delta[axis];
            enter = enter.max(a.min(b));
            leave = leave.min(a.max(b));
            if enter > leave {
                return None;
            }
        }
    }
    Some(enter)
}

fn burst(commands: &mut Commands, art: &Art, flight: &mut Flight, position: Vec3) {
    for _ in 0..12 {
        let velocity = Vec3::new(
            flight.random(-7.0, 7.0),
            flight.random(-7.0, 7.0),
            flight.random(-7.0, 7.0),
        );
        commands.spawn((
            Transient,
            Spark {
                velocity,
                life: 0.55,
            },
            Mesh3d(art.cube.clone()),
            MeshMaterial3d(art.hostile_laser.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(0.22)),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn combat(
    mut commands: Commands,
    mut flight: ResMut<Flight>,
    art: Res<Art>,
    explosion_art: Res<explosion::ExplosionArt>,
    player: Query<&Transform, With<Player>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    obstacles: Query<(Entity, &Transform, &Obstacle)>,
    bolts: Query<(Entity, &Transform, &Bolt)>,
    mut sounds: MessageWriter<Sound>,
) {
    if !flight.running() {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let mut destroyed = Vec::new();
    for (entity, t, bolt) in &bolts {
        if bolt.friendly {
            let enemy_hits = enemies
                .iter()
                .filter(|(enemy, _)| !destroyed.contains(enemy))
                .filter_map(|(enemy, target)| {
                    segment_box(
                        bolt.previous,
                        t.translation,
                        target.translation,
                        Vec3::new(1.05, 0.65, 1.05),
                    )
                    .map(|distance| (distance, Some(enemy), target.translation))
                });
            let wall_hits = obstacles.iter().filter_map(|(_, target, obstacle)| {
                segment_box(
                    bolt.previous,
                    t.translation,
                    target.translation,
                    obstacle.half + Vec3::splat(0.1),
                )
                .map(|distance| (distance, None, target.translation))
            });
            if let Some((_, hit, position)) = enemy_hits
                .chain(wall_hits)
                .min_by(|a, b| a.0.total_cmp(&b.0))
            {
                commands.entity(entity).despawn();
                flight.impact = flight.impact.max(if hit.is_some() { 0.3 } else { 0.12 });
                if let Some(enemy) = hit {
                    destroyed.push(enemy);
                    commands.entity(enemy).despawn();
                    flight.score += 100;
                    sounds.write(Sound::Explosion);
                    explosion::spawn(&mut commands, &explosion_art, &mut flight, position);
                }
            }
        } else if segment_box(
            bolt.previous,
            t.translation,
            player.translation,
            Vec3::splat(0.7),
        )
        .is_some()
        {
            commands.entity(entity).despawn();
            if flight.damage() {
                flight.impact = if flight.ended { 1.0 } else { 0.65 };
                sounds.write(if flight.health == 0 {
                    Sound::Defeat
                } else {
                    Sound::Hit
                });
                burst(&mut commands, &art, &mut flight, player.translation);
            }
        }
    }
    let crash = obstacles.iter().any(|(_, t, obstacle)| {
        intersects_box(
            t.translation,
            obstacle.half,
            player.translation,
            SHIP_RADIUS,
        )
    }) || enemies.iter().any(|(e, t)| {
        !destroyed.contains(&e) && t.translation.distance_squared(player.translation) < 1.1 * 1.1
    });
    if crash && flight.damage() {
        flight.impact = if flight.ended { 1.0 } else { 0.85 };
        sounds.write(if flight.health == 0 {
            Sound::Defeat
        } else {
            Sound::Hit
        });
        burst(&mut commands, &art, &mut flight, player.translation);
    }
}

fn hud(flight: Res<Flight>, mut text: Query<&mut Text, With<Hud>>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let status = if flight.health == 0 {
        "SHIP LOST - press R to retry"
    } else if flight.ended {
        "MISSION COMPLETE - press R to fly again"
    } else if flight.paused {
        "PAUSED"
    } else {
        "SECTOR 01 - NEON SPRAWL"
    };
    let roll = if flight.roll_cooldown == 0.0 {
        "READY".to_string()
    } else {
        format!("{:.1}s", flight.roll_cooldown)
    };
    text.0 = format!(
        "STARWING   /   {status}\nSHIELDS {}/5   SCORE {:05}   DISTANCE {:02}%   ROLL {roll}",
        flight.health,
        flight.score,
        (flight.elapsed / MISSION * 100.0).min(100.0) as u32
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steering_eases_in_reverses_and_returns_to_level() {
        let target = animate_steering(Quat::IDENTITY, Vec2::X, 10.0);
        let first = animate_steering(Quat::IDENTITY, Vec2::X, 0.02);
        assert!(first.angle_between(Quat::IDENTITY) > 0.0);
        assert!(first.angle_between(target) > 0.1);
        let mut settled = first;
        for _ in 0..50 {
            settled = animate_steering(settled, Vec2::X, 0.02);
        }
        assert!(settled.angle_between(target) < 0.001);
        let reversed = animate_steering(settled, Vec2::NEG_X, 0.02);
        assert!(reversed.angle_between(settled) > 0.0);
        assert!(reversed.angle_between(target.inverse()) > 0.1);
        for _ in 0..50 {
            settled = animate_steering(settled, Vec2::ZERO, 0.02);
        }
        assert!(settled.angle_between(Quat::IDENTITY) < 0.001);
        let half_step = animate_steering(Quat::IDENTITY, Vec2::X, 0.01);
        let two_steps = animate_steering(half_step, Vec2::X, 0.01);
        assert!(two_steps.angle_between(first) < 0.001);
    }

    #[test]
    fn player_and_enemy_animate_without_interrupting_rolls() {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(20));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);
        app.world_mut().run_system_once(pilot).unwrap();
        let steering = app.world().resource::<Flight>().steering;
        assert!(steering.angle_between(Quat::IDENTITY) > 0.0);
        app.world_mut().resource_mut::<Flight>().roll = 0.37;
        app.world_mut().run_system_once(pilot).unwrap();
        let flight = app.world().resource::<Flight>();
        let expected = Quat::from_rotation_z((1.0 - flight.roll / 0.7) * TAU) * flight.steering;
        let player = app
            .world_mut()
            .query_filtered::<&Transform, With<Player>>()
            .single(app.world())
            .unwrap();
        assert!(player.rotation.angle_between(expected) < 0.001);

        let facing = Quat::from_rotation_y(std::f32::consts::PI);
        let enemy = app
            .world_mut()
            .spawn((
                Enemy {
                    home: Vec2::ZERO,
                    phase: 0.0,
                    shot: 10.0,
                },
                Transform::from_xyz(0.0, 0.0, -100.0).with_rotation(facing),
            ))
            .id();
        app.world_mut().run_system_once(travel).unwrap();
        let rotation = app.world().get::<Transform>(enemy).unwrap().rotation;
        assert!(rotation.angle_between(facing) > 0.0);
        assert!(rotation.angle_between(facing) < 0.1);
        app.world_mut().resource_mut::<Flight>().paused = true;
        app.world_mut().run_system_once(travel).unwrap();
        assert_eq!(
            app.world().get::<Transform>(enemy).unwrap().rotation,
            rotation
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyR);
        app.world_mut().run_system_once(controls).unwrap();
        assert_eq!(app.world().resource::<Flight>().steering, Quat::IDENTITY);
    }

    #[test]
    fn collision_handles_edges_and_depth() {
        assert!(intersects_box(
            Vec3::ZERO,
            Vec3::ONE,
            Vec3::new(1.4, 0.0, 0.0),
            0.5
        ));
        assert!(!intersects_box(
            Vec3::ZERO,
            Vec3::ONE,
            Vec3::new(1.6, 0.0, 0.0),
            0.5
        ));
        assert!(!intersects_box(
            Vec3::ZERO,
            Vec3::ONE,
            Vec3::new(0.0, 0.0, 4.0),
            0.5
        ));
    }

    #[test]
    fn lasers_sweep_and_hit_the_nearest_surface() {
        assert_eq!(
            segment_box(Vec3::Z * 5.0, Vec3::NEG_Z * 5.0, Vec3::ZERO, Vec3::ONE),
            Some(0.4)
        );
        assert!(
            segment_box(
                Vec3::new(3.0, 0.0, 5.0),
                Vec3::new(3.0, 0.0, -5.0),
                Vec3::ZERO,
                Vec3::ONE
            )
            .is_none()
        );
        assert_eq!(
            segment_box(Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ONE),
            Some(0.0)
        );
    }

    #[test]
    fn lasers_cannot_score_twice_or_shoot_through_walls() {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = test_app();
        let enemy = app
            .world_mut()
            .spawn((
                Enemy {
                    home: Vec2::ZERO,
                    phase: 0.0,
                    shot: 1.0,
                },
                Transform::from_xyz(0.0, 0.0, -6.0),
            ))
            .id();
        let wall = app
            .world_mut()
            .spawn((
                Obstacle { half: Vec3::ONE },
                Transform::from_xyz(0.0, 0.0, -3.0),
            ))
            .id();
        let bolt = || {
            (
                Bolt {
                    velocity: Vec3::NEG_Z * 85.0,
                    previous: Vec3::ZERO,
                    friendly: true,
                },
                Transform::from_xyz(0.0, 0.0, -8.0),
            )
        };
        app.world_mut().spawn(bolt());
        app.world_mut().run_system_once(combat).unwrap();
        assert_eq!(app.world().resource::<Flight>().score, 0);
        assert_eq!(app.world().resource::<Flight>().impact, 0.12);
        assert!(app.world().get_entity(enemy).is_ok());
        app.world_mut().despawn(wall);
        app.world_mut().spawn(bolt());
        app.world_mut().spawn(bolt());
        app.world_mut().run_system_once(combat).unwrap();
        assert_eq!(app.world().resource::<Flight>().score, 100);
        assert_eq!(app.world().resource::<Flight>().impact, 0.3);
        assert!(app.world().get_entity(enemy).is_err());
        assert_eq!(
            app.world_mut()
                .query::<&explosion::Particle>()
                .iter(app.world())
                .count(),
            72
        );
        assert_eq!(
            app.world()
                .resource::<Messages<Sound>>()
                .iter_current_update_messages()
                .filter(|sound| matches!(sound, Sound::Explosion))
                .count(),
            1
        );
    }

    #[test]
    fn pause_freezes_play_and_mission_finishes() {
        let mut app = test_app();
        app.world_mut().resource_mut::<Flight>().paused = true;
        app.update();
        assert_eq!(app.world().resource::<Flight>().elapsed, 0.0);
        app.world_mut().resource_mut::<Flight>().paused = false;
        app.world_mut().resource_mut::<Flight>().elapsed = MISSION;
        app.update();
        assert!(app.world().resource::<Flight>().ended);
        assert_eq!(app.world().resource::<Flight>().health, 5);
        assert_eq!(
            app.world()
                .resource::<Messages<Sound>>()
                .iter_current_update_messages()
                .filter(|sound| matches!(sound, Sound::Victory))
                .count(),
            1
        );
        let mut cursor = app
            .world()
            .resource::<Messages<Sound>>()
            .get_cursor_current();
        app.update();
        assert_eq!(
            cursor
                .read(app.world().resource::<Messages<Sound>>())
                .count(),
            0
        );
    }

    #[test]
    fn window_close_and_escape_exit_after_death_or_while_paused() {
        for paused in [false, true] {
            for close_window in [false, true] {
                let mut app = test_app();
                let window = app.world_mut().spawn(Window::default()).id();
                {
                    let mut flight = app.world_mut().resource_mut::<Flight>();
                    flight.health = 0;
                    flight.ended = true;
                    flight.paused = paused;
                }
                if close_window {
                    app.world_mut()
                        .write_message(bevy::window::WindowCloseRequested { window });
                } else {
                    app.world_mut()
                        .resource_mut::<ButtonInput<KeyCode>>()
                        .press(KeyCode::Escape);
                }
                app.update();
                assert!(matches!(app.should_exit(), Some(AppExit::Success)));
                assert!(app.world().get::<Window>(window).is_some());
            }
        }
    }

    #[test]
    fn restart_clears_explosion_particles() {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = test_app();
        app.world_mut()
            .run_system_once(
                |mut commands: Commands,
                 art: Res<explosion::ExplosionArt>,
                 mut flight: ResMut<Flight>| {
                    explosion::spawn(&mut commands, &art, &mut flight, Vec3::ZERO);
                },
            )
            .unwrap();
        assert_eq!(
            app.world_mut()
                .query::<&explosion::Particle>()
                .iter(app.world())
                .count(),
            72
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyR);
        app.world_mut().run_system_once(controls).unwrap();
        assert_eq!(
            app.world_mut()
                .query::<&explosion::Particle>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn impacts_distinguish_damage_crashes_and_ignore_immunity() {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = test_app();
        let hostile_bolt = || {
            (
                Bolt {
                    velocity: Vec3::Z * 38.0,
                    previous: Vec3::NEG_Z * 2.0,
                    friendly: false,
                },
                Transform::from_xyz(0.0, 0.0, 2.0),
            )
        };
        app.world_mut().spawn(hostile_bolt());
        app.world_mut().run_system_once(combat).unwrap();
        assert_eq!(app.world().resource::<Flight>().impact, 0.65);
        app.world_mut().resource_mut::<Flight>().impact = 0.0;
        app.world_mut().spawn(hostile_bolt());
        app.world_mut().run_system_once(combat).unwrap();
        assert_eq!(app.world().resource::<Flight>().impact, 0.0);
        app.world_mut()
            .spawn((Obstacle { half: Vec3::ONE }, Transform::default()));
        app.world_mut().resource_mut::<Flight>().invulnerable = 0.0;
        app.world_mut().run_system_once(combat).unwrap();
        assert_eq!(app.world().resource::<Flight>().impact, 0.85);
        {
            let mut flight = app.world_mut().resource_mut::<Flight>();
            flight.health = 1;
            flight.invulnerable = 0.0;
        }
        app.world_mut().run_system_once(combat).unwrap();
        assert_eq!(app.world().resource::<Flight>().impact, 1.0);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyR);
        app.world_mut().run_system_once(controls).unwrap();
        app.world_mut().run_system_once(impact::animate).unwrap();
        assert_eq!(app.world().resource::<Flight>().impact, 0.0);
        let (pose, camera, aberration) = app
            .world_mut()
            .query::<(
                &Transform,
                &impact::ImpactCamera,
                &bevy::post_process::effect_stack::ChromaticAberration,
            )>()
            .single(app.world())
            .unwrap();
        assert_eq!(*pose, camera.base);
        assert_eq!(aberration.intensity, impact::CHROMATIC_BASELINE);
    }

    #[test]
    fn shields_and_roll_immunity() {
        let mut flight = Flight::default();
        assert!(flight.damage());
        assert_eq!(flight.health, 4);
        assert!(!flight.damage());
        for _ in 0..4 {
            flight.invulnerable = 0.0;
            flight.damage();
        }
        assert!(flight.ended);
        assert_eq!(flight.health, 0);
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<StandardMaterial>::default())
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Flight>()
            .add_message::<AppExit>()
            .add_message::<bevy::window::WindowCloseRequested>()
            .add_message::<Sound>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    controls,
                    pilot,
                    waves,
                    travel,
                    combat,
                    explosion::animate,
                    impact::animate,
                    hud,
                )
                    .chain(),
            );
        app.update();
        app
    }

    #[test]
    fn gameplay_systems_run_without_query_conflicts() {
        let mut app = test_app();
        app.world_mut().resource_mut::<Flight>().spawn = 0.0;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();
        assert_eq!(
            app.world_mut().query::<&Enemy>().iter(app.world()).count(),
            3
        );
        assert_eq!(
            app.world_mut().query::<&Bolt>().iter(app.world()).count(),
            2
        );
        assert_eq!(
            app.world()
                .resource::<Messages<Sound>>()
                .iter_current_update_messages()
                .filter(|sound| matches!(sound, Sound::Laser))
                .count(),
            1
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyR);
        app.update();
        assert_eq!(app.world().resource::<Flight>().health, 5);
        assert_eq!(
            app.world_mut().query::<&Enemy>().iter(app.world()).count(),
            0
        );
    }
}
