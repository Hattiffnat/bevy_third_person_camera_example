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
pub enum InputType {
    MouseKeyboard,
    Gamepad,
}

#[derive(Resource, Debug)]
pub struct UserKeymap {
    pub forward: KeyCode,
    pub back: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub run: KeyCode,
    pub jump: KeyCode,
    pub interact: KeyCode,
}

impl Default for UserKeymap {
    fn default() -> Self {
        UserKeymap {
            forward: KeyCode::KeyW,
            back: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            run: KeyCode::ShiftLeft,
            jump: KeyCode::Space,
            interact: KeyCode::KeyE,
        }
    }
}

#[derive(Resource, Debug)]
pub struct UserSettings {
    pub camera_settings: CameraSettings,
    pub mouse_keyboard_keymap: UserKeymap,
    pub input_type: InputType,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            camera_settings: CameraSettings::default(),
            mouse_keyboard_keymap: UserKeymap::default(),
            input_type: InputType::MouseKeyboard,
        }
    }
}
