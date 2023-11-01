use std::{f32::consts::PI, collections::{HashMap, HashSet}};

use bevy::{prelude::*, render::render_resource::PrimitiveTopology, input::mouse::MouseButtonInput};
use bevy_mod_picking::{PickableBundle, prelude::{Pointer, On, Listener, Over, Out, RaycastPickTarget, Click, Down}};
use hexx::*;
// use wasm_bindgen::JsValue;
// use web_sys::console;

pub const HEX_SIZE:f32 = 0.15;
pub const MAP_RADIUS: u32 = 10;
fn hexagonal_column(huneycomb_size: f32) -> Mesh{
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut l_transform = Transform::from_translation(Vec3::new(0., 0., huneycomb_size));
    let mut su_transform = 
        Transform::from_translation(Vec3::new(0., huneycomb_size * 0.3, huneycomb_size * 0.75));
    let mut sd_transform = 
        Transform::from_translation(Vec3::new(0., huneycomb_size * -0.3, huneycomb_size * 0.75));
    let mut base_large_vertex = Vec::with_capacity(6);
    let mut base_small_up_vertex = Vec::with_capacity(6);
    let mut base_small_down_vertex = Vec::with_capacity(6);
    let mut vertex: Vec<[f32;3]> = Vec::with_capacity(96);

    l_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(PI / 6.));
    su_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(PI / 6.));
    sd_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(PI / 6.));

    for _ in 0..6 {
        base_large_vertex.push(l_transform.translation.to_array());
        base_small_up_vertex.push(su_transform.translation.to_array());
        base_small_down_vertex.push(sd_transform.translation.to_array());
        l_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(PI / 3.));
        su_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(PI / 3.));
        sd_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(PI / 3.));
    }
    for i in 0..5{
        vertex = [vertex, vec![
            base_large_vertex[i].clone(), base_large_vertex[i+1].clone(), base_small_up_vertex[i+1].clone(),
            base_small_up_vertex[i].clone(), base_large_vertex[i].clone(), base_small_up_vertex[i+1].clone(), 
            base_large_vertex[i].clone(), base_small_down_vertex[i+1].clone(), base_large_vertex[i+1].clone(), 
            base_small_down_vertex[i].clone(), base_small_down_vertex[i+1].clone(), base_large_vertex[i].clone(),
        ]].concat();
    }
    vertex = [vertex, vec![
        base_large_vertex[5].clone(), base_large_vertex[0].clone(), base_small_up_vertex[0].clone(),
        base_small_up_vertex[5].clone(), base_large_vertex[5].clone(), base_small_up_vertex[0].clone(), 
        base_large_vertex[5].clone(), base_small_down_vertex[0].clone(), base_large_vertex[0].clone(), 
        base_small_down_vertex[5].clone(), base_small_down_vertex[0].clone(), base_large_vertex[5].clone(),
    ]].concat();

    vertex = [vertex, vec![
        base_small_up_vertex[0].clone(), base_small_up_vertex[1].clone(), base_small_up_vertex[5].clone(),
        base_small_up_vertex[1].clone(), base_small_up_vertex[2].clone(), base_small_up_vertex[5].clone(),
        base_small_up_vertex[2].clone(), base_small_up_vertex[4].clone(), base_small_up_vertex[5].clone(),
        base_small_up_vertex[2].clone(), base_small_up_vertex[3].clone(), base_small_up_vertex[4].clone(),
    ]].concat();
    vertex = [vertex, vec![
        base_small_down_vertex[0].clone(), base_small_down_vertex[5].clone(), base_small_down_vertex[1].clone(), 
        base_small_down_vertex[1].clone(), base_small_down_vertex[5].clone(), base_small_down_vertex[2].clone(), 
        base_small_down_vertex[2].clone(), base_small_down_vertex[5].clone(), base_small_down_vertex[4].clone(), 
        base_small_down_vertex[2].clone(), base_small_down_vertex[4].clone(), base_small_down_vertex[3].clone(), 
    ]].concat();
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION, 
        vertex
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 96]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 96]);
    mesh
}

#[derive(Debug, Resource)]
pub struct Map {
    pub entities: HashMap<Hex, Entity>,
    pub entities_forentity: HashMap<Entity, Hex>,
    pub blocked_coords: HashSet<Hex>,
    pub path_list: HashSet<Hex>,
    pub selected_list: HashSet<Hex>,
    pub selected_base: Hex,
    pub blue_entities: HashSet<Entity>,
    pub red_entities: HashSet<Entity>,
    pub layout: HexLayout,
    pub default_mat: Handle<StandardMaterial>,
    pub red_mat: Handle<StandardMaterial>,
    pub blue_mat: Handle<StandardMaterial>,
    pub path_mat: Handle<StandardMaterial>,
    pub highlite_mat :Handle<StandardMaterial>,
    pub seleced_mod: bool,
}

#[derive(Component)]
pub struct Honeycomb;

