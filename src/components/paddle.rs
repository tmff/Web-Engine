use crate::component::Component;
use winit::{
    event::*,
};

use crate::physics::RigidBody;

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
    fn start(&mut self){
        //initialize paddle
    }
    fn update(&mut self,rigidbody : &mut RigidBody, dt: f32){
        //update paddle
        if self.is_left_pressed {
            rigidbody.velocity.x = -10.0;
        }
        if self.is_right_pressed {
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