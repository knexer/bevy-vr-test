use bevy::prelude::*;
use bevy_oxr::xr_input::trackers::update_open_xr_controllers;
use bevy_xpbd_3d::prelude::*;

pub struct VelocityTrackingPlugin;

impl Plugin for VelocityTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            velocity_track
                // TODO make this configurable in the plugin
                .after(update_open_xr_controllers)
                .before(PhysicsSet::Prepare),
        );
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct VelocityTracked {
    pub follow_target: Entity,
    pub follow_strength: f32,
    pub max_distance: f32,
    pub rotation_follow_strength: f32,
}

fn velocity_track(
    mut tracked_objects: Query<(
        &VelocityTracked,
        &GlobalTransform,
        &mut Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
    )>,
    targets: Query<&GlobalTransform>,
) {
    for (
        track_config,
        global_transform,
        mut local_transform,
        mut linear_velocity,
        mut angular_velocity,
    ) in tracked_objects.iter_mut()
    {
        let global_transform = global_transform.compute_transform();
        let target_transform = targets.get(track_config.follow_target).unwrap();
        let target_transform = target_transform.compute_transform();

        // Position tracking
        let delta_position = target_transform.translation - global_transform.translation;
        if delta_position.length() <= track_config.max_distance {
            linear_velocity.0 = (delta_position * track_config.follow_strength).into();
        } else {
            local_transform.translation =
                global_transform.transform_point(target_transform.translation);
        }

        // Rotation tracking
        let delta_rotation =
            shortest_rotation_between(global_transform.rotation, target_transform.rotation);
        angular_velocity.0 =
            delta_rotation.to_scaled_axis() * track_config.rotation_follow_strength;
    }
}

fn shortest_rotation_between(from: Quat, to: Quat) -> Quat {
    match from.dot(to) > 0.0 {
        true => to * from.inverse(),
        false => to * -from.inverse(),
    }
}
