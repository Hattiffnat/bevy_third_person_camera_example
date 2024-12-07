use core::f32;

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use settings::UserSettings;

use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};

mod fps_counter;
mod settings;

#[derive(Component, Debug)]
struct ThirdPersCameraTarget {
    pub look_dir: Dir3,
}

impl Default for ThirdPersCameraTarget {
    fn default() -> Self {
        Self {
            look_dir: Dir3::NEG_Z,
        }
    }
}

#[derive(Component, Debug)]
struct ThirdPersonCamera;

#[derive(Component, Debug)]
struct ThirdPersCameraPivot;

const CUBE_SPAWN: Vec3 = Vec3::new(0.0, 1.5, 0.0);

const WALK_SPEED: f32 = 10.0;
const MOUSE_SPEED: f32 = 0.8;
const CAMERA_SPEED: f32 = 1.0;
const CHAR_TURN_SPEED: f32 = 5.0;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(fps_counter::FpsCounterPlugin)
        .insert_resource(settings::UserSettings::default())
        .add_systems(Startup, setup_scene)
        .add_systems(Update, camera_pivot_inherit_cube_translation)
        .add_systems(
            Update,
            (update_target_look, sync_target_with_look)
                .run_if(direction_command_is_pressed),
        )
        .add_systems(
            Update,
            (character_keyboard_control)
                .chain()
                .run_if(direction_command_is_pressed),
        )
        // .add_systems(Update, test_cube_rotation.run_if(arrow_key_is_pressed))
        .add_systems(Update, mouse_camera_control)
        .add_systems(
            Update,
            keyboard_camera_control.run_if(_camera_key_is_pressed),
        )
        .add_systems(Update, toggle_cursor)
        .add_systems(Update, pivot_gizmo)
        .add_systems(Update, character_transform_gizmo)
        .run();
}

fn _camera_key_is_pressed(
    keys: Res<ButtonInput<KeyCode>>,
    user_settings: Res<settings::UserSettings>,
) -> bool {
    let keymap = &user_settings.mouse_keyboard_keymap;
    keys.any_pressed([
        keymap.camera_up,
        keymap.camera_down,
        keymap.camera_left,
        keymap.camera_right,
    ])
}

fn direction_command_is_pressed(
    keys: Res<ButtonInput<KeyCode>>,
    user_settings: Res<settings::UserSettings>,
) -> bool {
    let mk_keymap = &user_settings.mouse_keyboard_keymap;
    keys.any_pressed([
        mk_keymap.forward,
        mk_keymap.left,
        mk_keymap.back,
        mk_keymap.right,
    ])
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<UserSettings>,
) {
    // commands.insert_resource(ClearColor(Color::srgb(0.4, 0.4, 0.3)));
    commands.insert_resource(AmbientLight {
        color: Color::default(),
        brightness: 500.0,
    });

    let floor = (
        Name::new("Floor"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(10.0, 10.0)))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.0, 0.5, 0.5))),
        // Transform::default(),
    );

    commands.spawn(floor);

    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let cube = (
        Name::new("MyCube"),
        ThirdPersCameraTarget::default(),
        Mesh3d(meshes.add(Cuboid::from_length(3.0))),
        MeshMaterial3d(debug_material),
        Transform::from_translation(CUBE_SPAWN),
    );

    let pivot = (
        Name::new("MyCameraPivot"),
        ThirdPersCameraPivot,
        Visibility::Hidden,
        Transform::default(),
    );

    let camera = (
        Name::new("MyCamera"),
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            ..default()
        }),
        Transform::from_translation(settings.camera_settings.translation)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ThirdPersonCamera,
    );

    commands.spawn(cube);
    let pivot_id = commands.spawn(pivot).id();
    let camera_id = commands.spawn(camera).id();

    commands.entity(pivot_id).add_child(camera_id);
}

fn update_target_look(
    mut target_q: Query<&mut ThirdPersCameraTarget>,
    pivot_transf_q: Query<&Transform, With<ThirdPersCameraPivot>>,
    user_settings: Res<settings::UserSettings>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // println!("turn_cube_out_of_camera");
    let Ok(mut target) = target_q.get_single_mut() else {
        return;
    };
    let Ok(pivot_transf) = pivot_transf_q.get_single() else {
        return;
    };

    let keymap = &user_settings.mouse_keyboard_keymap;

    // Направление вычисляется из ориентации опорной точки камеры, без учёта оси Y
    if let Some(new_dir) = {
        if keys.all_pressed([keymap.forward, keymap.right]) {
            Some(pivot_transf.forward().slerp(pivot_transf.right(), 0.5))
        } else if keys.all_pressed([keymap.forward, keymap.left]) {
            Some(pivot_transf.forward().slerp(pivot_transf.left(), 0.5))
        } else if keys.all_pressed([keymap.back, keymap.right]) {
            Some(pivot_transf.back().slerp(pivot_transf.right(), 0.5))
        } else if keys.all_pressed([keymap.back, keymap.left]) {
            Some(pivot_transf.back().slerp(pivot_transf.left(), 0.5))
        } else if keys.pressed(keymap.forward) {
            Some(pivot_transf.forward())
        } else if keys.pressed(keymap.back) {
            Some(pivot_transf.back())
        } else if keys.pressed(keymap.left) {
            Some(pivot_transf.left())
        } else if keys.pressed(keymap.right) {
            Some(pivot_transf.right())
        } else {
            None
        }
    } {
        // обнуляем ось Y
        if let Ok(new_dir) = Dir3::new(new_dir.with_y(0.0)) {
            target.look_dir = new_dir
        };
    }
}

