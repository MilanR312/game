
mod worldgen;
use bevy::{asset::RenderAssetUsages, diagnostic::{FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin}, pbr::{experimental::meshlet::MeshletPlugin, wireframe::{Wireframe, WireframeConfig, WireframePlugin}}, prelude::*, render::{diagnostic::RenderDiagnosticsPlugin, mesh::PrimitiveTopology, settings::{RenderCreation, WgpuFeatures, WgpuSettings}, RenderPlugin}};
use iyes_perf_ui::{prelude::PerfUiDefaultEntries, PerfUiPlugin};
use noise::NoiseFn;
use smooth_bevy_cameras::{controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin}, LookTransform, LookTransformPlugin};
use spade::{DelaunayTriangulation, Triangulation};




fn main() -> eyre::Result<()> {
    App::new().add_plugins(DefaultPlugins.set(RenderPlugin{
        render_creation: RenderCreation::Automatic(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        }),
            ..Default::default()
    }))
        .add_plugins(WireframePlugin)
        .insert_resource(WireframeConfig {
            global: true,
            default_color: Color::WHITE
        })
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(SystemInformationDiagnosticsPlugin)
        .add_plugins(LookTransformPlugin)
        .add_plugins(UnrealCameraPlugin::default())
        .add_plugins(RenderDiagnosticsPlugin)
        .add_plugins(MeshletPlugin{
            cluster_buffer_slots: 8192
        })
        .add_plugins(PerfUiPlugin)
        .add_systems(Startup, (setup))
        .run();
    Ok(())
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    commands.spawn(
        PerfUiDefaultEntries::default()
    );
    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        UnrealCameraBundle::new(
        UnrealCameraController::default(), 
        Vec3::new(-2.0, 5.0, -2.0), Vec3::splat(0.0), Vec3::Y)
    ));


    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(0.0, 10.0, 0.0)
    ));
    let height_map = worldgen::height_map();
    let chunk_count = 10;
    let mesh_resolution = 100;
    for i in 0..chunk_count {
        for j in 0..chunk_count {
            let x = i as f32 / chunk_count as f32 * 2.0 - 1.0;
            let y = j as f32 / chunk_count as f32 * 2.0 - 1.0;
            let mut mesh_points = vec![];
            let mut triangulation: DelaunayTriangulation<spade::Point2<f32>> = spade::DelaunayTriangulation::new();
            let mesh_width = 1.0 / chunk_count as f32;
            for delta_x in 0..mesh_resolution {
                for delta_y in 0..mesh_resolution {
                    let delta_x = delta_x as f32 / mesh_resolution as f32 * mesh_width;
                    let delta_y = delta_y as f32 / mesh_resolution as f32 * mesh_width;
                    triangulation.insert(spade::Point2::new(delta_x, delta_y)).unwrap();
                    let height = height_map.get([(x + delta_x) as f64, (y + delta_y) as f64]);
                    mesh_points.push(Vec3::new(x + delta_x,  height as f32, y + delta_y));
                }
            }
            let mut indices = vec![];
            for face in triangulation.inner_faces() {
                let verts = face.vertices();
                indices.push(verts[0].fix().index() as u32);
                indices.push(verts[1].fix().index() as u32);
                indices.push(verts[2].fix().index() as u32);
            }
            let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_points)
                .with_inserted_indices(bevy::render::mesh::Indices::U32(
                    indices
                ));
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                Wireframe,
                MeshMaterial3d(materials.add(Color::WHITE))
            ));
        }
    }

    /*commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::WHITE))
    ));*/
    
}


