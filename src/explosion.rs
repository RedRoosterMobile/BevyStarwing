use bevy::prelude::*;

use crate::{Flight, SPEED, Transient};

#[derive(Resource)]
pub struct ExplosionArt {
    sphere: Handle<Mesh>,
    chunk: Handle<Mesh>,
    streak: Handle<Mesh>,
    ring: Handle<Mesh>,
    flash: Handle<StandardMaterial>,
    fire: Handle<StandardMaterial>,
    debris: Handle<StandardMaterial>,
    smoke: Handle<StandardMaterial>,
}

pub fn setup(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> ExplosionArt {
    let mut glowing = |color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })
    };
    let flash = glowing(Color::srgb(1.0, 0.95, 0.65));
    let fire = glowing(Color::srgb(1.0, 0.32, 0.025));
    ExplosionArt {
        sphere: meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
        chunk: meshes.add(Sphere::new(1.0).mesh().ico(0).unwrap()),
        streak: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        ring: meshes.add(Torus::new(0.95, 1.05)),
        flash,
        fire,
        debris: materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.12, 0.08),
            ..default()
        }),
        smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(0.12, 0.065, 0.16, 0.35),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Flash,
    Ring,
    Spark,
    Debris,
    Smoke,
}

#[derive(Component)]
pub(crate) struct Particle {
    kind: Kind,
    velocity: Vec3,
    spin: Vec3,
    size: Vec3,
    age: f32,
    lifetime: f32,
}

pub fn spawn(commands: &mut Commands, art: &ExplosionArt, flight: &mut Flight, position: Vec3) {
    for (kind, count, mesh, material) in [
        (Kind::Flash, 1, &art.sphere, &art.flash),
        (Kind::Ring, 1, &art.ring, &art.fire),
        (Kind::Spark, 36, &art.streak, &art.fire),
        (Kind::Debris, 16, &art.chunk, &art.debris),
        (Kind::Smoke, 18, &art.sphere, &art.smoke),
    ] {
        for _ in 0..count {
            // Uniform directions on a sphere, including depth toward/away from the camera.
            let z = flight.random(-1.0, 1.0);
            let angle = flight.random(0.0, std::f32::consts::TAU);
            let radius = (1.0 - z * z).sqrt();
            let direction = Vec3::new(radius * angle.cos(), radius * angle.sin(), z);
            let (speed, lifetime, size) = match kind {
                Kind::Flash => (0.0, 0.22, Vec3::splat(0.4)),
                Kind::Ring => (0.0, 0.5, Vec3::splat(0.6)),
                Kind::Spark => (
                    flight.random(14.0, 28.0),
                    flight.random(0.35, 0.7),
                    Vec3::new(0.07, 0.07, flight.random(0.5, 1.2)),
                ),
                Kind::Debris => (
                    flight.random(5.0, 12.0),
                    flight.random(0.8, 1.4),
                    Vec3::splat(flight.random(0.12, 0.32)),
                ),
                Kind::Smoke => (
                    flight.random(2.0, 5.0),
                    flight.random(0.9, 1.6),
                    Vec3::splat(flight.random(0.35, 0.65)),
                ),
            };
            let rotation = if matches!(kind, Kind::Ring) {
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
            } else {
                Transform::default().looking_to(direction, Vec3::Y).rotation
            };
            commands.spawn((
                Transient,
                Particle {
                    kind,
                    velocity: direction * speed,
                    spin: if matches!(kind, Kind::Debris) {
                        Vec3::new(
                            flight.random(-8.0, 8.0),
                            flight.random(-8.0, 8.0),
                            flight.random(-8.0, 8.0),
                        )
                    } else {
                        Vec3::ZERO
                    },
                    size,
                    age: 0.0,
                    lifetime,
                },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(position)
                    .with_rotation(rotation)
                    .with_scale(size),
            ));
        }
    }
}

pub fn animate(
    time: Res<Time>,
    flight: Res<Flight>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut Particle)>,
) {
    if flight.paused {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut particle) in &mut particles {
        particle.age += dt;
        if particle.age >= particle.lifetime {
            commands.entity(entity).despawn();
            continue;
        }
        let progress = particle.age / particle.lifetime;
        transform.translation +=
            (particle.velocity + Vec3::Z * (SPEED + flight.elapsed * 0.08)) * dt;
        match particle.kind {
            Kind::Flash => transform.scale = Vec3::splat(0.4 + progress * 3.0) * (1.0 - progress),
            Kind::Ring => {
                transform.scale =
                    Vec3::new(1.0, (1.0 - progress).powi(2), 1.0) * (0.6 + progress * 7.0)
            }
            Kind::Spark => transform.scale = particle.size * (1.0 - progress),
            Kind::Debris => {
                particle.velocity.y -= 8.0 * dt;
                transform.rotation *= Quat::from_scaled_axis(particle.spin * dt);
                transform.scale = particle.size * ((1.0 - progress) * 4.0).min(1.0);
            }
            Kind::Smoke => {
                particle.velocity *= (-2.0 * dt).exp();
                particle.velocity.y += dt;
                transform.scale =
                    particle.size * (1.0 + progress * 3.0) * ((1.0 - progress) * 4.0).min(1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn explosion_spreads_in_3d_pauses_and_cleans_up() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time).init_resource::<Flight>();
        let art = setup(&mut Assets::default(), &mut Assets::default());
        app.insert_resource(art);
        app.world_mut()
            .run_system_once(
                |mut commands: Commands, art: Res<ExplosionArt>, mut flight: ResMut<Flight>| {
                    spawn(&mut commands, &art, &mut flight, Vec3::ZERO);
                },
            )
            .unwrap();
        let world = app.world_mut();
        let mut query = world.query::<(&Particle, &Transient)>();
        assert_eq!(query.iter(world).count(), 72);
        assert!(query.iter(world).any(|(p, _)| p.velocity.z > 5.0));
        assert!(query.iter(world).any(|(p, _)| p.velocity.z < -5.0));
        world.resource_mut::<Flight>().paused = true;
        world.run_system_once(animate).unwrap();
        assert!(query.iter(world).all(|(p, _)| p.age == 0.0));
        world.resource_mut::<Flight>().paused = false;
        world.resource_mut::<Flight>().ended = true;
        world.run_system_once(animate).unwrap();
        assert!(query.iter(world).all(|(p, _)| p.age > 0.0));
        for _ in 0..34 {
            world.run_system_once(animate).unwrap();
        }
        assert_eq!(query.iter(world).count(), 0);
    }
}
