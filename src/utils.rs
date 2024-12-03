use std::f32::consts::PI;

use bevy::math::{IVec3, Quat, Vec3};

#[inline]
pub fn div_floor<
    T: num_traits::Num + num_traits::identities::One + num_traits::sign::Signed + Copy,
>(
    x: T,
    y: T,
) -> T {
    let new = x / y;
    if x.is_negative() ^ y.is_negative() {
        new - T::one()
    } else {
        new
    }
}

pub enum RotationDirection {
    X,
    Y,
    Z,
}

pub fn rotate_around(pos: &Vec3, pivot: &Vec3, angle: f32, direction: &RotationDirection) -> Vec3 {
    let radiens = angle * (PI / 180.);
    let quat = match direction {
        RotationDirection::X => Quat::from_rotation_x(radiens),
        RotationDirection::Y => Quat::from_rotation_y(radiens),
        RotationDirection::Z => Quat::from_rotation_z(radiens),
    };
    quat.mul_vec3(*pos - *pivot) + *pivot
}

pub fn vec_round_to_int(vec: &Vec3) -> IVec3 {
    let rounded = vec.round();
    IVec3::new(rounded.x as i32, rounded.y as i32, rounded.z as i32)
}
