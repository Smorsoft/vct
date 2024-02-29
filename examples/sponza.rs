extern crate nalgebra_glm as glm;

use std::collections::HashMap;

use vct::{camera::CameraDescriptor, *};
use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

const CAMERA_MOVE_SPEED: f32 = 10.0;
const CAMERA_ROTATION_SPEED: f32 = 30.0;

fn main() {
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new().build(&event_loop).unwrap();
	window
		.set_cursor_grab(winit::window::CursorGrabMode::Confined)
		.unwrap();

	window.set_maximized(true);

	let render_settings = RendererSettings {
		resolution: [1920, 1080],
		extras: HashMap::new(),
	};

	let mut renderer = pollster::block_on(Renderer::new(&window, render_settings));
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

	event_loop.run(move |event, _, control_flow| {
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
					input:
						KeyboardInput {
							state: ElementState::Pressed,
							virtual_keycode: Some(VirtualKeyCode::Escape),
							..
						},
					..
				} => *control_flow = ControlFlow::Exit,
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state,
							virtual_keycode: Some(VirtualKeyCode::W),
							..
						},
					..
				} => w = state.to_owned(),
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state,
							virtual_keycode: Some(VirtualKeyCode::A),
							..
						},
					..
				} => a = state.to_owned(),
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state,
							virtual_keycode: Some(VirtualKeyCode::S),
							..
						},
					..
				} => s = state.to_owned(),
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state,
							virtual_keycode: Some(VirtualKeyCode::D),
							..
						},
					..
				} => d = state.to_owned(),
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state,
							virtual_keycode: Some(VirtualKeyCode::C),
							..
						},
					..
				} => c = state.to_owned(),
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state,
							virtual_keycode: Some(VirtualKeyCode::Space),
							..
						},
					..
				} => space = state.to_owned(),
				WindowEvent::KeyboardInput {
					input: KeyboardInput {
						state,
						virtual_keycode: Some(VirtualKeyCode::Q),
						..
					},
					..
				} => match *state {
					winit::event::ElementState::Pressed => {
						render_voxels = !render_voxels;
					},
					_ => {}
				}

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
			Event::MainEventsCleared => {
				renderer.update();
				let mut command_encoder = renderer.new_command_encoder(Some(&camera));
				if render_voxels {
					command_encoder.begin_pass(&mut render_meshify_pass);
				} else {
					command_encoder.begin_pass(&mut forward_render_pass);
				}
				command_encoder.finish();
			}
			_ => {}
		}
	});
}
