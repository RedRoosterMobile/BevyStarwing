use bevy::{prelude::*, render::render_resource::PrimitiveTopology};

use crate::Scenery;

const TILE: f32 = 8.0;
const TILES: usize = 22;
const FLOOR: f32 = -5.1;

#[derive(Default)]
struct PaintedMesh {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
}

impl PaintedMesh {
    fn triangle(&mut self, points: [Vec3; 3], color: Color) {
        self.positions.extend(points.map(|point| point.to_array()));
        self.colors.extend([color.to_linear().to_f32_array(); 3]);
    }

    // Real triangles, not polygon-mode wireframe (which WebGL cannot render).
    fn line(&mut self, a: Vec3, b: Vec3, width: f32, color: Color) {
        let direction = (b - a).normalize_or_zero();
        let side = direction.cross(Vec3::Y).normalize_or_zero() * width * 0.5;
        self.triangle([a - side, b - side, b + side], color);
        self.triangle([a - side, b + side, a + side], color);
    }

    fn finish(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.compute_flat_normals();
        mesh
    }
}

fn ridge(side: f32, column: usize, row: usize) -> Vec3 {
    let phase = (row % TILES) as f32 * std::f32::consts::TAU / TILES as f32;
    let height = if column == 0 {
        0.0
    } else {
        let wave = (phase * 3.0 + column as f32 * 1.9 + side).sin();
        let crest = (phase * 5.0 - column as f32 + side).cos();
        3.0 + column as f32 * 2.3 + wave * 3.0 + crest * 2.0
    };
    Vec3::new(
        side * (9.5 + column as f32 * 8.0),
        FLOOR + height.max(0.0),
        -(row as f32) * TILE,
    )
}

fn canyon_tile(row: usize) -> Mesh {
    let mut mesh = PaintedMesh::default();
    let cyan = Color::srgb(0.04, 0.8, 0.95);
    let pink = Color::srgb(1.0, 0.08, 0.55);
    let offset = Vec3::Z * row as f32 * TILE;
    let ground = Color::srgb(0.025, 0.008, 0.065);
    let a = Vec3::new(-9.5, FLOOR, 0.0);
    let b = Vec3::new(9.5, FLOOR, 0.0);
    mesh.triangle([a, b, b - Vec3::Z * TILE], ground);
    mesh.triangle([a, b - Vec3::Z * TILE, a - Vec3::Z * TILE], ground);
    for x in -4..=4 {
        let a = Vec3::new(x as f32 * 2.0, FLOOR + 0.025, 0.0);
        mesh.line(a, a - Vec3::Z * TILE, 0.035, cyan);
    }
    for z in [0.0, -4.0] {
        mesh.line(
            Vec3::new(-9.5, FLOOR + 0.025, z),
            Vec3::new(9.5, FLOOR + 0.025, z),
            0.045,
            pink,
        );
    }
    for side in [-1.0, 1.0] {
        mesh.line(
            Vec3::new(side * 9.3, FLOOR + 0.04, 0.0),
            Vec3::new(side * 9.3, FLOOR + 0.04, -TILE),
            0.12,
            pink,
        );
        for column in 0..5 {
            let a = ridge(side, column, row) + offset;
            let b = ridge(side, column + 1, row) + offset;
            let c = ridge(side, column, row + 1) + offset;
            let d = ridge(side, column + 1, row + 1) + offset;
            let face = Color::srgb(0.045 + column as f32 * 0.009, 0.009, 0.085);
            mesh.triangle([a, b, c], face);
            mesh.triangle([b, d, c], face);
            let edge = if column % 2 == 0 { pink } else { cyan };
            // Raise the ribbon slightly to avoid z-fighting with the mountain face.
            for (start, end) in [(a, b), (a, c), (b, c)] {
                mesh.line(start + Vec3::Y * 0.12, end + Vec3::Y * 0.12, 0.08, edge);
            }
            if column == 4 {
                mesh.line(b + Vec3::Y * 0.12, d + Vec3::Y * 0.12, 0.08, edge);
            }
        }
    }
    mesh.finish()
}

