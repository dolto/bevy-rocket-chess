use std::{collections::{HashMap, HashSet}, f32::consts::PI};

use bevy::{prelude::*, input::mouse::MouseButtonInput};
use bevy_mod_picking::prelude::{On, Click, Pointer, Listener, PointerButton};
use hexx::Hex;

use crate::game::graphics_3d::honeycomb::{HEX_SIZE, MAP_RADIUS, Map, Honeycomb};

use super::pawn::{SpawnAnimToggle, OtherSpawn, SpawnCategory};

#[derive(Resource)]
pub struct BishopRes{
    //scene: Handle<Scene>,
    mesh: Handle<Mesh>,
    bishop_list: HashMap<Hex, Entity>,
    blue_bishop_list: HashSet<Hex>,
    red_bishop_list: HashSet<Hex>,
    spawn_animation: (Handle<AnimationClip>, Name),
    idle_animation: (Handle<AnimationClip>, Name),
    control_target: Option<Hex>
}

#[derive(Component)]
pub struct Bishop{
    blue_team: bool,
    pos: Hex,
    is_atack: bool
}

#[derive(Component)]
pub struct BishopAttack{
    attack_target: Hex,
    speed: f32,
    move_time: f32
}

#[derive(Component)]
pub struct Attacking;

pub fn setup_asset_bishop(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut animations: ResMut<Assets<AnimationClip>>,
){
    let spawn_anim = Name::new("bishop_spawn");
    let mut spawn_animation = AnimationClip::default();
    spawn_animation.add_curve_to_path(
        EntityPath{parts: vec![spawn_anim.clone()]}, 
        VariableCurve { 
            keyframe_timestamps: vec![0.,0.6,1.2], 
            keyframes: Keyframes::Translation(
                vec![
                    Vec3::new(0., HEX_SIZE * -2., 0.),
                    Vec3::new(0., HEX_SIZE * 1.2, 0.),
                    Vec3::new(0., HEX_SIZE / 3., 0.)
                ]
            ) 
        }
    );
    let idle_anim = Name::new("bishop_idle");
    let mut idle_animation = AnimationClip::default();
    idle_animation.add_curve_to_path(
        EntityPath { parts: vec![idle_anim.clone()] }, 
        VariableCurve { 
            keyframe_timestamps: vec![0.,1.5,2.5,3.5,5.,6.5,7.,8.5,9.], 
            keyframes: Keyframes::Rotation(
                vec![
                    Quat::from_euler(EulerRot::XYZ, 0., 0., 0.),
                    Quat::from_euler(EulerRot::XYZ, PI/10., 0., 0.),
                    Quat::from_euler(EulerRot::XYZ, 0., 0., 0.),
                    Quat::from_euler(EulerRot::XYZ, PI/10. * -1., 0., 0.),
                    Quat::from_euler(EulerRot::XYZ, 0., 0., 0.),
                    Quat::from_euler(EulerRot::XYZ, 0., 0., PI/10.),
                    Quat::from_euler(EulerRot::XYZ, 0., 0., 0.),
                    Quat::from_euler(EulerRot::XYZ, 0., 0., PI/10. * -1.),
                    Quat::from_euler(EulerRot::XYZ, 0., 0., 0.)
                ]
            )
        }
    );

    let spawn_animation_handle = animations.add(spawn_animation);
    let idle_animation_handle = animations.add(idle_animation);
    let map_size = (MAP_RADIUS * MAP_RADIUS) as usize;
    commands.insert_resource(
        BishopRes{
            //scene: assets_server.load("pawn.glb#Scene0"),
            mesh: assets_server.load("bishop.glb#Mesh0/Primitive0"),
            blue_bishop_list: HashSet::with_capacity(map_size),
            red_bishop_list:HashSet::with_capacity(map_size),
            bishop_list: HashMap::with_capacity(map_size),
            spawn_animation: (spawn_animation_handle, spawn_anim),
            idle_animation:(idle_animation_handle, idle_anim),
            control_target: None
        }
    );
}

