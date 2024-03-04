use crate::component::Component;
use winit::event::*;
use crate::physics::RigidBody;


pub struct Ball {}

impl Ball {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for Ball {
    fn start(&mut self,rigidbodys : &mut Vec<RigidBody>,body_index: usize) {
        //initialize ball
        let rigidbody = &mut rigidbodys[body_index];
        rigidbody.add_force(cgmath::Vector3::new(0.0, 10.0, 0.0));

    }
    fn update(&mut self, dt: f32, rigidbodys : &mut Vec<RigidBody>,body_index: usize){
        //update ball
        let test = rigidbodys.clone();

        let rigidbody = &mut rigidbodys[body_index];
        for i in 0..test.len(){
            if i != body_index{
                let other = &test[i];
                if rigidbody.is_intersecting(other){
                    rigidbody.velocity = -rigidbody.velocity;
                }
            }
        }

    }
    fn input(&mut self,event: &WindowEvent) -> bool{
        //input for ball
        match event {
            _ => false
        }
    }
}