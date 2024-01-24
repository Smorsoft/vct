mod material;
mod mesh;

pub struct Model {
	pub mesh: mesh::Mesh,
	pub transform: crate::transform::Transform,
}

pub trait LoadModel {
	
}