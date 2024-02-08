use gloo::file::Blob;
//use thing::Thing;
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

use std::{collections::HashMap, iter};
use wgpu::util::DeviceExt;

mod texture;
mod camera;
mod gui;
mod model;
mod resources;
//mod thing;
mod physics;


use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use winit::window::Window;

use cgmath::{prelude::*, Vector3, Quaternion};


use wasm_timer::Instant;

use model::{DrawModel,Vertex, Model};

use web_sys::{HtmlInputElement, FileList, File};



struct ModelInstances {
    model : model::Model,
    instances : Vec<Instance>,
    instance_buffer : wgpu::Buffer,
}

impl ModelInstances {
    pub fn new(model : model::Model,device : &wgpu::Device,instances: Vec<Instance>) -> Self {
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Self {
            model,
            instances,
            instance_buffer,
        }
    }

}


pub struct Data{
    rotation_speed: f32,
    clear_color: [f32; 4],
}

impl Data {
    pub fn new() -> Self {
        Self {
            rotation_speed: 2.0 * std::f32::consts::PI / 60.0,
            clear_color: [0.1, 0.2, 0.3, 1.0],
        }
    }
}

pub enum CustomEvent {
    RedrawRequested,
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompareFunction {
    Undefined = 0,
    Never = 1,
    Less = 2,
    Equal = 3,
    LessEqual = 4,
    Greater = 5,
    NotEqual = 6,
    GreaterEqual = 7,
    Always = 8,
}


pub struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    rigid_body: physics::RigidBody,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.rigid_body.update(delta_time);
        self.position = self.rigid_body.position;
        self.rotation = self.rigid_body.rotation;
    }
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}





#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &camera::Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

