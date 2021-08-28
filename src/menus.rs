use crate::input::exit_on_escape_key;
use crate::AppState;
use bevy::app::{AppExit, Events};
use bevy::prelude::*;
use bevy_egui::egui::{FontDefinitions, FontFamily, Layout};
use bevy_egui::{egui, EguiContext};

const LOGO_ID: u64 = 0;

pub struct MainMenu;

impl Plugin for MainMenu {
    fn build(&self, app: &mut AppBuilder) {
        app
            //
            // .add_startup_system(setup.system().after(EguiSystem::BeginFrame))
            .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(setup.system()))
            .add_system_set(
                SystemSet::on_update(AppState::MainMenu)
                    .with_system(exit_on_escape_key.system())
                    .with_system(main_menu.system()),
            );
    }
}

pub fn setup(mut egui_context: ResMut<EguiContext>, assets: Res<AssetServer>) {
    let texture_handle = assets.load("menus/logo.png");
    egui_context.set_egui_texture(LOGO_ID, texture_handle);

    let mut fonts = FontDefinitions::default();
    fonts
        .family_and_size
        .insert(egui::TextStyle::Button, (FontFamily::Proportional, 80.0));
    egui_context.ctx().set_fonts(fonts);

    let mut style: egui::Style = (*egui_context.ctx().style()).clone();
    style.spacing.item_spacing.x = 20.0;
    style.spacing.item_spacing.y = 20.0;
    style.spacing.button_padding.x = 20.0;
    style.spacing.button_padding.y = 20.0;
    egui_context.ctx().set_style(style);
}

fn main_menu(
    egui_context: ResMut<EguiContext>,
    mut state: ResMut<State<AppState>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    egui::CentralPanel::default().show(egui_context.ctx(), |ui| {
        ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            ui.add(egui::widgets::Image::new(
                egui::TextureId::User(LOGO_ID),
                // [1171.0, 553.0],
                [1500.0, 256.0],
            ));

            if ui.button("Solo").clicked() {
                state.set(AppState::InGame).expect("Could not set state to InGame.");
            }

            if ui.button("Editor").clicked() {
                state.set(AppState::Editor).expect("Could not set state to Editor.");
            }

            if ui.button("Exit").clicked() {
                app_exit_events.send(AppExit);
            }
        });

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add(
                egui::Hyperlink::new("https://slowchop.itch.io/")
                    .text("Created by Slowchop Studios"),
            );
        });
    });
}
