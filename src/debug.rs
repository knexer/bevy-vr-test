use bevy::prelude::*;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_oxr::xr_input::debug_gizmos::OpenXrDebugRenderer;
use bevy_oxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpenXrDebugRenderer)
            .add_plugins(LogDiagnosticsPlugin::default())
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Update, proto_locomotion)
            .insert_resource(PrototypeLocomotionConfig::default());
    }
}
