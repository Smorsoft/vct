// Vertex Shader

struct CameraUniform {
	view_proj: mat4x4<f32>,
};


struct ModelUniform {
	matrix: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model_transform: ModelUniform;

struct VertexInput {
	@location(0) position: vec3<f32>,
	@location(1) color: vec3<f32>,
};

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
	model: VertexInput,
) -> VertexOutput {
	var out: VertexOutput;
	out.color = model.color;
	out.clip_position = camera.view_proj * model_transform.matrix * vec4<f32>(model.position, 1.0);
	return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(in.color, 1.0);
}