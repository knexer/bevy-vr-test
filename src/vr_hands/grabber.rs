use bevy::prelude::*;
use bevy_oxr::xr_input::Hand;
use bevy_xpbd_3d::{
    plugins::collision::contact_query::{closest_points, ClosestPoints},
    prelude::*,
};

use super::fixed_joint_2::FixedJoint2;

#[derive(Debug, Clone, Copy)]
pub enum GrabberState {
    Idle,
    Grabbing(Option<(Entity, Vec3)>),
    Grabbed(Entity, Entity),
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Grabber {
    pub hand: Hand,
    pub search_radius: f32,
    pub grab_tolerance: f32,
    pub grabbable_layer_mask: u32,
    pub state: GrabberState,
}

#[derive(Component, Debug, Clone)]
pub struct Grabbable {
    pub grabbed_by: Vec<Entity>,
}

#[derive(Event)]
pub struct StartGrabEvent {
    pub hand: Hand,
}

#[derive(Event)]
pub struct EndGrabEvent {
    pub hand: Hand,
}

pub struct GrabberPlugin;

impl Plugin for GrabberPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartGrabEvent>()
            .add_event::<EndGrabEvent>()
            .add_systems(
                Update,
                (
                    (handle_grab_start, handle_grab_end),
                    update_grabbing_grabbers,
                    show_grab_point_gizmos,
                    grab_when_close_enough,
                )
                    .chain(),
            );
    }
}

// Grabber lifecycle:
// When trigger is pressed, check for grabbable object
// If an object is found, switch to grabbing and start grabbing process
// If process is successful, switch to grabbed
// If process fails, switch to idle
// If trigger is released, release object and switch to idle

fn handle_grab_start(
    mut grab_events: EventReader<StartGrabEvent>,
    mut grabbers: Query<&mut Grabber>,
) {
    for event in grab_events.read() {
        let mut grabber = grabbers
            .iter_mut()
            .filter(|grabber| grabber.hand == event.hand)
            .next()
            .unwrap();

        assert!(
            matches!(grabber.state, GrabberState::Idle),
            "Started grab, but grabber was already grabbing."
        );
        grabber.state = GrabberState::Grabbing(None);
    }
}

fn update_grabbing_grabbers(
    mut grabbers: Query<(Entity, &GlobalTransform, &mut Grabber)>,
    mut grabbables: Query<&mut Grabbable>,
    spatial_query: SpatialQuery,
    colliders: Query<(&Collider, &GlobalTransform)>,
    parents: Query<&Parent>,
) {
    for (grabber_entity, transform, mut grabber) in grabbers.iter_mut() {
        if !matches!(grabber.state, GrabberState::Grabbing(_)) {
            continue;
        }

        let transform = transform.compute_transform();
        // Cast a sphere to find a grabbable object within the grabber's radius
        let candidates = spatial_query.shape_intersections(
            &Collider::ball(grabber.search_radius),
            transform.translation,
            transform.rotation,
            SpatialQueryFilter::new().with_masks_from_bits(grabber.grabbable_layer_mask),
        );

        let distance_to = |candidate: &Entity| -> (Entity, f32, Vec3) {
            let test_collider = Collider::ball(0.0);
            let (cand_collider, cand_transform) = colliders.get(*candidate).unwrap();
            let cand_transform = cand_transform.compute_transform();
            let closest_points = closest_points(
                &test_collider,
                transform.translation,
                transform.rotation,
                &cand_collider,
                cand_transform.translation,
                cand_transform.rotation,
                grabber.search_radius,
            )
            .unwrap();

            let closest_point = match closest_points {
                ClosestPoints::Intersecting => transform.translation,
                ClosestPoints::WithinMargin(_, p2) => p2,
                ClosestPoints::OutsideMargin => panic!("closest_points returned OutsideMargin"),
            };

            let sq_distance = transform.translation.distance_squared(closest_point);

            (*candidate, sq_distance, closest_point)
        };

        if let Some((mut closest_candidate, _, closest_point)) = candidates
            .iter()
            .map(distance_to)
            .min_by(|(_, d1, _), (_, d2, _)| d1.partial_cmp(d2).unwrap())
        {
            // Walk up the hierarchy to find the closest grabbable parent
            while !grabbables.get(closest_candidate).is_ok() {
                closest_candidate = parents.get(closest_candidate).unwrap().get();
            }

            for mut grabbable in grabbables.iter_mut() {
                grabbable.grabbed_by.retain(|e| *e != grabber_entity);
            }

            grabbables
                .get_mut(closest_candidate)
                .unwrap()
                .grabbed_by
                .push(grabber_entity);

            println!("Grabbing {:?}", closest_candidate);
            grabber.state = GrabberState::Grabbing(Some((closest_candidate, closest_point)));
        } else {
            for mut grabbable in grabbables.iter_mut() {
                grabbable.grabbed_by.retain(|e| *e != grabber_entity);
            }
            grabber.state = GrabberState::Grabbing(None);
        }
    }
}

