extern crate nalgebra_glm as glm;

use gltf::{camera::Projection, *};
use vct::*;
use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

fn main() {
	pollster::block_on(run());
}

async fn run() {
	let mut gltf_cameras = Vec::new();
	// let mut gltf_materials = Vec::new();
	// let mut gltf_meshes = Vec::new();
	

	let base = Gltf::open("examples/sponza/NewSponza_Main_glTF_002.gltf").unwrap();
	let curtains = Gltf::open("examples/sponza/NewSponza_Curtains_glTF.gltf").unwrap();
	for scene in base.scenes() {
		for node in scene.nodes() {
			if node.camera().is_some() {
				gltf_cameras.push((node.camera(), node.transform()));
			} else {
				for primitive in node.mesh().unwrap().primitives() {
					// primitive.
				}
			}
		}
	}

	// let (camera, camera_transform) = camera.unwrap();
	// let decomposed = camera_transform.decomposed();
	// let projection = match camera.projection() {
	// 	Projection::Perspective(proj) => proj,
	// 	_ => panic!(),
	// };
	
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.build(&event_loop)
		.expect("Failed to create the window");

	// let mut state = State::new(window).await;

	// let camera = state.new_camera(
	// 	decomposed.0.to_owned().into(),
	// 	projection.yfov(),
	// 	projection.aspect_ratio().unwrap(),
	// 	projection.zfar().unwrap(),
	// 	projection.znear(),
	// 	glm::Quat::from_parts(
	// 		decomposed.1[3],
	// 		glm::Vec3::new(decomposed.1[0], decomposed.1[1], decomposed.1[2]),
	// 	),
	// );

	// camera.update(&mut state.queue);

}
