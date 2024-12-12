use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct CameraSettings {
    pub translation: Vec3
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            translation: Vec3::new(0.0, 0.0, 10.0)
        }
    }
}

#[derive(Resource, Debug)]
pub struct UserKeymap {
    pub forward: KeyCode,
    pub back: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,

    pub camera_up: KeyCode,
    pub camera_down: KeyCode,
    pub camera_left: KeyCode,
    pub camera_right: KeyCode,
}

impl Default for UserKeymap {
    fn default() -> Self {
        UserKeymap {
            forward: KeyCode::KeyW,
            back: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,

            camera_up: KeyCode::ArrowUp,
            camera_down: KeyCode::ArrowDown,
            camera_left: KeyCode::ArrowLeft,
            camera_right: KeyCode::ArrowRight,
        }
    }
}

#[derive(Resource, Debug)]
pub struct UserSettings {
    pub camera_settings: CameraSettings,
    pub mouse_keyboard_keymap: UserKeymap,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            camera_settings: CameraSettings::default(),
            mouse_keyboard_keymap: UserKeymap::default(),
        }
    }
}
