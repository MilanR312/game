use bevy::prelude::*;
use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy_hanabi::prelude::*;

pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_campfire);
    }
}

fn spawn_campfire(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    asset_server: Res<AssetServer>,
) {
    let fire_pos = Vec3::new(40.093216, 2.4406188, 24.556684);

    let campfire_scene: Handle<Scene> = asset_server.load("bonfire/bonfire.glb#Scene0");

    commands.spawn(SceneBundle {
        scene: SceneRoot(campfire_scene),
        transform: Transform {
            translation: fire_pos,
            scale: Vec3::splat(0.2),
            ..default()
        },
        ..default()
    });

    let flame_handle = create_flame_effect(&mut effects);
    commands.spawn((
        ParticleEffect::new(flame_handle),
        Transform::from_translation(fire_pos),
        GlobalTransform::default(),
    ));

    let smoke_handle = create_smoke_effect(&mut effects);
    commands.spawn((
        ParticleEffect::new(smoke_handle),
        Transform::from_translation(fire_pos),
        GlobalTransform::default(),
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 800.0,
            range: 15.0,
            color: Color::rgb(1.0, 0.6, 0.3),
            ..default()
        },
        transform: Transform::from_translation(fire_pos + Vec3::Y * 0.5),
        ..default()
    });

    let sound = asset_server.load("sounds/campfire-crackling.mp3");
    commands.spawn((
        AudioPlayer::new(sound),
        PlaybackSettings::LOOP,
        Transform::from_translation(fire_pos),
    ));
}

fn create_flame_effect(effects: &mut ResMut<Assets<EffectAsset>>) -> Handle<EffectAsset> {
    let mut color_grad = Gradient::new();
    color_grad.add_key(0.0, Vec4::new(1.0, 0.8, 0.2, 1.0));
    color_grad.add_key(0.3, Vec4::new(1.0, 0.4, 0.0, 0.7));
    color_grad.add_key(1.0, Vec4::new(0.1, 0.1, 0.1, 0.0));

    let mut size_grad = Gradient::new();
    size_grad.add_key(0.0, Vec3::splat(0.02));  // kleiner begin
    size_grad.add_key(0.2, Vec3::splat(0.05));  // lager piekvolume
    size_grad.add_key(1.0, Vec3::splat(0.01));  // kleinere uitsterving


    let mut writer = ExprWriter::new();

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(0.05).expr(),
        dimension: ShapeDimension::Surface,
    };

    let velocity = writer
        .lit(Vec3::Y * 0.7)
        .uniform(writer.lit(Vec3::new(0.05, 0.2, 0.05)));
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, velocity.expr());

    let lifetime = writer.lit(1.0).uniform(writer.lit(1.5)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let wind = Vec3::new(0.15, 0.2, 0.05); // windrichting omhoog + opzij
    let accel = AccelModifier::new(writer.lit(wind).expr());

    let effect = EffectAsset::new(
        2048,
        SpawnerSettings::rate(80.0.into()),
        writer.finish(),
    )
    .with_name("campfire_flame")
    .update(accel)
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .render(ColorOverLifetimeModifier::new(color_grad))
    .render(SizeOverLifetimeModifier {
        gradient: size_grad,
        screen_space_size: false,
    })
    .render(OrientModifier {
        mode: OrientMode::FaceCameraPosition,
        rotation: None,
    });

    effects.add(effect)
}

fn create_smoke_effect(effects: &mut ResMut<Assets<EffectAsset>>) -> Handle<EffectAsset> {
    let mut smoke_grad = Gradient::new();
    smoke_grad.add_key(0.0, Vec4::new(0.3, 0.3, 0.3, 0.0));
    smoke_grad.add_key(0.2, Vec4::new(0.3, 0.3, 0.3, 0.1));
    smoke_grad.add_key(0.5, Vec4::new(0.3, 0.3, 0.3, 0.08));
    smoke_grad.add_key(1.0, Vec4::new(0.3, 0.3, 0.3, 0.0));

    let mut smoke_size = Gradient::new();
    smoke_size.add_key(0.0, Vec3::splat(0.1));
    smoke_size.add_key(1.0, Vec3::splat(3.0));

    let mut writer = ExprWriter::new();

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(0.15).expr(),
        dimension: ShapeDimension::Surface,
    };

    let velocity = writer
        .lit(Vec3::Y * 0.4)
        .uniform(writer.lit(Vec3::new(0.1, 0.2, 0.1)));
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, velocity.expr());

    let lifetime = writer.lit(5.0).uniform(writer.lit(6.0)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let wind = Vec3::new(0.25, 0.25, 0.1); // rook buigt mee omhoog
    let accel = AccelModifier::new(writer.lit(wind).expr());

    let effect = EffectAsset::new(
        1024,
        SpawnerSettings::rate(6.0.into()),
        writer.finish(),
    )
    .with_name("campfire_smoke")
    .update(accel)
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .render(ColorOverLifetimeModifier::new(smoke_grad))
    .render(SizeOverLifetimeModifier {
        gradient: smoke_size,
        screen_space_size: false,
    })
    .render(OrientModifier {
        mode: OrientMode::FaceCameraPosition,
        rotation: None,
    });

    effects.add(effect)
}
