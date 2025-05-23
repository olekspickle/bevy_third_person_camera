use crate::{zoom_condition, ThirdPersonCamera};
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use std::f32::consts::PI;

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, orbit_mouse.run_if(orbit_condition))
            .add_systems(Update, (zoom_mouse.run_if(zoom_condition),));
    }
}

// only run the orbit system if the cursor lock is disabled
fn orbit_condition(cam_q: Query<&ThirdPersonCamera>) -> bool {
    let Ok(cam) = cam_q.single() else {
        return true;
    };
    cam.cursor_lock_active
}

// heavily referenced https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
#[allow(clippy::type_complexity)]
pub fn orbit_mouse(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut cam_q: Query<(&ThirdPersonCamera, &mut Transform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mouse_evr: EventReader<MouseMotion>,
) {
    let mut rotation = Vec2::ZERO;
    for ev in mouse_evr.read() {
        rotation = ev.delta;
    }

    let Ok((cam, mut cam_transform)) = cam_q.single_mut() else {
        return;
    };

    if cam.mouse_orbit_button_enabled && !mouse.pressed(cam.mouse_orbit_button) {
        return;
    }

    rotation *= cam.sensitivity;

    if rotation.length_squared() > 0.0 {
        let window = window_q.single().unwrap();
        // Calculate pitch/yaw deltas
        let delta_x = rotation.x / window.width() * PI * cam.sensitivity.x;
        let delta_y = rotation.y / window.height() * PI * cam.sensitivity.y;

        // Current rotation
        let yaw = Quat::from_rotation_y(-delta_x);
        let pitch = Quat::from_rotation_x(-delta_y);
        let new_rotation = yaw * cam_transform.rotation * pitch;

        // Orientation check for "no flip"
        let mut passes_bounds = true;
        let up_vector = new_rotation * Vec3::Y;

        for bound in &cam.bounds {
            // Check NO_FLIP manually
            if bound.normal == Vec3::NEG_Y && bound.point == Vec3::ZERO {
                if up_vector.y <= 0.0 {
                    passes_bounds = false;
                    break;
                }
            } else {
                // Position-based bounds (e.g. floor)
                let rot_matrix = Mat3::from_quat(new_rotation);
                let new_position = rot_matrix * Vec3::new(0.0, 0.0, cam.zoom.radius);
                let to_cam = new_position - bound.point;
                if bound.normal.dot(to_cam) < 0.0 {
                    passes_bounds = false;
                    break;
                }
            }
        }

        if passes_bounds {
            cam_transform.rotation = new_rotation;
        }
    }

    let rot_matrix = Mat3::from_quat(cam_transform.rotation);
    cam_transform.translation = rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, cam.zoom.radius));
}

fn zoom_mouse(mut scroll_evr: EventReader<MouseWheel>, mut cam_q: Query<&mut ThirdPersonCamera>) {
    let mut scroll = 0.0;
    for ev in scroll_evr.read() {
        scroll += ev.y;
    }

    if let Ok(mut cam) = cam_q.single_mut() {
        if scroll.abs() > 0.0 {
            let new_radius =
                cam.zoom.radius - scroll * cam.zoom.radius * 0.1 * cam.zoom_sensitivity;
            cam.zoom.radius = new_radius.clamp(cam.zoom.min, cam.zoom.max);
        }
    }
}
