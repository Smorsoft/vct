extern crate nalgebra_glm as glm;

pub mod camera;
use camera::*;

pub mod renderer;
use renderer::*;

pub struct App {
	pub renderer: Renderer,
}

impl App {
	pub async fn new(window: &winit::window::Window) -> Self {
		Self {
			renderer: Renderer::new(window).await,
		}
	}

	pub fn load_gltf<P: AsRef<std::path::Path>>(
		&mut self,
		path: P,
		is_static: bool,
	) -> Vec<camera::Camera> {
		renderer::load_gltf::load_gltf(
			&self.renderer.context,
			&mut self.renderer.meshes,
			&mut self.renderer.materials,
			&self.renderer.model_bind_group_layout,
			&self.renderer.material_bind_group_layout,
			path,
			is_static,
		)
	}
}
