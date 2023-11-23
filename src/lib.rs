use bevy::prelude::*;

use bevy_oxr::{
    input::XrInput,
    resources::{XrFrameState, XrInstance, XrSession},
    xr_input::oculus_touch::OculusController,
    DefaultXrPlugins,
};
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
        .insert_resource(SleepingThreshold {
            linear: -0.01,
            angular: -0.01,
        })
        .insert_resource(Gravity(Vec3::new(0.0, -9.8, 0.0)))
        .add_plugins(DebugPlugin)
        .add_plugins(VelocityHandsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_cube)
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
        ColliderDensity(1000.0),
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

// spawn a cube when the b or y button is pressed
fn spawn_cube(
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut currently_pressed: Local<bool>,
) {
    // magic code to get the controller
    let frame_state = *frame_state.lock().unwrap();
    let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);

    if controller.b_button() || controller.y_button() {
        if *currently_pressed {
            return;
        }
        *currently_pressed = true;
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
        ));
    } else {
        *currently_pressed = false;
    }
}
