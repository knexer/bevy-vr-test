use bevy::prelude::*;

pub mod fixed_joint_2;
pub mod grabber;
pub mod velocity_tracking;

pub struct VrHandsPlugin;

impl Plugin for VrHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(grabber::GrabberPlugin)
            .add_plugins(velocity_tracking::VelocityTrackingPlugin)
            .add_plugins(fixed_joint_2::FixedJoint2Plugin);
    }
}
