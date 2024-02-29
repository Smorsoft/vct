
#[derive(Debug, Copy, Clone)]
pub struct PointLight {
	pub position: [f32; 3],
	pub color: [f32; 3],
	pub intensity: f32,
}

impl PointLight {
	pub fn to_uniform(self) -> PointLightUniform {
		PointLightUniform {
			position: self.position,
			_padding: 0,
			color: self.color,
			_padding2: 0,
			intensity: self.intensity,
		}
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightUniform {
	position: [f32; 3],
	_padding: u32,
	color: [f32; 3],
	_padding2: u32,
	intensity: f32,
} 