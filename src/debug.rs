use bevy::prelude::*;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_oxr::input::XrInput;
use bevy_oxr::resources::{XrFrameState, XrInstance, XrSession};
use bevy_oxr::xr_input::debug_gizmos::OpenXrDebugRenderer;
use bevy_oxr::xr_input::oculus_touch::OculusController;
use bevy_oxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};
use bevy_xpbd_3d::plugins::setup::{Physics, PhysicsTime};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpenXrDebugRenderer)
            .add_plugins(LogDiagnosticsPlugin::default())
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Update, (proto_locomotion, toggle_physics))
            .insert_resource(PrototypeLocomotionConfig::default());
    }
}

// Toggle physics when the A or X buttons are pressed on each touch controller.
fn toggle_physics(
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    mut physics_time: ResMut<Time<Physics>>,
    mut currently_pressed: Local<bool>,
) {
    // magic code to get the controller
    let frame_state = *frame_state.lock().unwrap();
    let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);

    if controller.a_button() || controller.x_button() {
        if *currently_pressed {
            return;
        }
        *currently_pressed = true;
        // Toggle physics
        if physics_time.is_paused() {
            physics_time.unpause();
        } else {
            physics_time.pause();
        }
    } else {
        *currently_pressed = false;
    }
}
