use bevy::{input::mouse::MouseMotion, prelude::*};
use settings::UserSettings;

mod settings;

#[derive(Component, Debug)]
struct ThirdPersCameraTarget;

#[derive(Component, Debug)]
struct ThirdPersonCamera;

#[derive(Component, Debug)]
struct ThirdPersCameraPivot;

const CUBE_SPAWN: Vec3 = Vec3::new(0.0, 1.5, 0.0);

const WALK_SPEED: f32 = 4.0;
const MOUSE_SPEED: f32 = 0.001;
const TURN_SPEED: f32 = 10.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(settings::UserSettings::default())
        .add_systems(Startup, setup_scene)
        .add_systems(Update, camera_pivot_inherit_cube_translation)
        .add_systems(
            Update,
            (sync_target_with_camera, character_keyboard_mouse_control)
                .chain()
                .run_if(direction_command_is_pressed),
        )
        // .add_systems(Update, test_cube_rotation.run_if(arrow_key_is_pressed))
        .add_systems(Update, my_camera_control)
        .add_systems(Update, pivot_gizmo)
        .add_systems(Update, cube_transform_gizmo)
        .run();
}

fn arrow_key_is_pressed(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.any_pressed([KeyCode::ArrowLeft, KeyCode::ArrowRight])
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

    let cube = (
        Name::new("MyCube"),
        ThirdPersCameraTarget,
        Mesh3d(meshes.add(Cuboid::from_length(3.0))),
        MeshMaterial3d(materials.add(Color::linear_rgb(1.0, 0.0, 0.0))),
        Transform::from_translation(CUBE_SPAWN),
    );

    let pivot = (
        Name::new("MyCameraPivot"),
        ThirdPersCameraPivot,
        Transform::default(),
    );

    let camera = (
        Name::new("MyCamera"),
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            ..default()
        }),
        Transform::from_translation(settings.camera_settings.translation).looking_at(Vec3::ZERO, Vec3::Y),
        ThirdPersonCamera,
    );

    commands.spawn(cube);
    let hoop_id = commands.spawn(pivot).id();
    let camera_id = commands.spawn(camera).id();

    commands.entity(hoop_id).add_child(camera_id);
}

fn sync_target_with_camera(
    mut target_transform_q: Query<
        &mut Transform,
        (With<ThirdPersCameraTarget>, Without<ThirdPersonCamera>),
    >,
    pivot_transf_q: Query<
        &Transform,
        (With<ThirdPersCameraPivot>, Without<ThirdPersCameraTarget>),
    >,
    time: Res<Time>,
) {
    // println!("turn_cube_out_of_camera");
    let Ok(mut target_transf) = target_transform_q.get_single_mut() else {
        return;
    };
    let Ok(pivot_transf) = pivot_transf_q.get_single() else {
        return;
    };

    let new_direction = pivot_transf.forward();
    let Ok(new_direction) = Dir3::from_xyz(new_direction.x, 0.0, new_direction.z) else {
        return;
    };

    target_transf.rotation = target_transf.rotation.lerp(
        target_transf.looking_to(new_direction, Dir3::Y).rotation,
        TURN_SPEED * time.delta_secs(),
    );
}

fn character_keyboard_mouse_control(
    mut cube_transform_q: Query<
        &mut Transform,
        (With<ThirdPersCameraTarget>, Without<ThirdPersCameraPivot>),
    >,
    pivot_transform_q: Query<
        &Transform,
        (With<ThirdPersCameraPivot>, Without<ThirdPersCameraTarget>),
    >,
    user_settings: Res<settings::UserSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut cube_transf) = cube_transform_q.get_single_mut() else {
        return;
    };

    let mk_keymap = &user_settings.mouse_keyboard_keymap;

    if let Some(movement_dir) = {
        if keys.all_pressed([mk_keymap.forward, mk_keymap.right]) {
            Some(cube_transf.forward().slerp(cube_transf.right(), 0.5))
        } else if keys.all_pressed([mk_keymap.forward, mk_keymap.left]) {
            Some(cube_transf.forward().slerp(cube_transf.left(), 0.5))
        } else if keys.all_pressed([mk_keymap.back, mk_keymap.right]) {
            Some(cube_transf.back().slerp(cube_transf.right(), 0.5))
        } else if keys.all_pressed([mk_keymap.back, mk_keymap.left]) {
            Some(cube_transf.back().slerp(cube_transf.left(), 0.5))
        } else if keys.pressed(mk_keymap.forward) {
            Some(cube_transf.forward())
        } else if keys.pressed(mk_keymap.back) {
            Some(cube_transf.back())
        } else if keys.pressed(mk_keymap.left) {
            Some(cube_transf.left())
        } else if keys.pressed(mk_keymap.right) {
            Some(cube_transf.right())
        } else {
            None
        }
    } {
        cube_transf.translation +=
            movement_dir.normalize() * WALK_SPEED * time.delta_secs()
    }
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

fn my_camera_control(
    mut mouse_motion_reader: EventReader<MouseMotion>,
    mut pivot_transform_q: Query<
        &mut Transform,
        (With<ThirdPersCameraPivot>, Without<Camera>),
    >,
) {
    let Ok(mut pivot_transf) = pivot_transform_q.get_single_mut() else {
        return;
    };

    let delta: Vec2 = mouse_motion_reader.read().map(|motion| motion.delta).sum();

    pivot_transf.rotate_y(-delta.x * MOUSE_SPEED);
    pivot_transf.rotate_local_x(-delta.y * MOUSE_SPEED);
}

fn pivot_gizmo(
    mut gizmos: Gizmos,
    hoop_global_t_q: Query<&GlobalTransform, With<ThirdPersCameraPivot>>,
) {
    let Ok(hoop_global_t) = hoop_global_t_q.get_single() else {
        return;
    };
    gizmos.axes(*hoop_global_t, 4.0);
}

fn cube_transform_gizmo(
    mut gizmos: Gizmos,
    hoop_global_t_q: Query<&GlobalTransform, With<ThirdPersCameraTarget>>,
) {
    let Ok(hoop_global_t) = hoop_global_t_q.get_single() else {
        return;
    };
    gizmos.axes(*hoop_global_t, 4.0);
}

fn _cube_rotation(
    mut cube_transf_q: Query<&mut Transform, With<ThirdPersCameraTarget>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut cube_transf) = cube_transf_q.get_single_mut() else {
        return;
    };

    if keys.pressed(KeyCode::ArrowLeft) {
        cube_transf.rotate_local_y(-TURN_SPEED * time.delta_secs());
    } else if keys.pressed(KeyCode::ArrowRight) {
        cube_transf.rotate_local_y(TURN_SPEED * time.delta_secs());
    };
}
