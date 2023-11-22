use bevy::prelude::*;
use bevy_oxr::xr_input::trackers::{
    update_open_xr_controllers, OpenXRController, OpenXRLeftController, OpenXRRightController,
    OpenXRTracker,
};
use bevy_xpbd_3d::prelude::*;

pub struct VelocityHandsPlugin;

impl Plugin for VelocityHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            move_hands
                .after(update_open_xr_controllers)
                .before(PhysicsSet::Prepare),
        );
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PhysicsHand {
    controller: Entity,
    follow_strength: f32,
    max_distance: f32,
}

fn setup(
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
    commands.spawn((
        PhysicsHand {
            controller: left_controller,
            follow_strength: 30.0,
            max_distance: 0.75,
        },
        RigidBody::Dynamic,
        ColliderDensity(0.0),
        Mass(1.0),
        Collider::cuboid(0.1, 0.1, 0.1),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.1, 0.0),
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
    commands.spawn((
        PhysicsHand {
            controller: right_controller,
            follow_strength: 30.0,
            max_distance: 0.75,
        },
        RigidBody::Dynamic,
        ColliderDensity(0.0),
        Mass(1.0),
        Collider::cuboid(0.1, 0.1, 0.1),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.1, 0.0),
            ..default()
        },
        ExternalForce::default(),
        Name::new("Right Hand"),
    ));
}

fn move_hands(
    mut hands: Query<
        (
            &PhysicsHand,
            &GlobalTransform,
            &mut Transform,
            &mut LinearVelocity,
        ),
        Without<OpenXRController>,
    >,
    controllers: Query<&GlobalTransform, With<OpenXRController>>,
) {
    for (hand, hand_global_transform, mut hand_local_transform, mut velocity) in hands.iter_mut() {
        let controller_transform = controllers.get(hand.controller).unwrap();
        let delta_position =
            controller_transform.translation() - hand_global_transform.translation();
        if delta_position.length() > hand.max_distance {
            hand_local_transform.translation =
                hand_global_transform.transform_point(controller_transform.translation());
            continue;
        }
        velocity.0 = delta_position * hand.follow_strength;
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