pub fn bishop_spawn_event(
    mut commands: Commands,
    mut res_map: ResMut<Map>,
    mut res_bishop: ResMut<BishopRes>,
    mut events_bishop: EventReader<OtherSpawn>
){ 
    for ev in events_bishop.iter(){
        match ev.category {
            SpawnCategory::Bishop => {
                let mesh = res_bishop.mesh.clone();
                let mat = 
                    if ev.blue_team {res_map.blue_mat.clone()} else {res_map.red_mat.clone()};
                if ev.blue_team{
                    res_bishop.blue_bishop_list.insert(ev.base_pos);
                }else {
                    res_bishop.red_bishop_list.insert(ev.base_pos);
                }
                let mut player = AnimationPlayer::default();
                player.play(res_bishop.spawn_animation.0.clone());
                let mut trans = Transform::from_xyz(0., HEX_SIZE/3. , 0.);
                trans.scale = Vec3{x:0.4, y:0.4, z:0.4};

                let tile = res_map.entities[&ev.base_pos];
                res_map.blue_entities.insert(tile);
                let mut entity = Entity::from_bits(0);
                commands.entity(tile).with_children(|p|{
                    entity = p.spawn(
                        (
                            PbrBundle{
                                mesh,
                                material: mat,
                                transform: trans,
                                ..Default::default()
                            },
                            Bishop{
                                blue_team: ev.blue_team,
                                pos: ev.base_pos,
                                is_atack: false
                            },
                            res_bishop.spawn_animation.1.clone(),
                            player,
                            SpawnAnimToggle
                        )
                    ).id();
                    res_bishop.bishop_list.insert(ev.base_pos, entity);
                });
                commands.entity(tile).insert(
                  On::<Pointer<Click>>::run(on_bishop_click)
                );
                
            },
            _=>{}
        }
    }

}

fn on_bishop_click(
    mut commands: Commands,
    event: Listener<Pointer<Click>>,
    mut res_map: ResMut<Map>,
    mut res_bishop: ResMut<BishopRes>
){
    if event.button == PointerButton::Primary{
        let target = event.target;
        let base_tile = res_map.entities_forentity[&target];
        res_bishop.control_target = Some(base_tile);
        let mut count = Hex{x: 1, y: 0};
        loop {
            let paint_tile = base_tile + count;
            let Some(block) = res_map.entities.get(&paint_tile) else {break};
            if res_map.red_entities.contains(block){
                break;
            }
            commands.entity(block.clone()).insert(
                On::<Pointer<Click>>::run(on_bishop_path_set_10)  
            );
            res_map.path_list.insert(paint_tile);
            count += Hex{x:1, y:0};
        }
        let mut count = Hex{x: -1, y: 0};
        loop {
            let paint_tile = base_tile + count;
            let Some(block) = res_map.entities.get(&paint_tile) else {break};
            if res_map.red_entities.contains(block){
                break;
            }
            commands.entity(block.clone()).insert(
                On::<Pointer<Click>>::run(on_bishop_path_set_m10)  
            );
            res_map.path_list.insert(paint_tile);
            count += Hex{x:-1, y:0};
        }
        let mut count = Hex{x: 0, y: 1};
        loop {
            let paint_tile = base_tile + count;
            let Some(block) = res_map.entities.get(&paint_tile) else {break};
            if res_map.red_entities.contains(block){
                break;
            }
            commands.entity(block.clone()).insert(
                On::<Pointer<Click>>::run(on_bishop_path_set_01)  
            );
            res_map.path_list.insert(paint_tile);
            count += Hex{x:0, y:1};
        }
        let mut count = Hex{x: 0, y: -1};
        loop {
            let paint_tile = base_tile + count;
            let Some(block) = res_map.entities.get(&paint_tile) else {break};
            if res_map.red_entities.contains(block){
                break;
            }
            commands.entity(block.clone()).insert(
                On::<Pointer<Click>>::run(on_bishop_path_set_0m1)  
            );
            res_map.path_list.insert(paint_tile);
            count += Hex{x:0, y:-1};
        }
        let mut count = Hex{x: 1, y: -1};
        loop {
            let paint_tile = base_tile + count;
            let Some(block) = res_map.entities.get(&paint_tile) else {break};
            if res_map.red_entities.contains(block){
                break;
            }
            commands.entity(block.clone()).insert(
                On::<Pointer<Click>>::run(on_bishop_path_set_1m1)  
            );
            res_map.path_list.insert(paint_tile);
            count += Hex{x:1, y:-1};
        }
        let mut count = Hex{x: -1, y: 1};
        loop {
            let paint_tile = base_tile + count;
            let Some(block) = res_map.entities.get(&paint_tile) else {break};
            if res_map.red_entities.contains(block){
                break;
            }
            commands.entity(block.clone()).insert(
                On::<Pointer<Click>>::run(on_bishop_path_set_m11)  
            );
            res_map.path_list.insert(paint_tile);
            count += Hex{x:-1, y:1};
        }
        
        for m in res_map.path_list.iter(){
            let entity = res_map.entities[m];
            commands.entity(entity).insert(
                res_map.path_mat.clone(),
            );
        }
    }
}

