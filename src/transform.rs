
pub struct Transform {
	id: crate::Id,
}

pub(crate) struct InternalTransform {
	pub position: glm::Vec3,
	pub rotation: glm::Vec3,
	pub scale: glm::Vec3,
	pub dirty: bool,
}