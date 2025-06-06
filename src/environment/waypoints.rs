use crate::{pathfinding::Graph, ui::DebugUiState};
use bevy::{asset::RenderAssetUsages, math::{vec3, vec3a, Vec3A}, pbr::{ExtendedMaterial, MaterialExtension, NotShadowCaster, NotShadowReceiver}, prelude::*, render::{mesh::{MeshVertexAttribute, PrimitiveTopology}, render_resource::{AsBindGroup, VertexFormat}, view::RenderLayers}};
use itertools::Itertools;

use super::map::{HeightMap, MapData};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct InWorldPathGizmos{}

#[derive(Component)]
pub struct Route{
    pub destination: Option<Vec3>
}


pub fn spawn_waypoints(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut config_store: ResMut<GizmoConfigStore>,
    height: Res<HeightMap>,
    map_data: Res<MapData>
) {
    let mut graph = Graph::new();
    let position = |x: f32, z: f32| {
        let y = height.height_at(x / map_data.size, z / map_data.size) * map_data.size  + 0.1;
        vec3(x,y, z)
    };
    let root = graph.add_position(position(0.0, 0.0));
    let end1 = graph.add_line(root, [
        position(54.340187, 29.405205),
        position(54.985283, 30.839685),
    ]);
    let end2 = graph.add_line(end1, [
        position(55.31216, 31.551071),
        position(55.774982, 32.461918)]);
    let end3 = graph.add_line(end2, [
        position(56.72774, 33.14938),
        position(56.8367, 34.662132),
        position(57.91015, 34.920918),
    ]);
    let _ = graph.add_line(end2, [
        position(58.37181, 35.225315),
    ]);
    let end3 = graph.add_line(end1, [
        position(56.023685, 30.297434),
        position(56.367462, 30.868362),
        position(56.544098, 32.442364),
        position(57.459675, 32.961998),
        position(57.726906, 34.238174),
    ]);
    graph.add_connection(end2, end3);

    let end1 = graph.add_line(root, [
        position(55.259304, 28.302235),
position(55.80431, 29.14139),
position(56.976105, 28.964819),
position(57.044518, 29.441828),
position(57.486248, 29.472027),
position(57.75024, 30.276371),
position(57.556965, 30.811337),
position(57.156387, 31.675951),
    ]);
    let _ = graph.add_line(end1, [
position(57.864807, 32.230354),
position(58.392185, 32.715355),
position(59.119495, 32.686176),
position(59.49411, 33.190777),
position(60.263203, 33.449413),
position(60.509605, 34.520927),
    ]);
    graph.add_connection(end2, end1);



    let node_material = materials.add(Color::linear_rgb(0.0, 1.0, 0.0));
    let node_mesh = meshes.add(Cuboid::from_length(0.5));
    /*for &pos in graph.positions() {
        commands.spawn((
            MeshMaterial3d(node_material.clone()),
            Mesh3d(node_mesh.clone()),

            Transform::from_translation(pos.into())
        ));
    }*/
    let (gizmo_config, _) = config_store.config_mut::<InWorldPathGizmos>();
    gizmo_config.depth_bias = 0.0;
    gizmo_config.line_width = 2.0;
    commands.insert_resource(Path(
        get_curve(vec3(0.0, 0.0, 0.0), vec3(50.0, 0.0, 30.0), &graph)
    ));
    commands.insert_resource(DynamicPath(None));

    commands.spawn((
        graph,
        Route{
            destination: None
        },
    ));
}