#[derive(Event)]
pub struct HexSelecedEndEvent{
    pub seleced_list: HashSet<Hex>,
    pub base_seleced: Hex
}
pub fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    let layout = HexLayout {
        hex_size: Vec2::splat(HEX_SIZE),
        ..default()
    };
    // materials
    let default_mat = materials.add(Color::WHITE.into());
    let blue_mat = materials.add(Color::rgb(0.18, 0.44, 0.725).into());
    let red_mat = materials.add(Color::rgb(0.858, 0.266, 0.333).into());
    let highlite_mat = materials.add(Color::LIME_GREEN.into());
    let path_mat = materials.add(Color::rgb(0.4, 0.627, 0.568).into());

    // mesh
    let mesh = hexagonal_column(HEX_SIZE);
    let mesh_handle = meshes.add(mesh);
    let blocked_coords = HashSet::with_capacity(MAP_RADIUS as usize * 3 );
    let mut entities_forentity: HashMap<Entity, Hex> = HashMap::with_capacity(MAP_RADIUS as usize * 3);
    let entities = shapes::hexagon(Hex::ZERO, MAP_RADIUS)
        //.enumerate()
        .map(|hex| {
            let pos = layout.hex_to_world_pos(hex);
            let id = commands
                .spawn((
                    PbrBundle {
                    transform: Transform::from_xyz(pos.x, 0., pos.y)
                        .with_scale(Vec3::splat(1.)),
                    mesh: mesh_handle.clone(),
                    material: default_mat.clone(),
                    ..default()
                    },
                    Honeycomb,
                    PickableBundle::default(),
                    RaycastPickTarget::default(),
                    On::<Pointer<Over>>::run(on_over),
                    On::<Pointer<Out>>::run(on_out),
                    On::<Pointer<Down>>::run(on_click)
                ))
                .id();
            entities_forentity.insert(id, hex);
            (hex, id)
        })
        .collect();
    let map = Map {
        entities,
        default_mat,
        blocked_coords,
        entities_forentity,
        path_list: Default::default(),
        selected_list: Default::default(),
        layout,
        red_mat,
        blue_mat,
        highlite_mat,
        path_mat,
        blue_entities: Default::default(),
        red_entities: Default::default(),
        seleced_mod: false,
        selected_base: Hex::ZERO
    };
    //console::log_1(&JsValue::from_str(format!("{:?}\n", map).as_str()));
    commands.insert_resource(map);
}

pub fn selected_mod(
    mut events_seleted: EventReader<MouseButtonInput>,
    mut res_grid: ResMut<Map>,
    mut events: EventWriter<HexSelecedEndEvent>,
    mut commands: Commands
){
    for ev in events_seleted.iter(){
        if ev.button == MouseButton::Left{
            match ev.state {
                bevy::input::ButtonState::Pressed => {
                    res_grid.seleced_mod = true;
                },
                bevy::input::ButtonState::Released => {
                    res_grid.seleced_mod = false;
                    events.send(HexSelecedEndEvent{
                        seleced_list: res_grid.selected_list.clone(),
                        base_seleced: res_grid.selected_base
                    });
                    for ele in res_grid.selected_list.iter() {
                        let target = res_grid.entities[ele];
                        if res_grid.blue_entities.contains(&target){
                            commands.entity(target).insert(res_grid.blue_mat.clone());
                        }
                        else if res_grid.red_entities.contains(&target){
                            commands.entity(target).insert(res_grid.red_mat.clone());
                        }
                        else{
                            commands.entity(target).insert(res_grid.default_mat.clone());
                        }
                    }
                    res_grid.selected_list.clear();
                }
            }
        }
    }
}

fn on_over(
    mut commands: Commands,
    event: Listener<Pointer<Over>>,
    mut grid: ResMut<Map>
){
    let target = event.target;
    if grid.seleced_mod{
        let seleced_hex = grid.entities_forentity[&target];
        grid.selected_list.insert(seleced_hex);
        commands.entity(target).insert(
            grid.path_mat.clone()
        );
    }else{
        commands.entity(target).insert(
            grid.highlite_mat.clone()
        );
    }
}

fn on_out(
    mut commands: Commands,
    event: Listener<Pointer<Out>>,
    grid: Res<Map>,
){
    let target = event.target;
    if grid.selected_list.contains(&grid.entities_forentity[&target]){
        commands.entity(target).insert(grid.path_mat.clone());
    }
    else if grid.blue_entities.contains(&target){
        commands.entity(target).insert(grid.blue_mat.clone());
    }
    else if grid.red_entities.contains(&target){
        commands.entity(target).insert(grid.red_mat.clone());
    }
    else{
        commands.entity(target).insert(grid.default_mat.clone());
    }
}

fn on_click(
    mut commands: Commands,
    event: Listener<Pointer<Down>>,
    mut grid: ResMut<Map>
){
    let target = event.target;
    if grid.seleced_mod{
        let seleced_hex = grid.entities_forentity[&target];
        grid.selected_base = seleced_hex;
        grid.selected_list.insert(seleced_hex);
        commands.entity(target).insert(
            grid.path_mat.clone()
        );
    }
}
