use winit::event;
use crate::physics::RigidBody;


pub trait Component {
    fn start(&mut self);
    fn update(&mut self,rigidbody : &mut RigidBody, dt: f32);
    fn input(&mut self,event: &event::WindowEvent) -> bool;
}