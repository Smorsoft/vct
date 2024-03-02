extern crate nalgebra_glm as glm;

use std::collections::HashMap;

use vct::{camera::CameraDescriptor, *};
use winit::{
	event::*, event_loop::{ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder
};

const CAMERA_MOVE_SPEED: f32 = 10.0;
const CAMERA_ROTATION_SPEED: f32 = 30.0;

fn main() {
	let event_loop = EventLoop::new().unwrap();
	event_loop.set_control_flow(ControlFlow::Poll);
	let window = std::sync::Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
	window
		.set_cursor_grab(winit::window::CursorGrabMode::Confined)
		.unwrap();

	window.set_maximized(true);

	let render_settings = RendererSettings {
		render_scale: 1.0,
		extras: HashMap::new(),
	};

	let mut renderer = pollster::block_on(Renderer::new(window.clone(), render_settings));
	let _cameras = renderer.load_gltf("examples/Sponza/Sponza.gltf", true);
	// .load_gltf("examples/Box.glb", true);

	let mut voxelization_pass = vct::command_encoder::voxelization::VoxelizationPass::new(&renderer);
	let mut meshify_pass = vct::command_encoder::voxelization::MeshifyPass::new(&renderer);
	let mut render_meshify_pass = vct::command_encoder::voxelization::RenderMeshifyPass::new(&renderer);
	let mut forward_render_pass = vct::command_encoder::forward::ForwardRenderingPass::new(&renderer);

	let camera = renderer.new_camera(&CameraDescriptor {
		position: [0.0, 0.0, 0.0].into(),
		rotation: glm::Quat::identity(),
		resolution: camera::Resolution::UseGlobalResolution,
		fovy: 90.0,
		znear: 0.001,
		zfar: 100000000.0,
	});

	renderer.update();
	let mut command_encoder = renderer.new_command_encoder(Some(&camera));
	command_encoder.begin_pass(&mut voxelization_pass);
	command_encoder.finish();

	let mut command_encoder = renderer.new_command_encoder(Some(&camera));
	command_encoder.begin_pass(&mut meshify_pass);
	command_encoder.finish();

	let mut instant = std::time::Instant::now();

	let mut w = ElementState::Released;
	let mut a = ElementState::Released;
	let mut s = ElementState::Released;
	let mut d = ElementState::Released;
	let mut c = ElementState::Released;
	let mut space = ElementState::Released;
	let mut render_voxels = false;

	event_loop.run(move |event, event_loop| {
		let new_instant = std::time::Instant::now();
		let delta_time = new_instant.duration_since(instant).as_secs_f64();
		instant = new_instant;

		let mut movement = glm::vec3(0.0, 0.0, 0.0);

		{
			match w {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation()),
						&glm::vec3(0.0, 0.0, 1.0),
					);

					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				}
				_ => {}
			}
			match a {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation()),
						&glm::vec3(1.0, 0.0, 0.0),
					);

					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				}
				_ => {}
			}
			match s {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation()),
						&glm::vec3(0.0, 0.0, -1.0),
					);

					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				}
				_ => {}
			}
			match d {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation()),
						&glm::vec3(-1.0, 0.0, 0.0),
					);

					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				}
				_ => {}
			}
			match c {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation()),
						&glm::vec3(0.0, 1.0, 0.0),
					);

					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				}
				_ => {}
			}
			match space {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation()),
						&glm::vec3(0.0, -1.0, 0.0),
					);

					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				}
				_ => {}
			}
		}
		camera.set_position(camera.position() + movement);

		match event {
			Event::WindowEvent {
				ref event,
				window_id,
			} if window_id == window.id() => match event {
				WindowEvent::Resized(size) => {
					// camera.aspect = ((window.inner_size().width as f32) / (window.inner_size().height as f32));
					renderer.resize(size.width, size.height);
				},
				WindowEvent::CloseRequested
				| WindowEvent::KeyboardInput {
					event:
						KeyEvent {
							state: ElementState::Pressed,
							physical_key: PhysicalKey::Code(KeyCode::Escape),
							..
						},
					..
				} => event_loop.exit(),
				WindowEvent::KeyboardInput {
					event:
						KeyEvent {
							state,
							physical_key: PhysicalKey::Code(KeyCode::KeyW),
							..
						},
					..
				} => w = state.to_owned(),
				WindowEvent::KeyboardInput {
					event:
						KeyEvent {
							state,
							physical_key: PhysicalKey::Code(KeyCode::KeyA),
							..
						},
					..
				} => a = state.to_owned(),
				WindowEvent::KeyboardInput {
					event:
						KeyEvent {
							state,
							physical_key: PhysicalKey::Code(KeyCode::KeyS),
							..
						},
					..
				} => s = state.to_owned(),
				WindowEvent::KeyboardInput {
					event:
						KeyEvent {
							state,
							physical_key: PhysicalKey::Code(KeyCode::KeyD),
							..
						},
					..
				} => d = state.to_owned(),
				WindowEvent::KeyboardInput {
					event:
						KeyEvent {
							state,
							physical_key: PhysicalKey::Code(KeyCode::KeyC),
							..
						},
					..
				} => c = state.to_owned(),
				WindowEvent::KeyboardInput {
					event:
						KeyEvent {
							state,
							physical_key: PhysicalKey::Code(KeyCode::Space),
							..
						},
					..
				} => space = state.to_owned(),
				WindowEvent::KeyboardInput {
					event:
						KeyEvent {
							state,
							physical_key: PhysicalKey::Code(KeyCode::KeyQ),
							..
						},
					..
				} => match *state {
					winit::event::ElementState::Pressed => {
						render_voxels = !render_voxels;
					},
					_ => {}
				},
				WindowEvent::KeyboardInput {
					event:
						KeyEvent {
							state: ElementState::Pressed,
							physical_key: PhysicalKey::Code(KeyCode::KeyF),
							..
						},
					..
				} => println!("{:#?}", camera.position()),

				_ => {}
			},
			Event::DeviceEvent { event, .. } => match event {
				DeviceEvent::MouseMotion { delta } => {
					let right = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation()),
						&glm::vec3(1.0, 0.0, 0.0),
					);
					let up = glm::vec3(0.0_f32, 1.0, 0.0);

					camera.set_rotation(glm::quat_rotate(&camera.rotation(), delta.0 as f32 * delta_time as f32 * CAMERA_ROTATION_SPEED, &up));
					camera.set_rotation(glm::quat_rotate(
						&camera.rotation(),
						delta.1 as f32 * delta_time as f32* CAMERA_ROTATION_SPEED,
						&right,
					));
				}
				_ => {}
			},
			Event::AboutToWait => {
				renderer.update();
				let mut command_encoder = renderer.new_command_encoder(Some(&camera));
				if render_voxels {
					command_encoder.begin_pass(&mut render_meshify_pass);
				} else {
					command_encoder.begin_pass(&mut forward_render_pass);
				}
				command_encoder.finish();
			},
			_ => {}
		}
}).unwrap();
}
