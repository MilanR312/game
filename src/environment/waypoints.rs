use crate::pathfinding::Graph;
use bevy::{math::{vec3, vec3a, Vec3A}, prelude::*};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct PathGizmos{}

#[derive(Component)]
pub struct Route;

pub fn spawn_waypoints(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut config_store: ResMut<GizmoConfigStore>
) {
    let mut graph = Graph::new();
    let root = graph.add_position(vec3(0.0, 0.0, 0.0));
    let end1 = graph.add_line(root, [
        vec3(5.0, 0.0, -1.0),
        vec3(15.0, 1.0, 2.0),
        vec3(40.0, -1.0, 0.0),
        vec3(50.0, 0.0, 0.0)
    ]);
    let end2 = graph.add_line(root, [
        vec3(0.0, 2.0, 15.0),
        vec3(3.0, 0.0, 25.0),
        vec3(-2.0, 0.0, 40.0),
        vec3(0.0, 0.0, 50.0) 
    ]);
    let end3 = graph.add_line(end1, [
        vec3(50.0, 0.0, 10.0),
        vec3(50.0, 3.0, 20.0),
        vec3(30.0, 0.0, 40.0),
    ]);
    graph.add_connection(end2, end3); // finish the triangle
    


    let node_material = materials.add(Color::linear_rgb(0.0, 1.0, 0.0));
    let node_mesh = meshes.add(Cuboid::from_length(0.5));
    for &pos in graph.positions() {
        commands.spawn((
            MeshMaterial3d(node_material.clone()),
            Mesh3d(node_mesh.clone()),

            Transform::from_translation(pos.into())
        ));
    }
    let (gizmo_config, _) = config_store.config_mut::<PathGizmos>();
    gizmo_config.depth_bias = -1.0;
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
    graph: Single<&Graph, With<Route>>
) {
    let pos = camera.into_inner();
    let graph = graph.into_inner();
    let xz_pos = pos.translation.with_y(0.0);
    *path = Path(
        get_curve(xz_pos, vec3(50.0, 0.0, 30.0), graph)
    );
}
#[derive(Resource)]
pub struct Path(Option<CubicCurve<Vec3>>);
pub fn draw_path(
    curve: Res<Path>,
    mut gizmos: Gizmos<PathGizmos>
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