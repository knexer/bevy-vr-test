use bevy::prelude::*;
use bevy_oxr::xr_input::trackers::{update_open_xr_controllers, OpenXRController};
use bevy_xpbd_3d::prelude::*;

pub struct VelocityHandsPlugin;

impl Plugin for VelocityHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            move_hands
                .after(update_open_xr_controllers)
                .before(PhysicsSet::Prepare),
        );
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PhysicsHand {
    pub controller: Entity,
    pub follow_strength: f32,
    pub max_distance: f32,
    pub rotation_follow_strength: f32,
}

fn move_hands(
    mut hands: Query<
        (
            &PhysicsHand,
            &GlobalTransform,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        Without<OpenXRController>,
    >,
    controllers: Query<&GlobalTransform, With<OpenXRController>>,
) {
    for (
        hand,
        hand_global_transform,
        mut hand_local_transform,
        mut linear_velocity,
        mut angular_velocity,
    ) in hands.iter_mut()
    {
        let hand_global_transform = hand_global_transform.compute_transform();
        let controller_transform = controllers.get(hand.controller).unwrap();
        let controller_transform = controller_transform.compute_transform();

        // Position tracking
        let delta_position = controller_transform.translation - hand_global_transform.translation;
        if delta_position.length() <= hand.max_distance {
            linear_velocity.0 = (delta_position * hand.follow_strength).into();
        } else {
            hand_local_transform.translation =
                hand_global_transform.transform_point(controller_transform.translation);
        }

        // Rotation tracking
        let delta_rotation = shortest_rotation_between(
            hand_global_transform.rotation,
            controller_transform.rotation,
        );
        angular_velocity.0 = delta_rotation.to_scaled_axis() * hand.rotation_follow_strength;
    }
}

fn shortest_rotation_between(from: Quat, to: Quat) -> Quat {
    match from.dot(to) > 0.0 {
        true => to * from.inverse(),
        false => to * -from.inverse(),
    }
}
