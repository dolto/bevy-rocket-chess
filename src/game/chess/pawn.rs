use std::f32::consts::PI;

use bevy::{prelude::*, utils::{HashSet, HashMap}};
use hexx::Hex;
use rand::Rng;
use crate::game::graphics_3d::honeycomb::{Map, MAP_RADIUS, HEX_SIZE, HexSelecedEndEvent, Honeycomb};

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum PawnSetup{
    #[default]
    Befor,
    After
}

#[derive(Resource)]
pub struct PawnRes{
    //scene: Handle<Scene>,
    mesh: Handle<Mesh>,
    pawn_list: HashMap<Hex, Entity>,
    blue_pawn_list: HashSet<Hex>,
    red_pawn_list: HashSet<Hex>,
    spawn_animation: (Handle<AnimationClip>, Name),
    idle_animation: (Handle<AnimationClip>, Name),
    action_animation: (Handle<AnimationClip>, Name)
}

#[derive(Event)]
pub struct PawnSpawn{
    blue_team: bool,
    pos: Hex,
    entity: Entity
}

#[derive(Component)]
pub struct Pawn{
    spawn_timer: Timer,
    blue_team: bool,
    pos: Hex
}
#[derive(Component)]
pub struct SpawnAnimToggle;

#[derive(Component)]
pub struct ActionAnimToggle;

#[derive(Component)]
pub struct CombinationTarget{
    trans: GlobalTransform,
    time: f32
}

pub fn setup_asset_pawn(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut pawn_setup_state: ResMut<NextState<PawnSetup>>,
    mut animations: ResMut<Assets<AnimationClip>>,
){
    let spawn_anim = Name::new("pawn_spawn");
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
    let idle_anim = Name::new("pawn_idle");
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

    let action_anim = Name::new("pawn_action");
    let mut action_animation = AnimationClip::default();
    action_animation.add_curve_to_path(
        EntityPath { parts: vec![action_anim.clone()] }, 
        VariableCurve { 
            keyframe_timestamps: vec![0.,0.5,1.,1.5], 
            keyframes: Keyframes::Rotation(
                vec![
                    Quat::IDENTITY,
                    Quat::from_euler(EulerRot::XYZ, PI/3., PI/4. * -1., 0.),
                    Quat::from_euler(EulerRot::XYZ, PI/3. * -1., PI/4. * -1., 0.),
                    Quat::IDENTITY
                ]
            )
        }
    );
    let spawn_animation_handle = animations.add(spawn_animation);
    let idle_animation_handle = animations.add(idle_animation);
    let action_animation_handle = animations.add(action_animation);
    let map_size = (MAP_RADIUS * MAP_RADIUS) as usize;
    commands.insert_resource(
        PawnRes{
            //scene: assets_server.load("pawn.glb#Scene0"),
            mesh: assets_server.load("pawn.glb#Mesh0/Primitive0"),
            blue_pawn_list: HashSet::with_capacity(map_size),
            red_pawn_list:HashSet::with_capacity(map_size),
            pawn_list: HashMap::with_capacity(map_size),
            spawn_animation: (spawn_animation_handle, spawn_anim),
            idle_animation:(idle_animation_handle, idle_anim),
            action_animation:(action_animation_handle, action_anim)
        }
    );
    pawn_setup_state.set(PawnSetup::After);
}

fn spawn_pawn(
    parent: &mut ChildBuilder,
    pawn_mesh: Handle<Mesh>,
    select_team: bool,
    pos: Hex,
    mat: Handle<StandardMaterial>,
    player: AnimationPlayer,
) -> Entity{
    let mut rng = rand::thread_rng();
    let mut trans = Transform::from_xyz(0., HEX_SIZE/3. , 0.);
    trans.scale = Vec3{x:0.4, y:0.4, z:0.4};
    parent.spawn((
        // SceneBundle{
        //     scene: pawn_scen,
        //     transform: trans,
        //     ..Default::default()
        // },
        PbrBundle{
            mesh: pawn_mesh,
            transform: trans,
            material: mat,
            ..Default::default()
        },
        Pawn{
            spawn_timer: Timer::from_seconds(rng.gen_range(2.8..3.2), TimerMode::Repeating),
            blue_team: select_team,
            pos
        },
        player,
        Name::new("pawn_spawn"),
        SpawnAnimToggle
    )
    ).id()
}

pub fn spawn_pawn_timer(
    mut query: Query<(&mut Pawn, Entity)>,
    res_time: Res<Time>,
    mut events_pawn_spawn: EventWriter<PawnSpawn>
){
    let time = res_time.delta();
    for (mut p, entity) in query.iter_mut(){
        p.spawn_timer.tick(time);
        if p.spawn_timer.finished(){
            events_pawn_spawn.send(PawnSpawn{
                blue_team: p.blue_team,
                pos: p.pos,
                entity
            });
        }
    }
}