pub fn calculate_path(
    mut path: ResMut<Path>,
    camera: Single<&mut Transform, With<Camera>>,
    graph: Single<(&Graph, &Route)>,
    height: Res<HeightMap>,
    map_data: Res<MapData>
) {
    let position = |x: f32, z: f32| {
        let y = height.height_at_checked(x / map_data.size, z / map_data.size)? * map_data.size  + 0.1;
        Some(vec3(x,y, z))
    };
    let pos = camera.into_inner();
    let (graph, route) = graph.into_inner();
    let xz_pos = pos.translation.with_y(0.0);
    let Some(start) = position(xz_pos.x, xz_pos.z) else {
        return;
    };
    let Some(end) = route.destination else {
        *path = Path(None);
        return;
    };

    *path = Path(
        get_curve(start, end, graph)
    );
}
#[derive(Resource)]
pub struct Path(Option<CubicCurve<Vec3>>);
#[derive(Resource)]
pub struct DynamicPath(Option<(Handle<Mesh>, Handle<WayPointMaterial>)>);
pub fn draw_settings(
    ui_state: ResMut<DebugUiState>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let (gizmo_config, _) = config_store.config_mut::<InWorldPathGizmos>();
    if ui_state.map_displayed {
        gizmo_config.depth_bias = -1.0;
    } else {
        gizmo_config.depth_bias = 0.0;
    }
}
pub fn draw_path(
    curve: Res<Path>,
    mut gizmos: Gizmos<InWorldPathGizmos>,
    
) {
    let Some(curve) = &curve.0 else {
        return
    };
    
    let resolution = 100 * curve.segments().len();
    gizmos.linestrip(
        curve.iter_positions(resolution),
        Color::srgb(1.0, 0.0, 0.0)
    );
}
#[derive(Clone, AsBindGroup, Asset, TypePath)]
pub struct WayPointMaterial{
    #[uniform(0)]
    t_start: f32,
    #[uniform(1)]
    is_disabled: u32
}
impl Material for WayPointMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "path.wgsl".into()
    }
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "path.wgsl".into()
    }
    fn specialize(
            pipeline: &bevy::pbr::MaterialPipeline<Self>,
            descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
            layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
            key: bevy::pbr::MaterialPipelineKey<Self>,
        ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
            let mut pos_position = 0;
            let mut normal_position = 1;
            if let Some(label) = &mut descriptor.label {
                if label == "pbr_prepass_pipeline" {
                    pos_position = 0;
                    normal_position = 3;
                    panic!("not supported");
                }
            }
            let vertex_layout = layout.0.get_layout(&[
                Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
                Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
                Mesh::ATTRIBUTE_NORMAL.at_shader_location(2),
                // Mesh::ATTRIBUTE_TANGENT.at_shader_location(4),
               // ATTRIBUTE_PATH_DISTANCE.at_shader_location(3),
                //ATTRIBUTE_PATH_LENGTH.at_shader_location(4),
            ])?;
            descriptor.vertex.buffers = vec![vertex_layout];
            Ok(())
    }
}





