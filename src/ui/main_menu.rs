use crate::player::PlayerSpawnCallback;
use crate::world_generation::generation_options::GenerationOptionsResource;
use bevy::app::App;
use bevy::prelude::{info, Commands, Plugin, Res, ResMut, Resource, Update};
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::egui;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Default)]
pub struct MainMenuPlugin {}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MainMenuState::default())
            .add_systems(Update, spawn_main_menu);
    }
}

#[derive(Resource)]
struct MainMenuState {
    state: MainMenuStates,
    seed: String,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self {
            state: MainMenuStates::Shown,
            seed: "Seed".into(),
        }
    }
}

enum MainMenuStates {
    Shown,
    Hidden,
}

impl MainMenuState {
    fn show_menu(&self) -> bool {
        match self.state {
            MainMenuStates::Shown => true,
            MainMenuStates::Hidden => false,
        }
    }
}

fn spawn_main_menu(
    mut menu_state: ResMut<MainMenuState>,
    mut gen_options: ResMut<GenerationOptionsResource>,
    player_spawn_callback: Res<PlayerSpawnCallback>,
    mut contexts: EguiContexts,
    mut commands: Commands,
) {
    if !menu_state.show_menu() {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("SpellHaven");

            ui.text_edit_singleline(&mut menu_state.seed);
            if ui.button("Start").clicked() {
                let mut hasher = DefaultHasher::new();
                menu_state.seed.hash(&mut hasher);
                let seed = hasher.finish();

                info!("Seed to use: {}", seed);
                *gen_options = GenerationOptionsResource::from_seed(seed);

                menu_state.state = MainMenuStates::Hidden;
                let _ = commands.run_system(player_spawn_callback.0);
            }
        });
    });
}
