use bevy::prelude::*;

use bevy_oxr::DefaultXrPlugins;
use bevy_xpbd_3d::prelude::*;
use grabber::StartGrabEvent;
use input::InputState;

mod debug;
mod grabber;
mod input;
mod scene;
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
        .add_plugins(debug::DebugPlugin)
        .add_plugins(scene::ScenePlugin)
        .add_plugins(velocity_hands::VelocityHandsPlugin)
        .add_plugins(grabber::GrabberPlugin)
        .add_plugins(input::InputPlugin)
        .add_systems(Update, (spawn_cube, left_hand_grab))
        .run();
}

#[derive(PhysicsLayer)]
pub enum Layer {
    Default,
    Grabbable,
    Hand,
}

// spawn a cube when the b or y button is pressed
fn spawn_cube(
    input_state: Res<InputState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if input_state.b_button.just_pressed || input_state.y_button.just_pressed {
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
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct LeftGrabberId(pub Entity);

fn left_hand_grab(
    input_state: Res<InputState>,
    left_grabber: Res<LeftGrabberId>,
    mut grab_events_writer: EventWriter<StartGrabEvent>,
) {
    if input_state.left_trigger.value > 0.5 && input_state.left_trigger.prev_value <= 0.5 {
        grab_events_writer.send(StartGrabEvent {
            grabber: left_grabber.0,
        });
    }
}
