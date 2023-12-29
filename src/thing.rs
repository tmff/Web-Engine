use crate::Instance;

use crate::physics::RigidBody;


pub struct Thing<'a>{
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    instance: &'a mut Instance,
    rigid_body: RigidBody,
}


impl<'a> Thing<'a>{
    pub fn new(instance: &'a mut Instance) -> Self {
        let rigid_body = RigidBody::new(
            instance.position,
            instance.rotation,
            cgmath::Vector3::new(0.0, 0.0, 0.0), cgmath::Vector3::new(0.0, -9.81, 0.0), 1.0);
        Self {
            position: instance.position,
            rotation: instance.rotation,
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