pub fn cancel_path(
    mut commands: Commands,
    mut events_click: EventReader<MouseButtonInput>,
    mut res_map: ResMut<Map>,
    mut res_bishop: ResMut<BishopRes>
){
    for ev in events_click.iter(){
        if ev.button == MouseButton::Left && !res_map.path_mod && res_bishop.control_target.is_none(){
            for m in res_map.path_list.iter(){
                let entity = res_map.entities[m];
                commands.entity(entity).insert(
                    if res_map.blue_entities.contains(&entity){
                        res_map.blue_mat.clone()
                    }else if res_map.red_entities.contains(&entity){
                        res_map.red_mat.clone()
                    }else{
                        res_map.default_mat.clone()
                    }
                );
                commands.entity(entity).remove::<On::<Pointer<Click>>>();
            }
            res_map.path_list.clear();
        }
    }
}

fn on_bishop_path_set_10(
    mut commands: Commands,
    event: Listener<Pointer<Click>>,
    mut res_bishop: ResMut<BishopRes>,
){
    if let Some(control) = res_bishop.control_target{
        let bishop_entity = res_bishop.bishop_list[&control];
        if event.button == PointerButton::Primary{
            commands.entity(bishop_entity).insert(
                BishopAttack{
                    attack_target: Hex::new(1,0),
                    speed: 1.,
                    move_time: 0.
                }  
            );
        }
        res_bishop.control_target = None;
    };
}
fn on_bishop_path_set_01(
    mut commands: Commands,
    event: Listener<Pointer<Click>>,
    mut res_bishop: ResMut<BishopRes>,
){
    if let Some(control) = res_bishop.control_target{
        let bishop_entity = res_bishop.bishop_list[&control];
        if event.button == PointerButton::Primary{
            commands.entity(bishop_entity).insert(
                BishopAttack{
                    attack_target: Hex::new(0,1),
                    speed: 1.,
                    move_time: 0.
                }  
            );
        }
        res_bishop.control_target = None;
    };
}
fn on_bishop_path_set_m10(
    mut commands: Commands,
    event: Listener<Pointer<Click>>,
    mut res_bishop: ResMut<BishopRes>,
){
    if let Some(control) = res_bishop.control_target{
        let bishop_entity = res_bishop.bishop_list[&control];
        if event.button == PointerButton::Primary{
            commands.entity(bishop_entity).insert(
                BishopAttack{
                    attack_target: Hex::new(-1,0),
                    speed: 1.,
                    move_time: 0.
                }  
            );
        }
        res_bishop.control_target = None;
    };
}
fn on_bishop_path_set_0m1(
    mut commands: Commands,
    event: Listener<Pointer<Click>>,
    mut res_bishop: ResMut<BishopRes>,
){
    if let Some(control) = res_bishop.control_target{
        let bishop_entity = res_bishop.bishop_list[&control];
        if event.button == PointerButton::Primary{
            commands.entity(bishop_entity).insert(
                BishopAttack{
                    attack_target: Hex::new(0,-1),
                    speed: 1.,
                    move_time: 0.
                }  
            );
        }
        res_bishop.control_target = None;
    };
}
fn on_bishop_path_set_m11(
    mut commands: Commands,
    event: Listener<Pointer<Click>>,
    mut res_bishop: ResMut<BishopRes>,
){
    if let Some(control) = res_bishop.control_target{
        let bishop_entity = res_bishop.bishop_list[&control];
        if event.button == PointerButton::Primary{
            commands.entity(bishop_entity).insert(
                BishopAttack{
                    attack_target: Hex::new(-1,1),
                    speed: 1.,
                    move_time: 0.
                }  
            );
        }
        res_bishop.control_target = None;
    };
}
fn on_bishop_path_set_1m1(
    mut commands: Commands,
    event: Listener<Pointer<Click>>,
    mut res_bishop: ResMut<BishopRes>,
){
    if let Some(control) = res_bishop.control_target{
        let bishop_entity = res_bishop.bishop_list[&control];
        if event.button == PointerButton::Primary{
            commands.entity(bishop_entity).insert(
                BishopAttack{
                    attack_target: Hex::new(1,-1),
                    speed: 1.,
                    move_time: 0.
                }  
            );
        }
        res_bishop.control_target = None;
    };
}

