
use std::f32::consts::PI;

use bevy::{prelude::*, input::mouse::{MouseButtonInput, MouseMotion, MouseWheel}};
use bevy_mod_picking::prelude::RaycastPickCamera;

const MOVE_SPEED:f32 = 0.01;
const ZOOM_SPEED:f32 = 0.05;

pub enum CameraMode {
    Camera2d,
    Camera3d
}

#[derive(Resource)]
pub struct CameraInfo{
    cameramod: CameraMode,
    location: Vec3,
    rotation: Quat,
    camera_entity: Entity,
    fov: f32,
    is_changed: bool
}

#[derive(Component)]
pub struct MainCamera;

pub fn camera_spawn(
    mut commands: Commands,
){
    let location = Transform::from_xyz(
        0.,6.,13.
    ).looking_at(Vec3::ZERO, Vec3::Y);
    let camera = commands.spawn((
        Camera3dBundle{
            transform: location.clone(),
            ..default()
        },
        RaycastPickCamera::default(),
        MainCamera,
    )).id();
    commands.insert_resource(CameraInfo{
        cameramod: CameraMode::Camera3d,
        location: location.translation,
        rotation: location.rotation.into(),
        camera_entity: camera,
        fov: PI/4.,
        is_changed: false
    });
}

pub fn move_camera(
    mut commands: Commands,
    mut res_camerainfo: ResMut<CameraInfo>,
    mut query_camera: Query<&mut Transform, With<MainCamera>>,
    mut events_input: EventReader<MouseButtonInput>,
    mut events_move: EventReader<MouseMotion>,
    mut event_wheel: EventReader<MouseWheel>,
    mut is_move: Local<bool>
){
    for ev in events_input.iter(){
        if ev.button == MouseButton::Middle{
            match ev.state {
                bevy::input::ButtonState::Pressed => {
                    *is_move = true;
                },
                bevy::input::ButtonState::Released => {
                    *is_move = false;
                },
            }
        }
    }
    for ev in event_wheel.iter(){
        res_camerainfo.fov = (PI/3.).min((PI / 15.).max(res_camerainfo.fov + ev.y * ZOOM_SPEED * -1.));
        //println!("fov:{}, min:{}", res_camerainfo.fov, (PI/3.));
        commands.entity(res_camerainfo.camera_entity).insert(
            Projection::Perspective(PerspectiveProjection { fov: res_camerainfo.fov, ..Default::default()})
        );
    }
    for ev in events_move.iter(){
        if *is_move{
            let mut trans = query_camera.get_single_mut().unwrap();
            let mut translation = Transform::from_xyz(
                trans.translation.x + ev.delta.x * MOVE_SPEED * -1. * res_camerainfo.fov, 
                trans.translation.y, 
                trans.translation.z + ev.delta.y * MOVE_SPEED * -1. * res_camerainfo.fov) ;
            translation.rotation = trans.rotation;
            *trans = translation;
        }
    }


}

pub fn camera_mode_switch(
    mut commands: Commands,
    time: Res<Time>,
    mut res_camerainfo: ResMut<CameraInfo>,
){
    if res_camerainfo.is_changed{
        match res_camerainfo.cameramod {
            CameraMode::Camera2d => {
                if res_camerainfo.fov <= PI/4.{
                    res_camerainfo.cameramod = CameraMode::Camera3d;
                    res_camerainfo.is_changed = false;
                }else{
                    let delay = time.delta_seconds();
                    res_camerainfo.fov -= delay * 10.;
                }
                commands.entity(res_camerainfo.camera_entity).insert(
                    Projection::Perspective(PerspectiveProjection { fov: res_camerainfo.fov, ..Default::default()})
                );
            },
            CameraMode::Camera3d => {
                if res_camerainfo.fov >= PI{
                    res_camerainfo.cameramod = CameraMode::Camera2d;
                    res_camerainfo.is_changed = false;
                    commands.entity(res_camerainfo.camera_entity).insert(
                        Projection::Orthographic(OrthographicProjection::default())
                    );
                    return
                }
                let delay = time.delta_seconds();
                res_camerainfo.fov += delay * 10.;
                commands.entity(res_camerainfo.camera_entity).insert(
                    Projection::Perspective(PerspectiveProjection { fov: res_camerainfo.fov, ..Default::default()})
                );
            },
        }
    }
}

pub fn camera_switch_test(
    mut res_camerainfo: ResMut<CameraInfo>,
    mut events: EventReader<MouseButtonInput>
){
    for ev in events.iter(){
        if ev.button == MouseButton::Middle{
            res_camerainfo.is_changed = true;
        }
    }
}