pub fn draw_path_v2(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WayPointMaterial>>,
    curve: Res<Path>,
    mut curve_mesh: ResMut<DynamicPath>,
) {
    // this version actually draws a mesh instead of drawing the gizmo
    let Some(curve) = &curve.0 else {
        let Some(x) = &curve_mesh.0 else {
            return;
        };

        if let Some(old_material) = materials.get_mut(x.1.id()) {
            old_material.is_disabled = 1;
        }
        return
    };
    // in a function so it isnt made every iteration, just once when it is called
    fn material(t_start: f32) -> WayPointMaterial{
        WayPointMaterial { t_start, is_disabled: 1  }
        
    }
    let sub_resolution = 10;
    let resolution = sub_resolution * curve.segments().len();
    let segments_to_display = 10;
    let mut positions = Vec::with_capacity(2 * segments_to_display * sub_resolution);
    let mut normals = Vec::with_capacity(2 * segments_to_display * sub_resolution);
    let mut uvs = Vec::with_capacity(2 * segments_to_display * sub_resolution);

    //let normals = vec![Vec3::new(0.0, 1.0, 0.0).to_array(); 2 * segments_to_display * 100];
    let width = 0.1;
    let normal = vec3(0.0, 1.0, 0.0).to_array();
    let total_length = (sub_resolution * segments_to_display) as f32;

    let next_fixed = curve.segments()[0].position(1.0);
    let player_pos = curve.segments()[0].position(0.0);

    let mut initial_uv = -next_fixed.distance(player_pos);
    let uvs_per_distance = 2.0;
    // https://en.wikipedia.org/wiki/Triangle_strip
    /*{
        let segment = curve.segments()[0];
        // in this segment we count backwards
        let stable_pos = segment.position(1.0);
        let pos = segment.position(0.0);
        let dist = stable_pos.distance(pos);
        for i in 0..sub_resolution {
            let t = i as f32 / sub_resolution as f32;
            let t_next = (i + 1) as f32 / sub_resolution as f32;
            let pos = segment.position(t);
            let pos_next = segment.position(t_next);
            let delta = (pos_next - pos).with_y(0.0).normalize_or_zero();

            let left = Vec3::new(-delta.z, 0.0, delta.x) * width;
            let right = Vec3::new(delta.z, 0.0, -delta.x) * width;
            let left = left + pos;
            let right = right + pos;
            positions.push(right.to_array());
            positions.push(left.to_array());
            normals.push(normal);
            normals.push(normal);

            let d = (1.0 - t_next) * dist;

            let uv = d * uvs_per_distance;

            uvs.push(Vec2::new(0.0, uv % 1.0).to_array());
            uvs.push(Vec2::new(0.0, uv % 1.0).to_array());
        }
    }*/

    let t_initial = {
        let segment = curve.segments()[0];
        let start = segment.position(0.0);
        let end = segment.position(1.0);
        let dist = start.distance(end);
        -dist
    };
    let mut t_step = -t_initial / 10.0;
    let mut t_outer = t_initial;

    for (idx, segment) in curve.segments().iter().enumerate() {
        let start = segment.position(0.0);
        let end = segment.position(1.0);
        let dist = 1.0 / segments_to_display as f32;
        let local_copy_length = total_length;
        for i in 0..sub_resolution {
            let t = i as f32 / sub_resolution as f32;
            let t_next = (i + 1) as f32 / sub_resolution as f32;
            let pos = segment.position(t);
            let pos_next = segment.position(t_next);

            let delta = (pos_next - pos).with_y(0.0).normalize_or_zero();

            let left = Vec3::new(-delta.z, 0.0, delta.x) * width;
            let right = Vec3::new(delta.z, 0.0, -delta.x) * width;
            let left = left + pos;
            let right = right + pos;
            positions.push(right.to_array());
            positions.push(left.to_array());
            normals.push(normal);
            normals.push(normal);
            let sign = if idx == 0 {
                -1.0
            } else {
                1.0
            };

            uvs.push(Vec2::new(-1.0, t_outer).to_array());
            uvs.push(Vec2::new(1.0, t_outer).to_array());
            t_outer += t_step;
        }
        t_step = 2.0 *  (segments_to_display - 1) as f32 / total_length;
    }
   
   
    let mesh = Mesh::new(PrimitiveTopology::TriangleStrip, RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    let new = match curve_mesh.0.take() {
        Some(x) => {
            if let Some(old_mesh) = meshes.get_mut(x.0.id()){
                *old_mesh = mesh;
            }
            if let Some(old_material) = materials.get_mut(x.1.id()) {
                old_material.t_start = t_initial;
                old_material.is_disabled = 0;
            }
            Some(x)
        },
        None => {
            let mesh = meshes.add(mesh);
            let material = materials.add(material(t_initial));
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                NotShadowCaster,
                NotShadowReceiver,
            ));
            Some((mesh, material))
        }
    };
    curve_mesh.0 = new;

}
// for the path to look good we need at least
// C1 continuity (actually G1 but C1 is a superset)
// for it to pass through the control points
// and for ease of creation
// for it to only be defined by the nodes
//
// for this we use the catmull-rom spline
// for info about continuity and other types of splines
// see: https://www.youtube.com/watch?v=jvPPXbo87ds
fn get_curve(
    start: Vec3,
    destination: Vec3,
    graph: &Graph
) -> Option<CubicCurve<Vec3>>{
    let control_points = graph.a_star_positions(start, destination)?;
    //CubicBSpline::new(control_points).to_curve().ok()
    CubicCardinalSpline::new_catmull_rom(control_points).to_curve().ok()
    
}