pub fn pawn_spawn_anim_is_end(
    mut commands: Commands,
    mut query_player: Query<(&mut AnimationPlayer, Entity), With<SpawnAnimToggle>>,
    res_pawn: Res<PawnRes>,
){
    for (mut player, ent) in query_player.iter_mut(){
        if player.elapsed() > 1.2{
            let mut entity = commands.entity(ent);
            entity.remove::<SpawnAnimToggle>();
            player.play(res_pawn.idle_animation.0.clone()).repeat();
            entity.insert(res_pawn.idle_animation.1.clone());
        }
    }
}
pub fn pawn_action_anim_is_end(
    mut commands: Commands,
    mut query_player: Query<(&mut AnimationPlayer, Entity), With<ActionAnimToggle>>,
    res_pawn: Res<PawnRes>,
){
    for (mut player, ent) in query_player.iter_mut(){
        if player.elapsed() > 1.5{
            let mut entity = commands.entity(ent);
            entity.remove::<ActionAnimToggle>();
            player.play(res_pawn.idle_animation.0.clone()).repeat();
            entity.insert(res_pawn.idle_animation.1.clone());
        }
    }
}

pub fn pawn_combination_is_end(
    mut commands: Commands,
    mut query_pawns: Query<(&mut AnimationPlayer, &mut CombinationTarget, &mut Transform, &GlobalTransform, Entity)>,
    res_time: Res<Time>
){
    for (mut player,mut combi, mut trans, gt,entity) in query_pawns.iter_mut(){
        player.pause();
        combi.time += res_time.delta_seconds();
        let mut move_tarns = (combi.trans.translation() - gt.translation()) * combi.time * 2.;
        move_tarns.y = HEX_SIZE / 3. + f32::sin(combi.time * PI*2.) / 4.;
        trans.translation = move_tarns;

        if combi.time >= 0.5{
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_pawn_event(
    mut commands: Commands,
    mut res_pawn: ResMut<PawnRes>,
    mut res_map: ResMut<Map>,
    mut events_pawn_spawn: EventReader<PawnSpawn>,
    mut query_player: Query<&mut AnimationPlayer, With<Pawn>>
){
    'eventing: for ev in events_pawn_spawn.iter(){
        let mat: Handle<StandardMaterial>;
        let hex_pos:Hex;
        let mut spawn_list = vec![
            [0,1],
            [1,0],
            [-1,1],
            [0,-1],
            [-1,0],
            [1,-1],
        ];
        let mut rng = rand::thread_rng();
        loop {
            let index = rng.gen_range(0..spawn_list.len());
            let _pos = Hex{x: ev.pos.x + spawn_list[index][0], y: ev.pos.y + spawn_list[index][1]};
            let entity = res_map.entities.get(&_pos);
            if entity.is_some(){
                let ent = entity.unwrap();
                if !res_map.blue_entities.contains(ent) && !res_map.red_entities.contains(ent){
                    hex_pos = _pos;
                    break;
                }else{
                    spawn_list.remove(index);
                }
            }else{
                spawn_list.remove(index);
            }

            if spawn_list.is_empty(){
                continue 'eventing;
            }
        }
        let spawn_entity = res_map.entities[&hex_pos];
        if ev.blue_team{
            res_pawn.blue_pawn_list.insert(hex_pos);
            mat = res_map.blue_mat.clone();
            res_map.blue_entities.insert(spawn_entity);
        }else{
            res_pawn.red_pawn_list.insert(hex_pos);
            mat = res_map.red_mat.clone();
            res_map.red_entities.insert(spawn_entity);
        }

        let mut pawn = commands.entity(ev.entity);
        pawn.insert(ActionAnimToggle);
        let mut spawner = query_player.get_mut(ev.entity).unwrap();
        spawner.play(res_pawn.action_animation.0.clone());
        pawn.insert(res_pawn.action_animation.1.clone());
        
        let mut player = AnimationPlayer::default();
        player.play(res_pawn.spawn_animation.0.clone());
        commands.entity(spawn_entity).insert(mat.clone());
        commands.entity(spawn_entity).with_children(|p|{
            let mesh = res_pawn.mesh.clone();
            res_pawn.pawn_list.insert(hex_pos, spawn_pawn(p,mesh,ev.blue_team, hex_pos, mat, player));
        });
    }
}

pub fn test_setup(
    mut commands: Commands,
    mut res_pawn: ResMut<PawnRes>,
    mut res_map: ResMut<Map>
){
    let pos = (MAP_RADIUS) as i32;
    let hex_pos = Hex{x:pos, y:0};
    let entity = res_map.entities[&hex_pos];
    res_pawn.blue_pawn_list.insert(hex_pos);
    let mat = res_map.blue_mat.clone();
    res_map.blue_entities.insert(entity);
    commands.entity(entity).insert(mat.clone());
    let spawn_entity = res_map.entities[&hex_pos];
    let mut player = AnimationPlayer::default();
    player.play(res_pawn.spawn_animation.0.clone());
    commands.entity(spawn_entity).with_children(|p|{
        let mesh = res_pawn.mesh.clone();
        res_pawn.pawn_list.insert(hex_pos, spawn_pawn(p, mesh ,true, hex_pos, mat,player));
    });
}

pub fn selected_event(
    mut commands: Commands,
    mut events_selected: EventReader<HexSelecedEndEvent>,
    mut res_pawn: ResMut<PawnRes>,
    mut res_map: ResMut<Map>,
    query_transform: Query<&GlobalTransform, With<Honeycomb>>
){
    for ev in events_selected.iter(){
        let base_hex = ev.base_seleced;
        let mut hex_list: HashSet<Hex> = HashSet::with_capacity(ev.seleced_list.capacity());
        for hex in ev.seleced_list.iter(){
            if res_pawn.blue_pawn_list.contains(hex){
                hex_list.insert(hex.clone());
            }
        }
        if bishop_patton(base_hex, hex_list.clone()){
            let base_tile = res_map.entities[&base_hex];
            for hex in hex_list.iter(){
                let pawn = res_pawn.pawn_list.remove(hex).unwrap();
                res_pawn.blue_pawn_list.remove(hex);
                let tile = res_map.entities[hex];
                res_map.blue_entities.remove(&tile);
                commands.entity(tile).insert(
                    res_map.default_mat.clone()
                );
                commands.entity(pawn).insert(
                    CombinationTarget{
                        trans: query_transform.get(base_tile).unwrap().clone(),
                        time: 0.
                    }
                );
            }
            //이벤트 발생
        }
    }
}

pub fn bishop_patton(
    base_hex: Hex,
    hex_list: HashSet<Hex>
) -> bool{
    if hex_list.len() != 3{
        return false;
    }
    let patton = [
        [Hex{x: base_hex.x + 1,y: base_hex.y}, Hex{x: base_hex.x + 2,y: base_hex.y}],
        [Hex{x: base_hex.x + 1,y: base_hex.y}, Hex{x: base_hex.x + 1,y: base_hex.y + 1}],
        [Hex{x: base_hex.x + 1,y: base_hex.y}, Hex{x: base_hex.x + 2,y: base_hex.y - 1}],

        [Hex{x: base_hex.x - 1,y: base_hex.y}, Hex{x: base_hex.x - 2,y: base_hex.y}],
        [Hex{x: base_hex.x - 1,y: base_hex.y}, Hex{x: base_hex.x - 2,y: base_hex.y + 1}],
        [Hex{x: base_hex.x - 1,y: base_hex.y}, Hex{x: base_hex.x - 1,y: base_hex.y - 1}],

        [Hex{x: base_hex.x,y: base_hex.y - 1}, Hex{x: base_hex.x,y: base_hex.y - 2}],
        [Hex{x: base_hex.x,y: base_hex.y - 1}, Hex{x: base_hex.x - 1,y: base_hex.y - 1}],
        [Hex{x: base_hex.x,y: base_hex.y - 1}, Hex{x: base_hex.x + 1,y: base_hex.y - 2}],

        [Hex{x: base_hex.x,y: base_hex.y + 1}, Hex{x: base_hex.x,y: base_hex.y + 2}],
        [Hex{x: base_hex.x,y: base_hex.y + 1}, Hex{x: base_hex.x + 1,y: base_hex.y + 1}],
        [Hex{x: base_hex.x,y: base_hex.y + 1}, Hex{x: base_hex.x - 1,y: base_hex.y + 2}],

        [Hex{x: base_hex.x + 1,y: base_hex.y - 1}, Hex{x: base_hex.x + 2,y: base_hex.y - 2}],
        [Hex{x: base_hex.x + 1,y: base_hex.y - 1}, Hex{x: base_hex.x + 1,y: base_hex.y - 2}],
        [Hex{x: base_hex.x + 1,y: base_hex.y - 1}, Hex{x: base_hex.x + 2,y: base_hex.y - 1}],

        [Hex{x: base_hex.x - 1,y: base_hex.y + 1}, Hex{x: base_hex.x - 2,y: base_hex.y + 2}],
        [Hex{x: base_hex.x - 1,y: base_hex.y + 1}, Hex{x: base_hex.x - 1,y: base_hex.y + 2}],
        [Hex{x: base_hex.x - 1,y: base_hex.y + 1}, Hex{x: base_hex.x - 2,y: base_hex.y + 1}],
    ];
    let mut is_ok = true;
    for p in patton.iter(){
        is_ok = true;
        for pp in p.iter(){
            if !hex_list.contains(pp){
                is_ok = false;
                break;
            }
        }
        if is_ok{
            break;
        }
    }
    return is_ok;
}
