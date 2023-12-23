pub struct RigidBody {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub velocity: cgmath::Vector3<f32>,
    pub acceleration: cgmath::Vector3<f32>,
    pub mass: f32,
}


impl RigidBody {
    pub fn new(position: cgmath::Vector3<f32>,rotation: cgmath::Quaternion<f32>, velocity: cgmath::Vector3<f32>, acceleration: cgmath::Vector3<f32>, mass: f32) -> Self {
        Self {
            position,
            rotation,
            velocity,
            acceleration,
            mass,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let force = self.compute_force();
        let acceleration = force / self.mass;
        self.velocity += acceleration * delta_time;
        self.position += self.velocity * delta_time;
    }

    fn compute_force(&mut self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(0.0, -9.81, 0.0)
    }
}