fn grab_when_close_enough(
    mut commands: Commands,
    mut grabbers: Query<(Entity, &GlobalTransform, &mut Grabber)>,
    transforms: Query<&GlobalTransform>,
    children: Query<&Parent>,
    rbs: Query<(), With<RigidBody>>,
) {
    for (grabber_entity, grabber_transform, mut grabber) in grabbers.iter_mut() {
        if let GrabberState::Grabbing(Some((grabbed_entity, closest_point))) = grabber.state {
            if grabber_transform.translation().distance(closest_point) < grabber.grab_tolerance {
                let joint_config_for = |mut entity: Entity| -> (Entity, Vec3, Quat) {
                    while !rbs.get(entity).is_ok() {
                        entity = children.get(entity).unwrap().get();
                    }
                    let transform = transforms.get(entity).unwrap();
                    let local_anchor = transform.affine().inverse().transform_point(closest_point);
                    let rotation = transform.compute_transform().rotation;
                    (entity, local_anchor, rotation)
                };

                let (grabber_entity, grabber_local_anchor, grabber_rotation) =
                    joint_config_for(grabber_entity);
                let (grabbed_entity, grabbed_local_anchor, grabbed_rotation) =
                    joint_config_for(grabbed_entity);

                // grabber_rotation * x = grabbed_rotation
                // x = grabber_rotation.inverse() * grabbed_rotation
                let rotation_offset = grabber_rotation.inverse() * grabbed_rotation;

                // Create a joint between the grabber and the grabbed object
                let joint_id = commands
                    .spawn((
                        FixedJoint2::new(grabber_entity, grabbed_entity)
                            .with_local_anchor_1(grabber_local_anchor)
                            .with_local_anchor_2(grabbed_local_anchor)
                            .with_rotation_offset(rotation_offset.into()),
                        Name::new("Grab Joint"),
                    ))
                    .id();

                println!("Grabbed {:?}", grabbed_entity);
                grabber.state = GrabberState::Grabbed(grabbed_entity, joint_id);
            }
        }
    }
}

fn handle_grab_end(
    mut commands: Commands,
    mut grab_events: EventReader<EndGrabEvent>,
    mut grabbers: Query<(Entity, &mut Grabber)>,
    mut grabbable: Query<&mut Grabbable>,
) {
    for event in grab_events.read() {
        let (grabber_entity, mut grabber) = grabbers
            .iter_mut()
            .filter(|(_, grabber)| grabber.hand == event.hand)
            .next()
            .unwrap();

        assert!(
            !matches!(grabber.state, GrabberState::Idle),
            "Ended grab, but grabber was not grabbing."
        );

        let grabbed_entity = match grabber.state {
            GrabberState::Grabbed(entity, joint) => {
                commands.entity(joint).despawn_recursive();
                Some(entity)
            }
            GrabberState::Grabbing(Some((entity, _))) => Some(entity),
            GrabberState::Grabbing(None) => None,
            GrabberState::Idle => continue,
        };

        grabber.state = GrabberState::Idle;
        if let Some(grabbed_entity) = grabbed_entity {
            println!("Releasing {:?}", grabbed_entity);
            let mut grabbable = grabbable.get_mut(grabbed_entity).unwrap();
            grabbable.grabbed_by.retain(|e| *e != grabber_entity);
        }
    }
}

// Show a gizmo for each grab point
fn show_grab_point_gizmos(grabbers: Query<&Grabber>, mut gizmos: Gizmos) {
    for grabber in grabbers.iter() {
        if let GrabberState::Grabbing(Some((_, point))) = grabber.state {
            gizmos.sphere(point, Quat::IDENTITY, 0.02, Color::rgb(0.0, 1.0, 0.0));
        }
    }
}
