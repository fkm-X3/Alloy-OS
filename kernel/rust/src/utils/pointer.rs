/// Pointer/mouse utility helpers shared across runtime paths.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotionUpdate {
    pub next_x: i32,
    pub next_y: i32,
    pub actual_dx: i32,
    pub actual_dy: i32,
}

pub fn apply_relative_motion(
    current_x: i32,
    current_y: i32,
    delta_x: i32,
    delta_y: i32,
    max_x: i32,
    max_y: i32,
) -> MotionUpdate {
    let bounded_max_x = max_x.max(0);
    let bounded_max_y = max_y.max(0);

    let next_x = current_x.saturating_add(delta_x).clamp(0, bounded_max_x);
    let next_y = current_y.saturating_add(delta_y).clamp(0, bounded_max_y);

    MotionUpdate {
        next_x,
        next_y,
        actual_dx: next_x.saturating_sub(current_x),
        actual_dy: next_y.saturating_sub(current_y),
    }
}

pub fn button_state_changed(previous_buttons: u8, current_buttons: u8, mask: u8) -> Option<bool> {
    let was_pressed = (previous_buttons & mask) != 0;
    let is_pressed = (current_buttons & mask) != 0;

    if was_pressed == is_pressed {
        None
    } else {
        Some(is_pressed)
    }
}
