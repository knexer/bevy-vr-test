use bevy::{ecs::system::SystemParam, gltf::Gltf, prelude::*};
use bevy_oxr::xr_input::{
    trackers::{OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker},
    Hand,
};
use bevy_xpbd_3d::prelude::*;

use crate::{
    vr_hands::grabber::{Grabbable, Grabber, GrabberState},
    vr_hands::velocity_tracking::VelocityTracked,
    Layer,
};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AssetState>()
            .add_systems(Startup, (setup, spawn_player, load_gltf))
            .add_systems(Update, check_loaded.run_if(in_state(AssetState::Loading)))
            .add_systems(OnEnter(AssetState::Loaded), spawn_test_gltf);
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub enum AssetState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Resource)]
struct AssetLibHandle(Handle<Gltf>);

#[derive(SystemParam)]
pub struct AssetLib<'w> {
    handle: Res<'w, AssetLibHandle>,
    asset_server: Res<'w, Assets<Gltf>>,
}

impl AssetLib<'_> {
    pub fn scene(&self, name: &str) -> Handle<Scene> {
        let gltf = self.asset_server.get(self.handle.0.clone()).unwrap();
        gltf.named_scenes[name].clone()
    }

    pub fn is_loaded(&self) -> bool {
        self.asset_server.get(self.handle.0.clone()).is_some()
    }
}

fn load_gltf(mut commands: Commands, ass: Res<AssetServer>) {
    let gltf = ass.load("gen/test.gltf");
    commands.insert_resource(AssetLibHandle(gltf));
}

fn check_loaded(assets: AssetLib, mut asset_state: ResMut<NextState<AssetState>>) {
    if assets.is_loaded() {
        asset_state.set(AssetState::Loaded);
    }
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
        CollisionLayers::new([Layer::Default], [Layer::Default]),
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
        CollisionLayers::new([Layer::Grabbable, Layer::Default], [Layer::Default]),
        Grabbable { grabbed_by: vec![] },
        Name::new("Grabbable Cube"),
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

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //left hand
    let left_controller = commands
        .spawn((
            OpenXRLeftController,
            OpenXRController,
            OpenXRTracker,
            SpatialBundle::default(),
            Name::new("Left Controller"),
        ))
        .id();
    commands
        .spawn((
            VelocityTracked {
                follow_target: left_controller,
                follow_strength: 30.0,
                max_distance: 0.75,
                rotation_follow_strength: 30.0,
            },
            RigidBody::Dynamic,
            ColliderDensity(1000.0),
            Collider::cuboid(0.1, 0.05, 0.1),
            CollisionLayers::new([Layer::Hand, Layer::Default], [Layer::Default]),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.05, 0.1))),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 0.1, 0.0),
                ..default()
            },
            Name::new("Left Hand"),
        ))
        .with_children(|parent| {
            // grab point
            parent.spawn((
                SpatialBundle::from_transform(Transform::from_xyz(0.0, -0.05, 0.0)),
                Grabber {
                    hand: Hand::Left,
                    search_radius: 0.1,
                    grab_tolerance: 0.02,
                    grabbable_layer_mask: Layer::Grabbable.to_bits(),
                    state: GrabberState::Idle,
                },
                Name::new("Left Grab Point"),
            ));
        });

    //right hand
    let right_controller = commands
        .spawn((
            OpenXRRightController,
            OpenXRController,
            OpenXRTracker,
            SpatialBundle::default(),
            Name::new("Right Controller"),
        ))
        .id();
    commands
        .spawn((
            VelocityTracked {
                follow_target: right_controller,
                follow_strength: 30.0,
                max_distance: 0.75,
                rotation_follow_strength: 30.0,
            },
            RigidBody::Dynamic,
            ColliderDensity(1000.0),
            Collider::cuboid(0.1, 0.05, 0.1),
            CollisionLayers::new([Layer::Hand, Layer::Default], [Layer::Default]),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.05, 0.1))),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 0.1, 0.0),
                ..default()
            },
            Name::new("Right Hand"),
        ))
        .with_children(|parent| {
            // grab point
            parent.spawn((
                SpatialBundle::from_transform(Transform::from_xyz(0.0, -0.05, 0.0)),
                Grabber {
                    hand: Hand::Right,
                    search_radius: 0.1,
                    grab_tolerance: 0.02,
                    grabbable_layer_mask: Layer::Grabbable.to_bits(),
                    state: GrabberState::Idle,
                },
                Name::new("Right Grab Point"),
            ));
        });
}

fn spawn_test_gltf(mut commands: Commands, asset_lib: AssetLib) {
    commands.spawn(SceneBundle {
        scene: asset_lib.scene("ship"),
        transform: Transform::from_xyz(0.0, 0.25, 0.0),
        ..default()
    });

    commands.spawn(SceneBundle {
        scene: asset_lib.scene("hamster"),
        transform: Transform::from_xyz(0.25, 0.25, 0.0),
        ..default()
    });
}
