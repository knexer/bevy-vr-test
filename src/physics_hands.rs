use bevy::prelude::*;
use bevy_oxr::xr_input::trackers::{
    update_open_xr_controllers,
    OpenXRController,
    OpenXRLeftController,
    OpenXRRightController,
    OpenXRTracker,
    // OpenXRTrackingRoot,
};
use bevy_xpbd_3d::prelude::*;

// A physics-based grabbing and interaction plugin for bevy_openxr and bevy_xpbd_3d.
pub struct PhysicsHandsPlugin;

impl Plugin for PhysicsHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, move_hands.after(update_open_xr_controllers));
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicsHand {
    controller: Entity,
}

fn setup(
    // tracking_root: Query<Entity, With<OpenXRTrackingRoot>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let tracking_root = tracking_root.single();
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
    // commands
    //     .entity(tracking_root)
    //     .push_children(&[left_controller]);
    commands.spawn((
        PhysicsHand {
            controller: left_controller,
        },
        RigidBody::Kinematic,
        Collider::cuboid(0.1, 0.1, 0.1),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        ExternalForce::default(),
        Name::new("Left Hand"),
    ));

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
    // commands
    //     .entity(tracking_root)
    //     .push_children(&[right_controller]);
    commands.spawn((
        PhysicsHand {
            controller: right_controller,
        },
        RigidBody::Kinematic,
        Collider::cuboid(0.1, 0.1, 0.1),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        ExternalForce::default(),
        Name::new("Right Hand"),
    ));
}

fn move_hands(
    mut hands: Query<(&PhysicsHand, &mut Transform), Without<OpenXRController>>,
    controllers: Query<&GlobalTransform, With<OpenXRController>>,
) {
    for (hand, mut transform) in hands.iter_mut() {
        let controller = controllers.get(hand.controller).unwrap();
        // TODO: replace with PID controller and forces
        transform.translation = controller.translation();
    }
}

// Brainstorming: what's the strategy here?
// First question - what are the requirements?
// We need to support physical interactions with objects in the scene. Generally, this means
// applying forces. These forces can be applied in two ways:
// 1. By pushing the hand against the object, e.g. punching/slapping
// 2. By grabbing the object, which creates a joint between the hand and the object

// For this to make sense, the hand itself must be a physics object, subject to forces from the
// environment. Thus the hand must be dynamic rigidbody, and we cannot control its pose directly.
// Instead, we must apply forces to it to bring it towards the desired pose.

// So in summary we'll have the following structure:
// 1. A controller entity, which is a kinematic rigidbody with no collider, user-controlled.
// 2. A hand entity, which is a dynamic rigidbody with a collider, trailing behind the controller.
