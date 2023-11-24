use bevy::prelude::*;
use bevy_oxr::xr_input::Hand;
use bevy_xpbd_3d::{
    plugins::collision::contact_query::{closest_points, ClosestPoints},
    prelude::*,
};

#[derive(Debug, Clone, Copy)]
pub enum GrabberState {
    Idle,
    Grabbing(Option<(Entity, Vec3)>),
    Grabbed(Entity),
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Grabber {
    pub hand: Hand,
    pub radius: f32,
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
    mut grabbable: Query<(Entity, &mut Grabbable)>,
    spatial_query: SpatialQuery,
    colliders: Query<(&Collider, &GlobalTransform)>,
) {
    for (grabber_entity, transform, mut grabber) in grabbers.iter_mut() {
        if !matches!(grabber.state, GrabberState::Grabbing(_)) {
            continue;
        }

        let transform = transform.compute_transform();
        // Cast a sphere to find a grabbable object within the grabber's radius
        let candidates = spatial_query.shape_intersections(
            &Collider::ball(grabber.radius),
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
                grabber.radius,
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

        if let Some((closest_candidate, _, closest_point)) = candidates
            .iter()
            .map(distance_to)
            .min_by(|(_, d1, _), (_, d2, _)| d1.partial_cmp(d2).unwrap())
        {
            for (entity, mut grabbable) in grabbable.iter_mut() {
                grabbable.grabbed_by.retain(|e| *e != grabber_entity);
                if entity == closest_candidate {
                    grabbable.grabbed_by.push(grabber_entity);
                    break;
                }
            }
            grabber.state = GrabberState::Grabbing(Some((closest_candidate, closest_point)));
        } else {
            for (_, mut grabbable) in grabbable.iter_mut() {
                grabbable.grabbed_by.retain(|e| *e != grabber_entity);
            }
            grabber.state = GrabberState::Grabbing(None);
        }
    }
}

fn handle_grab_end(
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
            GrabberState::Grabbed(entity) | GrabberState::Grabbing(Some((entity, _))) => {
                Some(entity)
            }
            GrabberState::Grabbing(None) => None,
            GrabberState::Idle => continue,
        };

        grabber.state = GrabberState::Idle;
        if let Some(grabbed_entity) = grabbed_entity {
            if let Ok(mut grabbable) = grabbable.get_mut(grabbed_entity) {
                grabbable.grabbed_by.retain(|e| *e != grabber_entity);
            }
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
