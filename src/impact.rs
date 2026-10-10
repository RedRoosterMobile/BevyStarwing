use bevy::{post_process::effect_stack::ChromaticAberration, prelude::*};

use crate::Flight;

pub const CHROMATIC_BASELINE: f32 = 0.02;

#[derive(Component)]
pub struct ImpactCamera {
    pub base: Transform,
    phase: f32,
}

impl ImpactCamera {
    pub fn new(base: Transform) -> Self {
        Self { base, phase: 0.0 }
    }
}

pub fn animate(
    time: Res<Time>,
    mut flight: ResMut<Flight>,
    mut cameras: Query<(&mut Transform, &mut ChromaticAberration, &mut ImpactCamera)>,
) {
    for (mut transform, mut aberration, mut camera) in &mut cameras {
        if !flight.paused {
            camera.phase += time.delta_secs() * 65.0;
        }
        // Always offset the resting pose, never last frame's shake (no camera drift).
        let strength = flight.impact * flight.impact;
        let phase = camera.phase;
        *transform = camera.base;
        transform.translation += camera.base.rotation
            * Vec3::new(phase.sin(), (phase * 1.37).sin(), 0.0)
            * strength
            * 0.3;
        transform.rotation *= Quat::from_rotation_z((phase * 0.83).sin() * strength * 0.025);
        aberration.intensity = CHROMATIC_BASELINE + flight.impact * 0.18;
    }
    // Let the final hit settle even on the results screen; pause freezes the effect.
    if !flight.paused {
        flight.impact = (flight.impact - time.delta_secs() * 2.5).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn shake_scales_freezes_and_returns_exactly_to_rest() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(20));
        app.insert_resource(time).init_resource::<Flight>();
        let base =
            Transform::from_xyz(0.0, 3.0, 12.0).looking_at(Vec3::new(0.0, 0.0, -28.0), Vec3::Y);
        let camera = app
            .world_mut()
            .spawn((
                base,
                ChromaticAberration {
                    intensity: CHROMATIC_BASELINE,
                    ..default()
                },
                ImpactCamera::new(base),
            ))
            .id();
        app.world_mut().resource_mut::<Flight>().impact = 0.2;
        app.world_mut().run_system_once(animate).unwrap();
        let small = app
            .world()
            .get::<Transform>(camera)
            .unwrap()
            .translation
            .distance(base.translation);
        let weak_aberration = app
            .world()
            .get::<ChromaticAberration>(camera)
            .unwrap()
            .intensity;
        {
            let mut flight = app.world_mut().resource_mut::<Flight>();
            flight.impact = 1.0;
            flight.paused = true;
        }
        app.world_mut().run_system_once(animate).unwrap();
        let strong = *app.world().get::<Transform>(camera).unwrap();
        assert!(strong.translation.distance(base.translation) > small * 20.0);
        assert!(
            app.world()
                .get::<ChromaticAberration>(camera)
                .unwrap()
                .intensity
                > weak_aberration * 3.0
        );
        app.world_mut().run_system_once(animate).unwrap();
        assert_eq!(*app.world().get::<Transform>(camera).unwrap(), strong);
        assert_eq!(app.world().resource::<Flight>().impact, 1.0);
        {
            let mut flight = app.world_mut().resource_mut::<Flight>();
            flight.paused = false;
            flight.ended = true;
        }
        for _ in 0..22 {
            app.world_mut().run_system_once(animate).unwrap();
        }
        assert_eq!(app.world().resource::<Flight>().impact, 0.0);
        assert_eq!(*app.world().get::<Transform>(camera).unwrap(), base);
        assert_eq!(
            app.world()
                .get::<ChromaticAberration>(camera)
                .unwrap()
                .intensity,
            CHROMATIC_BASELINE
        );
    }
}