pub fn bishop_spawn_anim_is_end(
    mut commands: Commands,
    mut query_player: Query<(&mut AnimationPlayer, Entity), (With<SpawnAnimToggle>, With<Bishop>)>,
    res_bishop: Res<BishopRes>,
){
    for (mut player, ent) in query_player.iter_mut(){
        if player.elapsed() > 1.2{
            let mut entity = commands.entity(ent);
            entity.remove::<SpawnAnimToggle>();
            player.play(res_bishop.idle_animation.0.clone()).repeat();
            entity.insert(res_bishop.idle_animation.1.clone());
        }
    }
}

pub fn bishop_attacking(
    mut commands: Commands,
    mut query_bishop: Query<(Entity,&mut Bishop, &mut BishopAttack, &GlobalTransform, &mut Transform)>,
    query_transform: Query<&GlobalTransform, With<Honeycomb>>,
    mut res_bishop: ResMut<BishopRes>,
    mut res_map: ResMut<Map>,
    res_time: Res<Time>
){
    for (entity,mut bishop, mut bishopat, globalt,mut trans)
     in query_bishop.iter_mut(){
        let tile = res_map.entities[&bishop.pos];
        if !bishop.is_atack{
            res_bishop.bishop_list.remove(&bishop.pos);
            commands.entity(tile).remove::<On::<Pointer<Click>>>();
            if bishop.blue_team{
                res_map.blue_entities.remove(&tile);
                res_bishop.blue_bishop_list.remove(&bishop.pos);
            }else{
                res_map.red_entities.remove(&tile);
                res_bishop.red_bishop_list.remove(&bishop.pos);
            }
            bishop.is_atack = true;
        }
        if res_map.red_entities.contains(&tile){
            commands.entity(entity).despawn();
            //원래는 여기에 록이 있는지 확인 후 폭팔해야함
        }else{
            let delta = res_time.delta_seconds();
            bishopat.move_time += delta * bishopat.speed;
            bishopat.speed += delta;

            let target_pos = bishop.pos + bishopat.attack_target;
            let target_entity = res_map.entities.get(&target_pos);
            if target_entity.is_some(){
                let target_entity = target_entity.unwrap();
                let target_tarns = query_transform.get(target_entity.clone()).unwrap().translation();
                let base_pos_trans = query_transform.get(res_map.entities[&bishop.pos]).unwrap().translation();

                let move_trans = base_pos_trans * bishopat.move_time.max(1.) + target_tarns * (1. - bishopat.move_time.max(1.));
                trans.translation = globalt.translation() - move_trans;
                trans.rotation = Quat::from_euler(EulerRot::XYZ, PI / 4., 0., 0.);
                trans.look_at(target_tarns, Vec3::Y);

                if bishopat.move_time > 1. {
                    bishop.pos = target_pos;
                    bishopat.move_time = 0.;
                }
            }else{
                commands.entity(entity).despawn();
            }
        }
    }
}
