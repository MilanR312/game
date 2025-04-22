use crate::{pathfinding::Graph, ui::UIState};
use bevy::{math::{vec3, vec3a, Vec3A}, prelude::*, render::view::RenderLayers};

use super::map::{HeightMap, MapData};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct InWorldPathGizmos{}

#[derive(Component)]
pub struct Route;



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

    commands.spawn((
        graph,
        Route,
    ));
}

pub fn calculate_path(
    mut path: ResMut<Path>,
    camera: Single<&mut Transform, With<Camera>>,
    graph: Single<&Graph, With<Route>>,
    height: Res<HeightMap>,
    map_data: Res<MapData>
) {
    let position = |x: f32, z: f32| {
        let y = height.height_at(x / map_data.size, z / map_data.size) * map_data.size  + 0.1;
        vec3(x,y, z)
    };
    let pos = camera.into_inner();
    let graph = graph.into_inner();
    let xz_pos = pos.translation.with_y(0.0);
    let start = position(xz_pos.x, xz_pos.z);
    let end = position(60.509605, 34.520927);
    *path = Path(
        get_curve(start, end, graph)
    );
}
#[derive(Resource)]
pub struct Path(Option<CubicCurve<Vec3>>);
pub fn draw_path(
    curve: Res<Path>,
    mut gizmos: Gizmos<InWorldPathGizmos>,
    ui_state: ResMut<UIState>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let Some(curve) = &curve.0 else {
        return
    };
    let (gizmo_config, _) = config_store.config_mut::<InWorldPathGizmos>();
    if ui_state.map_displayed {
        gizmo_config.depth_bias = -1.0;
    } else {
        gizmo_config.depth_bias = 0.0;
    }
    let resolution = 100 * curve.segments().len();
    gizmos.linestrip(
        curve.iter_positions(resolution),
        Color::srgb(1.0, 0.0, 0.0)
    );

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

    CubicCardinalSpline::new_catmull_rom(control_points).to_curve().ok()
}