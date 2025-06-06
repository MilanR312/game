use bevy::prelude::*;

/// specific points that are manually placed to make things look better
pub fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        SceneRoot(asset_server.load("house/house.glb#Scene0")),
        Transform::from_xyz(38.770237, 2.271224, 23.179953)
            .with_scale(Vec3::splat(0.1)),
    ));

    // Spawn the big tree
    commands.spawn((
        SceneRoot(asset_server.load("big_tree/big_tree.glb#Scene0")),
        Transform::from_xyz(36.6558, 3.5, 15.713834),
    ));

    // Spawn the rocks
    commands.spawn((
        SceneRoot(asset_server.load("rocks/self_made_rocks.glb#Scene0")),
        Transform::from_xyz(72.03199, 1.9987919, 65.12867)
        .with_scale(Vec3::splat(0.05)),
    ));

    // Spawn the ruins
    commands.spawn((
        SceneRoot(asset_server.load("ruins/ruins.glb#Scene0")),
        Transform {
            translation: Vec3::new(80.68331, 3.6289705, 26.552511),
            rotation: Quat::from_rotation_y(65.0_f32.to_radians()), // 45° 
            scale: Vec3::splat(0.075),
        },
    ));

    commands.spawn((
        SceneRoot(asset_server.load("shipwreck/shipwreck.glb#Scene0")),
        Transform {
            translation: Vec3::new(87.72732, 2.0260856, 66.71166), 
            rotation: Quat::from_rotation_y(25.0_f32.to_radians()), 
            scale: Vec3::splat(0.05), 
        },
    ));
    
    
}
