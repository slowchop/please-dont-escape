use crate::input::exit_on_escape_key;
use crate::AppState;
use bevy::app::{AppExit, Events};
use bevy::prelude::*;
use bevy_egui::egui::Layout;
use bevy_egui::{egui, EguiContext};

const LOGO_ID: u64 = 0;

pub struct MainMenu;

impl Plugin for MainMenu {
    fn build(&self, app: &mut AppBuilder) {
        app
            //
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
}

fn main_menu(
    egui_context: ResMut<EguiContext>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    egui::CentralPanel::default().show(egui_context.ctx(), |ui| {
        ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            ui.add(egui::widgets::Image::new(
                egui::TextureId::User(LOGO_ID),
                [1500.0, 256.0],
            ));

            ui.label("Please Don't Escape");
            if ui.button("Solo").clicked() {
                info!("Clicked")
            }
            if ui.button("Multiplayer").clicked() {
                info!("Clicked")
            }
            if ui.button("Exit").clicked() {
                info!("Clicked");
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
