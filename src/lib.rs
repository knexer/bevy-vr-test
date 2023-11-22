use bevy::prelude::*;

use bevy_oxr::DefaultXrPlugins;
use bevy_xpbd_3d::prelude::*;
use debug::DebugPlugin;
use velocity_hands::VelocityHandsPlugin;

mod debug;
mod physics_hands;
mod velocity_hands;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultXrPlugins)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(SubstepCount(30))
        .insert_resource(Gravity(Vec3::ZERO))
        .add_plugins(DebugPlugin)
        .add_plugins(VelocityHandsPlugin)
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(5.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0.0, -0.5, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::halfspace(Vec3::Y),
    ));
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(0.1, 0.1, 0.1),
    ));
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.8, 0.0, 0.0).into()),
            transform: Transform::from_xyz(0.0, 0.5, 1.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.1, 0.1, 0.1),
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
