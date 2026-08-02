use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

use crate::camera::Camera;
use crate::constants::*;
use crate::ui_panels::UiState;

pub fn handle_input(input: &mut WinitInputHelper, camera: &mut Camera, ui_state: &mut UiState) {
    let diff = input.cursor_diff();

    if input.mouse_held(0) {
        camera.orbit(diff.0 * CAMERA_ORBIT_SPEED, -diff.1 * CAMERA_ORBIT_SPEED);
    }
    if input.mouse_held(1) {
        camera.pan(-diff.0, diff.1, CAMERA_PAN_SPEED);
    }

    let scroll_delta = input.scroll_diff();
    if scroll_delta.1 != 0.0 {
        camera.zoom(scroll_delta.1 * CAMERA_ZOOM_SPEED);
    }

    if input.key_pressed(KeyCode::F1) {
        ui_state.toggle_panel();
    }
    if input.key_pressed(KeyCode::F2) {
        ui_state.show_grid = !ui_state.show_grid;
    }
}
