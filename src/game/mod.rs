mod camera_controll;
mod database;
mod graphics_3d;
mod chess;
mod ui;

use bevy::{prelude::*, window::{WindowTheme, PresentMode}};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_shader_utils::ShaderUtilsPlugin;

use self::{chess::ChessGamePlugin, graphics_3d::Graphics3dPlugins, camera_controll::CameraControllPlugin};

pub fn run(){
    App::new()
    .add_plugins((
        DefaultPlugins.set(
        WindowPlugin {
            primary_window: Some(Window{
                title: "game".into(),
                resolution: (393., 851.).into(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: false,
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }
        ),
        DefaultPickingPlugins.build(),
        ShaderUtilsPlugin
    ))
    .add_plugins((
        ChessGamePlugin,
        CameraControllPlugin,
        Graphics3dPlugins,
    ))
    .run();
}