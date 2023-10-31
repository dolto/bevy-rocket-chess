use bevy::prelude::*;

use self::camera_controll::{camera_spawn, camera_mode_switch, camera_switch_test, move_camera};

mod camera_controll;

pub struct CameraControllPlugin;

impl Plugin for CameraControllPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, 
            camera_spawn
        )
        .add_systems(Update, 
            (
                move_camera,
            )
        )
        ;
    }
}