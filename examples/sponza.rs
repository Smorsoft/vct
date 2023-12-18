extern crate nalgebra_glm as glm;

use vct::*;
use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

const CAMERA_MOVE_SPEED: f32 = 10.0;

fn main() {
	use winit::{
		event::*,
		event_loop::{ControlFlow, EventLoop},
		window::WindowBuilder,
	};

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new().build(&event_loop).unwrap();

	let mut app = pollster::block_on(App::new(&window));
	let _cameras = app
		// .load_gltf("examples/sponza/NewSponza_Main_glTF_002.gltf", true);
		.load_gltf("examples/Box.glb", true);

	let mut camera = camera::Camera {
		position: [0.0, 0.0, -10.0].into(),
		rotation: glm::Quat::identity(),
		aspect: 16.0 / 9.0,
		fovy: 90.0,
		znear: 0.001,
		zfar: 100000000.0,
	};

	app.renderer.update_camera(&camera);

	let mut instant = std::time::Instant::now();

	let mut w = ElementState::Released;
	let mut a = ElementState::Released;
	let mut s = ElementState::Released;
	let mut d = ElementState::Released;
	let mut c = ElementState::Released;
	let mut space = ElementState::Released;

	event_loop.run(move |event, _, control_flow| {
		let new_instant = std::time::Instant::now();
		let delta_time = new_instant.duration_since(instant).as_secs_f64();
		instant = new_instant;

		let mut movement = glm::vec3(0.0, 0.0, 0.0);
		
		{
			match w {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation),
						&glm::vec3(0.0, 0.0, 1.0),
					);
	
					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				},
				_ => {}
			}
			match a {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation),
						&glm::vec3(1.0, 0.0, 0.0),
					);
					
					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				},
				_ => {}
			}
			match s {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation),
						&glm::vec3(0.0, 0.0, -1.0),
					);
					
					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				},
				_ => {}
			}
			match d {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation),
						&glm::vec3(-1.0, 0.0, 0.0),
					);

	
					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				},
				_ => {}
			}
			match c {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation),
						&glm::vec3(0.0, 1.0, 0.0),
					);
	
					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				},
				_ => {}
			}
			match space {
				ElementState::Pressed => {
					let direction = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation),
						&glm::vec3(0.0, -1.0, 0.0),
					);
	
					movement += direction * CAMERA_MOVE_SPEED * delta_time as f32;
				},
				_ => {}
			}
		}

		camera.position += movement;
		app.renderer.update_camera(&camera);

		match event {
			Event::WindowEvent {
				ref event,
				window_id,
			} if window_id == window.id() => match event {
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
				
				_ => {}
			},
			Event::DeviceEvent {event, ..} => match event {
				DeviceEvent::MouseMotion { delta } => {
					let right = glm::quat_rotate_vec3(
						&glm::quat_inverse(&camera.rotation),
						&glm::vec3(-1.0, 0.0, 0.0),
					);
					let up = glm::vec3(0.0_f32, -1.0, 0.0);

					camera.rotation = glm::quat_rotate(&camera.rotation, delta.0 as f32 * delta_time as f32, &up);
					camera.rotation = glm::quat_rotate(&camera.rotation, delta.1 as f32 * delta_time as f32, &right);

				},
				_ => {}
			},
			Event::MainEventsCleared => {
				app.renderer.render().unwrap();
			}
			_ => {}
		}
	});
}
