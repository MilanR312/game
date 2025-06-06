use bevy::prelude::*;
use bevy::render::mesh::{Mesh, SphereMeshBuilder, SphereKind};
use bevy::math::primitives::Sphere;
use noise::{NoiseFn, Perlin};
use rand::prelude::*;

#[derive(Component)]
pub struct CloudBall {
    base_pos: Vec3,
    offset: f32,
    drift_dir: Vec3,
    material_handle: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct CloudColor {
    r: f32,
    g: f32,
    b: f32,
}

pub struct CloudPlugin;

impl Plugin for CloudPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            color: Color::rgb(0.95, 0.96, 1.0),
            brightness: 0.7,
        })
        .add_systems(Startup, spawn_cloud_mass)
        .add_systems(Update, (animate_cloud_mass, fade_clouds_by_distance));
    }
}

fn spawn_cloud_mass(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let perlin = Perlin::new(1337);
    let mut rng = rand::thread_rng();

    let sphere_mesh = meshes.add(
        SphereMeshBuilder {
            sphere: Sphere { radius: 1.0 },
            kind: SphereKind::Ico { subdivisions: 1 },
        }
        .build(),
    );

    let base_height = 50.0;
    let scale = 2.5; 
    let spacing = 4.0;
    let size_x = 60;
    let size_z = 40;

    let mut potential_positions = Vec::new();

    for x in -size_x..size_x {
        for z in -size_z..size_z {
            let xf = x as f64 * 0.12;
            let zf = z as f64 * 0.12;
            let noise_val = perlin.get([xf, zf]);

            if noise_val > 0.38 {
                let mut neighbor_count = 0;
                for dx in -2..=2 {
                    for dz in -2..=2 {
                        if dx == 0 && dz == 0 {
                            continue;
                        }
                        let nx = (x + dx) as f64 * 0.12;
                        let nz = (z + dz) as f64 * 0.12;
                        if perlin.get([nx, nz]) > 0.38 {
                            neighbor_count += 1;
                        }
                    }
                }
                if neighbor_count < 8 {
                    continue;
                }

                let intensity = (noise_val - 0.38) / (1.0 - 0.38);
                let cluster_scale = scale * intensity as f32 + 0.6;

                let pos = Vec3::new(
                    x as f32 * spacing + rng.gen_range(-0.3..0.3),
                    base_height + rng.gen_range(-0.8..0.8),
                    z as f32 * spacing + rng.gen_range(-0.3..0.3),
                );

                potential_positions.push((pos, cluster_scale));
            }
        }
    }

    let mut valid_positions = Vec::new();

    for (i, (pos_i, scale_i)) in potential_positions.iter().enumerate() {
        let mut touches_other = false;
        for (j, (pos_j, scale_j)) in potential_positions.iter().enumerate() {
            if i == j {
                continue;
            }
            let dist = pos_i.distance(*pos_j);
            let radius_sum = scale_i + scale_j;
            if dist < radius_sum * 0.95 {
                touches_other = true;
                break;
            }
        }
        if touches_other {
            valid_positions.push((*pos_i, *scale_i));
        }
    }

    for (pos, base_scale) in valid_positions {
        let stretch = Vec3::new(
            rng.gen_range(0.9..1.2),
            rng.gen_range(0.4..0.7),
            rng.gen_range(0.9..1.2),
        );

        let final_scale = Vec3::splat(base_scale) * stretch;

        let material_handle = materials.add(StandardMaterial {
            base_color: Color::srgba(0.92, 0.95, 1.0, 0.45),
            alpha_mode: AlphaMode::Premultiplied, 
            unlit: true,
            ..default()
        });

        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            Transform {
                translation: pos,
                scale: final_scale,
                ..default()
            },
            GlobalTransform::default(),
            CloudBall {
                base_pos: pos,
                offset: rng.gen_range(0.0..1.0) * 100.0,
                drift_dir: Vec3::new(0.01, 0.0, 0.004),
                material_handle,
            },
            CloudColor {
                r: 0.92,
                g: 0.95,
                b: 1.0,
            },
        ));
    }
}


fn animate_cloud_mass(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &CloudBall)>
) {
    let t = time.elapsed_secs();
    for (mut transform, cloud) in &mut query {
        let vertical = (t + cloud.offset).sin() * 0.4;
        transform.translation = cloud.base_pos + Vec3::Y * vertical + cloud.drift_dir * t;
    }
}

fn fade_clouds_by_distance(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Transform, &CloudBall, &CloudColor)>
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        let cam_pos = camera_transform.translation;

        for (transform, cloud, color) in &query {
            let distance = transform.translation.distance(cam_pos);
            let fade_start = 150.0;
            let fade_end = 300.0;
        
            let alpha = if distance < fade_start {
                0.5
            } else if distance > fade_end {
                0.0
            } else {
                let t = (fade_end - distance) / (fade_end - fade_start);
                t * 0.5
            };
        
            if let Some(mat) = materials.get_mut(&cloud.material_handle) {
                mat.base_color = Color::srgba(color.r, color.g, color.b, alpha);
            }
        }
    }
}
