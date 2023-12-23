use crate::{Instance};

use crate::physics::{RigidBody};


pub struct Thing{
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    instance: Instance,
    rigid_body: RigidBody,
}


impl Thing{
    pub fn new(position: cgmath::Vector3<f32>, rotation: cgmath::Quaternion<f32>, instance: Instance) -> Self {
        let rigid_body = RigidBody::new(position,
            rotation,
            cgmath::Vector3::new(0.0, 0.0, 0.0), cgmath::Vector3::new(0.0, -9.81, 0.0), 1.0);
        Self {
            position,
            rotation,
            instance,
            rigid_body,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.rigid_body.update(delta_time);
        self.position = self.rigid_body.position;
        self.rotation = self.rigid_body.rotation;

        self.instance.update_transform(self.position, self.rotation);
    }
}