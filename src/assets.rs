use bevy::{ecs::system::SystemParam, gltf::Gltf, prelude::*};
use bevy_gltf_components::{process_loaded_scenes, track_new_gltf, ComponentsFromGltfPlugin};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentsFromGltfPlugin)
            .add_state::<AssetState>()
            .add_systems(Startup, load_gltf)
            .add_systems(Update, check_loaded.run_if(in_state(AssetState::Loading)))
            .add_systems(
                OnExit(AssetState::Loading),
                (track_new_gltf, process_loaded_scenes).chain(),
            );
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
