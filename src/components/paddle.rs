use crate::component::Component;
use winit::event::*;


use crate::physics::RigidBody;
use gloo::console::log;

pub struct Paddle {
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}


impl Paddle {
    pub fn new() -> Self {
        Self {
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }
    
}


impl Component for Paddle{
    fn start(&mut self,rigidbodys : &mut Vec<RigidBody>,body_index: usize){
        //initialize paddle
    }
    fn update(&mut self, dt: f32, rigidbodys : &mut Vec<RigidBody>,body_index: usize){
        //update paddle
        let test = rigidbodys.clone();

        
        let rigidbody = &mut rigidbodys[body_index];
        for i in 0..test.len(){
            if i != body_index{
                let other = &test[i];
                if rigidbody.is_intersecting(other){

                }
            }
        }
        
        if self.is_left_pressed {
            rigidbody.velocity.x = -10.0;
        }
        else if self.is_right_pressed {
            rigidbody.velocity.x = 10.0;
        }
        else{
            rigidbody.velocity.x = 0.0;
        }
    }
    fn input(&mut self,event: &WindowEvent) -> bool{
        //input for paddle
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::I | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::J | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::K | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::L | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}