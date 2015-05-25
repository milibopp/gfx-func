//! Camera specification and reactive controls.

use std::ops::Neg;
use std::fmt::Debug;
use num::{ Zero, One };
use nalgebra::{ Pnt3, Vec3, BaseFloat, Norm, one, zero, UnitQuat, Rotate,
                Rotation, Mat4, PerspMat3, Iso3, ToHomogeneous, Inv };
use carboxyl::{ Signal, Stream };
use carboxyl_window::button::Direction;


/// The number (i.e. -1, 0, +1) associated with the direction.
fn sign<T: Zero + Neg<Output=T> + One>(dir: Direction) -> T {
    match dir {
        Direction::Positive => one(),
        Direction::Negative => -one::<T>(),
        Direction::Still => zero()
    }
}


/// 3D movement direction.
///
/// Any combination of individual x, y, z movement directions.
#[derive(Clone, Copy)]
pub struct MovementState3 {
    pub x: Direction,
    pub y: Direction,
    pub z: Direction,
}

impl MovementState3 {
    pub fn new(x: Direction, y: Direction, z: Direction) -> MovementState3 {
        MovementState3 { x: x, y: y, z: z }
    }

    pub fn unit_vector<T: BaseFloat>(&self) -> Vec3<T> {
        let v = Vec3::new(sign(self.x), sign(self.y), sign(self.z));
        if !v.is_zero() { v.normalize() } else { v }
    }
}


/// A camera.
#[derive(Clone)]
pub struct Camera<T> {
    pub position: Pnt3<T>,
    pub attitude: Attitude<T>,
    pub projection: PerspMat3<T>,
}

impl<T: BaseFloat> Camera<T> {
    pub fn new(pos: Pnt3<T>, att: Attitude<T>, proj: PerspMat3<T>) -> Camera<T> {
        Camera { position: pos, attitude: att, projection: proj }
    }

    pub fn view(&self) -> Mat4<T> {
        let mut view: Iso3<_> = one();
        view.look_at_z(
            &self.position,
            &(self.position + self.attitude.forward()),
            &self.attitude.up()
        );
        view.inv_mut();
        view.to_homogeneous()
    }

    pub fn proj_view(&self) -> Mat4<T> {
        *self.projection.as_mat() * self.view()
    }
}


/// A first-person camera.
pub fn fps_camera<T>(start: Pnt3<T>, time: &Stream<T>,
                  movement: &Signal<MovementState3>,
                  attitude: &Signal<Attitude<T>>,
                  projection: &Signal<PerspMat3<T>>)
    -> Signal<Camera<T>>
    where T: BaseFloat + Clone + Send + Sync + 'static,
{
    let velocity = lift!(|m, att| att.quat().rotate(&m.unit_vector()), movement, attitude);
    let position = integrate_position(start, time, &velocity);
    lift!(Camera::new, &position, attitude, projection)
}


/// Accumulate position from velocity and time.
pub fn integrate_position<T>(start: Pnt3<T>, time: &Stream<T>,
                              velocity: &Signal<Vec3<T>>)
    -> Signal<Pnt3<T>>
    where T: BaseFloat + Clone + Send + Sync + 'static,
{
    velocity.snapshot(time, |v, dt| v * dt)
        .scan(start, |x, dx| x + dx)
}


/// Wraps a quaternion with a rotation convention.
#[derive(Clone, Copy, PartialEq)]
pub struct Attitude<T> {
    quat: UnitQuat<T>,
}

impl<T: BaseFloat> Attitude<T> {
    pub fn new() -> Attitude<T> { Attitude { quat: one() } }
    pub fn quat(&self) -> &UnitQuat<T> { &self.quat }

    pub fn right(&self) -> Vec3<T> { self.quat.rotate(&Vec3::x()) }
    pub fn left(&self) -> Vec3<T> { self.quat.rotate(&(-Vec3::x())) }
    pub fn up(&self) -> Vec3<T> { self.quat.rotate(&Vec3::y()) }
    pub fn down(&self) -> Vec3<T> { self.quat.rotate(&(-Vec3::y())) }
    pub fn back(&self) -> Vec3<T> { self.quat.rotate(&Vec3::z()) }
    pub fn forward(&self) -> Vec3<T> { self.quat.rotate(&(-Vec3::z())) }

    /// Yaw to a new attitude. A positive value means to yaw towards the right.
    pub fn yaw(&self, angle: T) -> Attitude<T> {
        Attitude { quat: self.quat.prepend_rotation(&(self.up() * angle)) }
    }

    /// Pitch to a new attitude. A positive value means to pitch upwards.
    pub fn pitch(&self, angle: T) -> Attitude<T> {
        Attitude { quat: self.quat.prepend_rotation(&(self.right() * angle)) }
    }
}

/// Integrate attitude signal from stream of relative mouse motions
pub fn space_attitude<T>(mouse: &Stream<(T, T)>, sensitivity: T)
    -> Signal<Attitude<T>>
    where T: BaseFloat + Send + Sync + Clone + 'static + Debug,
{
    mouse.scan(
        Attitude::new(),
        move |att, (dx, dy)| {
            att.yaw(-dx * sensitivity)
               .pitch(-dy * sensitivity)
        }
    )
}


#[cfg(test)]
mod test {
    use carboxyl::Sink;
    use nalgebra::{Pnt3, Vec3, Orig, ApproxEq, zero};
    use super::*;

    #[test]
    fn movement_state() {
        use mappings::Direction::*;
        assert_eq!(
            MovementState3 { x: Still, y: Positive, z: Still }
                .unit_vector::<f64>(),
            Vec3::y()
        );
    }

    #[test]
    fn position_accum() {
        let velocity = Sink::new();
        let time = Sink::new();
        let position = integrate_position(
            Orig::orig(),
            &time.stream(),
            &velocity.stream().hold(zero())
        );
        assert_eq!(position.sample(), Orig::orig());
        velocity.send(Vec3::new(1.0, 0.0, 0.0));
        time.send(2.1);
        assert_eq!(position.sample(), Pnt3::new(2.1, 0.0, 0.0));
    }

    #[test]
    fn attitude_yaw() {
        use std::f64::consts;
        let attitude = Attitude::new();
        let new_attitude = attitude.yaw(consts::PI / 2.0);
        assert_approx_eq!(
            new_attitude.forward(),
            attitude.left()
        );
    }

    #[test]
    fn attitude_pitch() {
        use std::f64::consts;
        let attitude = Attitude::new();
        let new_attitude = attitude.pitch(-consts::PI / 2.0);
        assert_approx_eq!(attitude.forward(), new_attitude.up());
    }

    #[test]
    fn attitude_accum() {
        use std::f64::consts;
        let mouse = Sink::new();
        let attitude = space_attitude(&mouse.stream(), consts::PI / 50.0);
        mouse.send((25, 0));
        assert_approx_eq!(
            attitude.sample().quat,
            Attitude::new().yaw(consts::PI / 2.).quat
        );
        mouse.send((-25, 0));
        assert_approx_eq!(
            attitude.sample().quat,
            Attitude::new().quat
        );
        mouse.send((0, 25));
        assert_approx_eq!(
            attitude.sample().quat,
            Attitude::new().pitch(consts::PI / 2.).quat
        );
    }
}

