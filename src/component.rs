use winit::event;
use crate::physics::RigidBody;


pub trait Component {
    fn start(&mut self,rigidbodys : &mut Vec<RigidBody>,body_index: usize);
    fn update(&mut self, dt: f32,rigidbodys : &mut Vec<RigidBody>,body_index: usize);
    fn input(&mut self,event: &event::WindowEvent) -> bool;
}