fn sync_target_with_look(
    mut target_transform_q: Query<
        (&mut Transform, &ThirdPersCameraTarget),
        With<ThirdPersCameraTarget>,
    >,
    time: Res<Time>,
) {
    for (mut target_transf, target) in target_transform_q.iter_mut() {
        target_transf.rotation = target_transf.rotation.rotate_towards(
            target_transf.looking_to(target.look_dir, Dir3::Y).rotation,
            CHAR_TURN_SPEED * time.delta_secs(),
        );
    }
}

fn character_keyboard_control(
    mut target_transform_q: Query<&mut Transform, With<ThirdPersCameraTarget>>,
    time: Res<Time>,
) {
    let Ok(mut target_transf) = target_transform_q.get_single_mut() else {
        return;
    };

    // двигаем персонажа вперёд
    let forward = target_transf.forward();
    target_transf.translation += forward * WALK_SPEED * time.delta_secs()
}

fn camera_pivot_inherit_cube_translation(
    mut pivot_transf_q: Query<
        &mut Transform,
        (With<ThirdPersCameraPivot>, Without<ThirdPersCameraTarget>),
    >,
    cube_transf_q: Query<
        &Transform,
        (With<ThirdPersCameraTarget>, Without<ThirdPersCameraPivot>),
    >,
) {
    let Ok(mut pivot_transf) = pivot_transf_q.get_single_mut() else {
        return;
    };
    let Ok(cube_transf) = cube_transf_q.get_single() else {
        return;
    };

    pivot_transf.translation = cube_transf.translation;
}

fn mouse_camera_control(
    mut mouse_motion_reader: EventReader<MouseMotion>,
    mut pivot_transform_q: Query<
        (&mut Transform, &ThirdPersCameraPivot),
        With<ThirdPersCameraPivot>,
    >,
    time: Res<Time>,
) {
    for (mut pivot_transf, pivot) in pivot_transform_q.iter_mut() {
        let delta: Vec2 = mouse_motion_reader.read().map(|motion| motion.delta).sum();

        pivot_transf.rotate_y(-delta.x * MOUSE_SPEED * CAMERA_SPEED * time.delta_secs());
        pivot_transf
            .rotate_local_x(-delta.y * MOUSE_SPEED * CAMERA_SPEED * time.delta_secs());
    }
}

fn keyboard_camera_control(
    keys: Res<ButtonInput<KeyCode>>,
    user_settings: Res<settings::UserSettings>,
    time: Res<Time>,
    mut pivot_transform_q: Query<
        &mut Transform,
        (With<ThirdPersCameraPivot>, Without<Camera>),
    >,
) {
    let Ok(mut pivot_transf) = pivot_transform_q.get_single_mut() else {
        return;
    };

    let keymap = &user_settings.mouse_keyboard_keymap;

    if keys.pressed(keymap.camera_up) {
        pivot_transf.rotate_local_x(-CAMERA_SPEED * time.delta_secs());
    }

    if keys.pressed(keymap.camera_down) {
        pivot_transf.rotate_local_x(CAMERA_SPEED * time.delta_secs());
    }

    if keys.pressed(keymap.camera_left) {
        pivot_transf.rotate_y(-CAMERA_SPEED * time.delta_secs());
    }

    if keys.pressed(keymap.camera_right) {
        pivot_transf.rotate_y(CAMERA_SPEED * time.delta_secs());
    }
}

fn pivot_gizmo(
    mut gizmos: Gizmos,
    hoop_global_t_q: Query<&GlobalTransform, With<ThirdPersCameraPivot>>,
) {
    let Ok(pivot_global_t) = hoop_global_t_q.get_single() else {
        return;
    };
    gizmos.axes(*pivot_global_t, 4.0);
}

fn character_transform_gizmo(
    mut gizmos: Gizmos,
    character_global_t_q: Query<&GlobalTransform, With<ThirdPersCameraTarget>>,
) {
    let Ok(character_global_t) = character_global_t_q.get_single() else {
        return;
    };
    gizmos.axes(*character_global_t, 4.0);
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255,
        102, 255, 198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn toggle_cursor(mut window: Single<&mut Window>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Space) {
        window.cursor_options.visible = !window.cursor_options.visible;
        window.cursor_options.grab_mode = match window.cursor_options.grab_mode {
            CursorGrabMode::None => CursorGrabMode::Locked,
            CursorGrabMode::Locked | CursorGrabMode::Confined => CursorGrabMode::None,
        };
    }
}
