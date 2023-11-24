use bevy::prelude::*;
use bevy_oxr::xr_input::{
    trackers::{OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker},
    Hand,
};
use bevy_xpbd_3d::prelude::*;

use crate::{
    grabber::{Grabber, GrabberState},
    velocity_hands::PhysicsHand,
    Layer,
};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup, spawn_player));
    }
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
        ColliderDensity(1000.0),
        RigidBody::Dynamic,
        Collider::cuboid(0.1, 0.1, 0.1),
        CollisionLayers::new([Layer::Grabbable, Layer::Default], [Layer::Default]),
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

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //left hand
    let left_controller = commands
        .spawn((
            OpenXRLeftController,
            OpenXRController,
            OpenXRTracker,
            SpatialBundle::default(),
            Name::new("Left Controller"),
        ))
        .id();
    commands
        .spawn((
            PhysicsHand {
                controller: left_controller,
                follow_strength: 30.0,
                max_distance: 0.75,
                rotation_follow_strength: 30.0,
            },
            RigidBody::Dynamic,
            ColliderDensity(1000.0),
            Collider::cuboid(0.1, 0.05, 0.1),
            CollisionLayers::new([Layer::Hand, Layer::Default], [Layer::Default]),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.05, 0.1))),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 0.1, 0.0),
                ..default()
            },
            Name::new("Left Hand"),
        ))
        .with_children(|parent| {
            // grab point
            parent.spawn((
                SpatialBundle::from_transform(Transform::from_xyz(0.0, -0.05, 0.0)),
                Grabber {
                    hand: Hand::Left,
                    radius: 0.1,
                    grabbable_layer_mask: Layer::Grabbable.to_bits(),
                    state: GrabberState::Idle,
                },
                Name::new("Left Grab Point"),
            ));
        });

    //right hand
    let right_controller = commands
        .spawn((
            OpenXRRightController,
            OpenXRController,
            OpenXRTracker,
            SpatialBundle::default(),
            Name::new("Right Controller"),
        ))
        .id();
    commands
        .spawn((
            PhysicsHand {
                controller: right_controller,
                follow_strength: 30.0,
                max_distance: 0.75,
                rotation_follow_strength: 30.0,
            },
            RigidBody::Dynamic,
            ColliderDensity(1000.0),
            Collider::cuboid(0.1, 0.05, 0.1),
            CollisionLayers::new([Layer::Hand, Layer::Default], [Layer::Default]),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.05, 0.1))),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 0.1, 0.0),
                ..default()
            },
            Name::new("Right Hand"),
        ))
        .with_children(|parent| {
            // grab point
            parent.spawn((
                SpatialBundle::from_transform(Transform::from_xyz(0.0, -0.05, 0.0)),
                Grabber {
                    hand: Hand::Right,
                    radius: 0.1,
                    grabbable_layer_mask: Layer::Grabbable.to_bits(),
                    state: GrabberState::Idle,
                },
                Name::new("Right Grab Point"),
            ));
        });
}
