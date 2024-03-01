use std::f32::EPSILON;

use cgmath::InnerSpace;
use cgmath::Vector3;
use cgmath::SquareMatrix;
use cgmath::Quaternion;
use gloo::console::log;

pub struct RigidBody {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub velocity: cgmath::Vector3<f32>,
    pub acceleration: cgmath::Vector3<f32>,
    pub angular_velocity: cgmath::Vector3<f32>,
    pub mass: f32,
    pub shape: Shape,
}


#[derive(Copy,Clone)]
pub enum Shape {
    Sphere(f32),
    Box(cgmath::Vector3<f32>),
}



impl RigidBody {
    pub fn new(position: Vector3<f32>,rotation: Quaternion<f32>, velocity: Vector3<f32>, acceleration: Vector3<f32>, mass: f32) -> Self {
        Self {
            position,
            rotation,
            velocity,
            acceleration,
            mass,
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            shape: Shape::Box(Vector3::new(1.0, 1.0, 1.0)),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let acceleration = self.acceleration;
        self.velocity += acceleration * delta_time;
        self.position += self.velocity * delta_time;
        self.update_rotation(delta_time);
    }


    pub fn add_torque_impulse(&mut self, torque: cgmath::Vector3<f32>) {
        // Change in angular velocity = i^-1 * torque
        let i = self.moment_of_inertia().invert().unwrap();
        let delta_angular_velocity = i * torque;
        self.angular_velocity += delta_angular_velocity;
    }

    pub fn add_force(&mut self, force: cgmath::Vector3<f32>) {
        self.acceleration += force / self.mass;
    }

    pub fn is_intersecting(&self, other: &RigidBody) -> bool {
        match (self.shape, other.shape) {
            (Shape::Sphere(radius1), Shape::Sphere(radius2)) => {
                let distance = (self.position - other.position).magnitude();
                distance < radius1 + radius2
            }
            (Shape::Box(size1), Shape::Box(size2)) => {
                let half_size1 = size1 / 2.0;
                let half_size2 = size2 / 2.0;
                let distance = (self.position - other.position);
                distance.x.abs() < half_size1.x + half_size2.x &&
                    distance.y.abs() < half_size1.y + half_size2.y &&
                    distance.z.abs() < half_size1.z + half_size2.z
            }
            (Shape::Sphere(radius), Shape::Box(size)) => {
                let half_size = size / 2.0;
                let mut distance = (self.position - other.position);
                distance.x = distance.x.max(-half_size.x).min(half_size.x);
                distance.y = distance.y.max(-half_size.y).min(half_size.y);
                distance.z = distance.z.max(-half_size.z).min(half_size.z);
                distance.magnitude() < radius
            }
            (Shape::Box(size), Shape::Sphere(radius)) => other.is_intersecting(self),

            _ => unimplemented!("is_intersecting not implemented for this shape"),
        }
    }

    fn update_rotation(&mut self ,delta_time: f32) {
        let half_delta_rot = Quaternion::from_sv(0.0,0.5 * self.angular_velocity * delta_time);
        let new_orientation = self.rotation + (self.rotation * half_delta_rot);
        log!("Angular Velocity: {:?}", self.angular_velocity.magnitude().to_string());
        if new_orientation.magnitude() < f32::EPSILON{ // Avoid NaN maybe needs a cleaner fix
            return;
        }


        let normalised_rotation = new_orientation.normalize();
        self.rotation = normalised_rotation;
    }


    fn moment_of_inertia(&self) -> cgmath::Matrix3<f32> {
        match self.shape {
            Shape::Box(size) => {
                let x = size.x * size.x;
                let y = size.y * size.y;
                let z = size.z * size.z;
                let i = self.mass / 12.0;
                cgmath::Matrix3::new(
                    i * (y + z), 0.0, 0.0,
                    0.0, i * (x + z), 0.0,
                    0.0, 0.0, i * (x + y),
                )
            }
            Shape::Sphere(radius) => {
                let i = (2.0 / 5.0) * self.mass * radius * radius;
                cgmath::Matrix3::new(
                    i, 0.0, 0.0,
                    0.0, i, 0.0,
                    0.0, 0.0, i,
                )
            }
            _ => unimplemented!("moment_of_inertia not implemented for this shape"),
        }
    }
}