struct State{
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    diffuse_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    diffuse_texture: texture::Texture,
    camera: camera::Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: camera::CameraController,
    //instances: Vec<Instance>,
    //instance_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    gui : gui::Gui,
    start_time: Instant,
    data: Data,
    last_frame_time : Instant,
    frame_times : Vec<u128>,
    //obj_model: model::Model,
    model_instances: Vec<ModelInstances>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl State {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();


        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);


        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())            
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");


        let diffuse_bytes = include_bytes!("allmyfellas.png"); // CHANGED!
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "allmyfellas.png").unwrap(); // CHANGED!



        const NUM_INSTANCES_PER_ROW: u32 = 10;
        //const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);

        /*
        const SPACE_BETWEEN: f32 = 15.0;
        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
        
                let position = cgmath::Vector3 { x, y: 0.0, z };
        
                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };
        
                Instance {
                    position, rotation,
                }
            })
        }).collect::<Vec<_>>();
        
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
        */


        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        
        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );
        
        let camera = camera::Camera {
            eye: (0.0,1.0,2.0).into(),
            target: (0.0,0.0,0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        
        // in new() after creating `camera`

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        
        let gui = gui::Gui::new(&window, &device);

        

        let clear_color = wgpu::Color::RED;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout,&camera_bind_group_layout],
                push_constant_ranges: &[],
        });


        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, 
                stencil: wgpu::StencilState::default(), 
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,


        });

        let camera_controller = camera::CameraController::new(0.2);
        let obj_model =
    resources::load_model("french_bulldog.obj", &device, &queue, &texture_bind_group_layout)
        .await
        .unwrap();



        Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            clear_color,
            size,
            window,
            render_pipeline,
            diffuse_bind_group,
            diffuse_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            depth_texture,
            gui,
            start_time: Instant::now(),
            data: Data::new(),
            last_frame_time: Instant::now(),
            frame_times: vec![],
            model_instances:vec![],
            texture_bind_group_layout,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0{
            self.size = new_size;
            self.config.height = new_size.height;
            self.config.width = new_size.width;
            
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
            }
            _ => {},
        }
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        let now = Instant::now();
        let delta = now - self.last_frame_time;
        self.last_frame_time = now;
        self.frame_times.push(delta.as_millis());

        if self.frame_times.len() > 10 {
            self.frame_times.remove(0);
        }
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        //let model_instances_to_update: Vec<&ModelInstances> = vec![]; // List of model instances that have been changed this frame, write the buffer for these
        let last_delta = self.get_last_delta() as f32 / 1000.0;


        //Do all processing

        //This might be bad for performance, but for now updating every model instance buffer every frame will work
        for i in 0..self.model_instances.len() {
            // Accessing each instance mutably
            for instance in self.model_instances[i].instances.iter_mut() {
                instance.update(last_delta);
            }
        
            // Preparing data for the buffer
            let instance_data = self.model_instances[i]
                .instances
                .iter()
                .map(Instance::to_raw)
                .collect::<Vec<_>>();
        
            // Writing data to the buffer
            self.queue.write_buffer(
                &self.model_instances[i].instance_buffer,
                0,
                bytemuck::cast_slice(&instance_data),
            );
        }
        

        /*
        for i in self.instances.iter_mut(){
            let amount = cgmath::Quaternion::from_angle_y(cgmath::Rad(self.data.rotation_speed));
            i.rotation = i.rotation * amount;
        }
        let instance_data = self
        .instances
        .iter()
        .map(Instance::to_raw)
        .collect::<Vec<_>>();

    self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );
        */
        
    }

    fn instance_from_model(&mut self, model: model::Model){
        let model_name = model.name.clone();

        let mut new_instance = Instance {
            position: cgmath::Vector3{x: 0.0, y: 0.0, z: 0.0},
            rotation: cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0),
            rigid_body: physics::RigidBody::new(
                cgmath::Vector3{x: 0.0, y: 0.0, z: 0.0},
                cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0),
                cgmath::Vector3{x: 0.0, y: 0.0, z: 0.0},
                cgmath::Vector3{x: 0.0, y: -9.81, z: 0.0},
                1.0,
            ),
        };
        
        if let Some(instance) = self.model_instances.iter_mut().find(|i| i.model.name == model_name) { //Maybe change comparison to model file hash
            // Model instance found, push new instance to instances
            instance.instances.push(new_instance);
        } else {
            // No existing model instance, create a new one and push to model_instances
            let instances = vec![new_instance];
            self.model_instances.push(ModelInstances::new(model, &self.device, instances));
        }

    }
    



    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        self.setup_gui();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // render()

            render_pass.set_pipeline(&self.render_pipeline);
            /*
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            //TODO: add draw_model_instance for each ModelInstance
            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.camera_bind_group,
            );
            */

            for model_instance in self.model_instances.iter() {
                render_pass.set_vertex_buffer(1, model_instance.instance_buffer.slice(..));
                render_pass.draw_model_instanced(
                    &model_instance.model,
                    0..model_instance.instances.len() as u32,
                    &self.camera_bind_group,
                );
            }


            
        }
        self.gui.render(&mut encoder, &output, &self.window, &self.device, &self.queue);

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn setup_gui(&mut self){
        self.gui.begin_new_frame(self.start_time.elapsed().as_secs_f64());

        let platform = self.gui.platform_mut();
        let avg_frame_time = self.frame_times.iter().sum::<u128>() / (self.frame_times.len() + 1) as u128;

        egui::Window::new("Info")
        .resizable(true)
        .show(&platform.context(), |ui| {
            ui.add(egui::Label::new(format!(
                "Frame time: {}ms",
                avg_frame_time,
            )));
            //ui.add(egui::Slider::new(&mut self.data.rotation_speed, 0.0..=1.0).text("Rotation Speed"));
            ui.color_edit_button_rgba_premultiplied(&mut self.data.clear_color);
        });
        self.clear_color = wgpu::Color {
            r: self.data.clear_color[0] as f64,
            g: self.data.clear_color[1] as f64,
            b: self.data.clear_color[2] as f64,
            a: self.data.clear_color[3] as f64,
        };
    }

    fn get_last_delta(&self) -> u128 {
        self.frame_times[self.frame_times.len() - 1]
    }

}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));
        
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(window).await;

    let obj_model = //Move this into a start function in state maybe
    resources::load_model("french_bulldog.obj", &state.device, &state.queue, &state.texture_bind_group_layout)
        .await
        .unwrap();

    state.instance_from_model(obj_model);

    resources::get_files().unwrap();


    event_loop.run(move |event, _, control_flow| {
        
        state.gui.handle_event(&event);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },
                    
                    WindowEvent::ScaleFactorChanged {new_inner_size, ..} => {
                        state.resize(**new_inner_size)
                    },
                    

                    _ => {}
                }
            },
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            match state.render() {
                Ok(_) => {},

                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }

        Event::MainEventsCleared => {
            state.window().request_redraw();
        }


        
        _ => {}
    }});
}



