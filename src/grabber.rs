use bevy::prelude::*;
use bevy_oxr::xr_input::Hand;
use bevy_xpbd_3d::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum GrabberState {
    Idle,
    Grabbing(Entity),
    Grabbed(Entity),
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Grabber {
    pub hand: Hand,
    pub radius: f32,
    pub grabbable_layer_mask: u32,
    pub state: GrabberState,
}

#[derive(Event)]
pub struct StartGrabEvent {
    pub hand: Hand,
}

pub struct GrabberPlugin;

impl Plugin for GrabberPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartGrabEvent>()
            .add_systems(Update, start_grab);
    }
}

// Grabber lifecycle:
// When trigger is pressed, check for grabbable object
// If an object is found, switch to grabbing and start grabbing process
// If process is successful, switch to grabbed
// If process fails, switch to idle
// If trigger is released, release object and switch to idle

fn start_grab(
    mut commands: Commands,
    mut grab_events: EventReader<StartGrabEvent>,
    mut grabbers: Query<(&GlobalTransform, &mut Grabber)>,
    spatial_query: SpatialQuery,
) {
    for event in grab_events.read() {
        let (transform, mut grabber) = grabbers
            .iter_mut()
            .filter(|(_, grabber)| grabber.hand == event.hand)
            .next()
            .unwrap();

        let transform = transform.compute_transform();
        // Cast a sphere to find a grabbable object within the grabber's radius
        let candidates = spatial_query.shape_intersections(
            &Collider::ball(grabber.radius),
            transform.translation,
            transform.rotation,
            SpatialQueryFilter::new().with_masks_from_bits(grabber.grabbable_layer_mask),
        );

        // TODO For now just delete it to test what we have so far
        for candidate in candidates {
            commands.entity(candidate).despawn_recursive();
        }
    }
}
