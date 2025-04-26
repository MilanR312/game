use bevy::prelude::*;

/// specific points that are manually placed to make things look better
pub fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn((
        SceneRoot(asset_server.load("big_oak/tree_gn.glb#Scene0")),
        Transform::from_xyz(10.0, 10.0, 10.0)
    ));
}