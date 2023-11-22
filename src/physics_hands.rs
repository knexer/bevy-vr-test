use bevy::{math::DVec3, prelude::*};
use bevy_oxr::xr_input::trackers::{
    update_open_xr_controllers, OpenXRController, OpenXRLeftController, OpenXRRightController,
    OpenXRTracker,
};
use bevy_xpbd_3d::prelude::*;

// A physics-based grabbing and interaction plugin for bevy_openxr and bevy_xpbd_3d.
pub struct PhysicsHandsPlugin;

impl Plugin for PhysicsHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            move_hands
                .after(update_open_xr_controllers)
                .before(PhysicsSet::Prepare),
        );
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicsHand {
    controller: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
struct PIDController {
    p: f32,
    i: f32,
    d: f32,
    max_force: f32,
    max_integral_error: f32,
    max_error: f32,
    integral_error: Vec3,
    last_error: Vec3,
}

impl PIDController {
    fn new(
        p: f32,
        i: f32,
        d: f32,
        max_force: f32,
        max_integral_error: f32,
        max_error: f32,
    ) -> Self {
        Self {
            p,
            i,
            d,
            max_force,
            max_integral_error,
            max_error,
            integral_error: Vec3::ZERO,
            last_error: Vec3::ZERO,
        }
    }

    fn update(&mut self, error: Vec3, dt: f32) -> Vec3 {
        if dt < 1e-6 {
            return Vec3::ZERO;
        }
        let p = self.p * error;
        self.integral_error =
            (self.integral_error + error * dt).clamp_length_max(self.max_integral_error);
        let i = self.i * self.integral_error;
        let d = self.d * (error - self.last_error) / dt;
        self.last_error = error;
        (p + i + d).clamp_length_max(self.max_force)
    }

    fn reset(&mut self) {
        self.integral_error = Vec3::ZERO;
        self.last_error = Vec3::ZERO;
    }
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
        PIDController::new(100.0, 10.0, 20.0, 20.0, 2.0, 0.25),
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
        },
        RigidBody::Dynamic,
        ColliderDensity(0.0),
        Mass(10.0),
        Collider::cuboid(0.1, 0.1, 0.1),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.1, 0.0),
            ..default()
        },
        PIDController::new(1000.0, 100.0, 200.0, 200.0, 4.0, 0.25),
        ExternalForce::default(),
        Name::new("Right Hand"),
    ));
}

fn move_hands(
    mut hands: Query<
        (
            &PhysicsHand,
            &GlobalTransform,
            &mut PIDController,
            &mut ExternalForce,
            &mut Transform,
            // &mut Position,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        Without<OpenXRController>,
    >,
    controllers: Query<&GlobalTransform, With<OpenXRController>>,
    time: Res<Time>,
) {
    for (
        hand,
        global_transform,
        mut pid_controller,
        mut force,
        mut local_transform,
        // mut position,
        mut linear_velocity,
        mut angular_velocity,
    ) in hands.iter_mut()
    {
        let controller = controllers.get(hand.controller).unwrap();
        let error = controller.translation() - global_transform.translation();
        println!("error: {:?}", error);
        if error.length() > pid_controller.max_error || !global_transform.translation().is_finite()
        {
            println!(
                "error too large, resetting to {:?}",
                controller.translation()
            );
            pid_controller.reset();
            force.set_force(DVec3::ZERO);
            local_transform.translation = controller.translation();
            // position.0 = controller.translation();
            linear_velocity.0 = DVec3::ZERO;
            angular_velocity.0 = DVec3::ZERO;
        } else {
            let applied_force = pid_controller.update(error, time.delta_seconds());
            force.set_force(applied_force.into());
            println!("force: {:?}", applied_force);
        }
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