fn sunset() -> Mesh {
    let mut mesh = PaintedMesh::default();
    // The disc is sliced geometrically, so its black bands reveal the sky behind it.
    let radius = 27.0_f32;
    for band in 0..80 {
        let low = -radius + band as f32 * radius * 2.0 / 80.0;
        let high = low + radius * 2.0 / 80.0;
        if low < 0.0 && band % 8 < 3 {
            continue;
        }
        let left = (radius * radius - low * low).max(0.0).sqrt();
        let right = (radius * radius - high * high).max(0.0).sqrt();
        let t = band as f32 / 80.0;
        let color = Color::srgb(1.0, 0.08 + t * 0.72, 0.48 - t * 0.4);
        let a = Vec3::new(-left, low, 0.0);
        let b = Vec3::new(left, low, 0.0);
        let c = Vec3::new(-right, high, 0.0);
        let d = Vec3::new(right, high, 0.0);
        mesh.triangle([a, b, c], color);
        mesh.triangle([b, d, c], color);
    }
    mesh.finish()
}

pub fn setup(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let ink = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    for row in 0..TILES {
        commands.spawn((
            Scenery,
            Mesh3d(meshes.add(canyon_tile(row))),
            MeshMaterial3d(ink.clone()),
            Transform::from_xyz(0.0, 0.0, 14.0 - row as f32 * TILE),
        ));
    }
    // Keep the backdrop crisp while terrain fades into the horizon haze.
    let sky_ink = materials.add(StandardMaterial {
        fog_enabled: false,
        ..materials.get(&ink).unwrap().clone()
    });
    let mut sky = PaintedMesh::default();
    let stops = [
        (-180.0, Color::srgb(0.055, 0.008, 0.12)),
        (-12.0, Color::srgb(0.3, 0.025, 0.22)),
        (12.0, Color::srgb(0.48, 0.055, 0.25)),
        (55.0, Color::srgb(0.12, 0.015, 0.22)),
        (120.0, Color::srgb(0.018, 0.004, 0.055)),
        (350.0, Color::srgb(0.018, 0.004, 0.055)),
    ];
    for pair in stops.windows(2) {
        let (low, lower) = pair[0];
        let (high, upper) = pair[1];
        let points = [
            Vec3::new(-500.0, low, -245.0),
            Vec3::new(500.0, low, -245.0),
            Vec3::new(-500.0, high, -245.0),
            Vec3::new(500.0, high, -245.0),
        ];
        for index in [0, 1, 2, 1, 3, 2] {
            sky.positions.push(points[index].to_array());
            sky.colors.push(
                if index < 2 { lower } else { upper }
                    .to_linear()
                    .to_f32_array(),
            );
        }
    }
    // Sparse deterministic stars, kept above the sunset rather than over gameplay.
    for i in 0..90 {
        let x = ((i * 73 % 211) as f32 / 211.0 - 0.5) * 420.0;
        let y = 55.0 + (i * 47 % 137) as f32;
        let size = if i % 7 == 0 { 0.3 } else { 0.12 };
        let center = Vec3::new(x, y, -240.0);
        sky.triangle(
            [
                center + Vec3::Y * size,
                center - Vec3::X * size,
                center + Vec3::X * size,
            ],
            Color::srgb(0.45, 0.7, 0.9),
        );
    }
    commands.spawn((
        Mesh3d(meshes.add(sky.finish())),
        MeshMaterial3d(sky_ink.clone()),
    ));
    commands.spawn((
        Mesh3d(meshes.add(sunset())),
        MeshMaterial3d(sky_ink),
        Transform::from_xyz(0.0, 27.0, -230.0),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canyon_is_periodic_and_keeps_the_flight_lane_clear() {
        for side in [-1.0, 1.0] {
            for column in 0..=5 {
                let first = ridge(side, column, 0);
                let last = ridge(side, column, TILES);
                assert_eq!(first.x, last.x);
                assert_eq!(first.y, last.y);
                assert_eq!(first.z - last.z, TILE * TILES as f32);
                assert!(first.x.abs() >= 9.5);
            }
        }
        for row in 0..TILES {
            let mesh = canyon_tile(row);
            assert!(mesh.count_vertices() > 100);
            assert_eq!(mesh.count_vertices() % 3, 0);
        }
        assert!(sunset().count_vertices() > 100);
    }
}
