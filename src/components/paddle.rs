use crate::component::Component;
use winit::event::*;


use crate::physics::RigidBody;
use gloo::console::log;

pub struct Paddle {
    input_keys : Vec<VirtualKeyCode>,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}


impl Paddle {
    pub fn new(input_keys : Vec<VirtualKeyCode>) -> Self {
        Self {
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            input_keys,
        }
    }
    
}


impl Component for Paddle{
    fn start(&mut self,rigidbodys : &mut Vec<RigidBody>,body_index: usize){
        //initialize paddle
    }
    fn update(&mut self, dt: f32, rigidbodys : &mut Vec<RigidBody>,body_index: usize){
        //update paddle

        
        let rigidbody = &mut rigidbodys[body_index];
        
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
                if keycode == &self.input_keys[0] {
                    self.is_forward_pressed = is_pressed;
                    return true;
                }
                else if keycode == &self.input_keys[1] {
                    self.is_backward_pressed = is_pressed;
                    return true;
                }
                else if keycode == &self.input_keys[2] {
                    self.is_left_pressed = is_pressed;
                    return true;
                }
                else if keycode == &self.input_keys[3] {
                    self.is_right_pressed = is_pressed;
                    return true;
                }
                else{
                    return false;
                }
            }
            _ => false,
        }
    }
}