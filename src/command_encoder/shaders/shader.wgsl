// Vertex Shader

struct CameraUniform {
	view_proj: mat4x4<f32>,
};


struct ModelUniform {
	matrix: mat4x4<f32>,
};

struct PointLight {
	position: vec3<f32>,
	color: vec3<f32>,
	intensity: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model_transform: ModelUniform;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;
@group(2) @binding(2)
var t_metal: texture_2d<f32>;
@group(2) @binding(3)
var s_metal: sampler;
@group(2) @binding(4)
var t_normal: texture_2d<f32>;
@group(2) @binding(5)
var s_normal: sampler;


// @group(2) @binding(0)
// var<uniform> point_lights: array<PointLight, 23>;

struct VertexNormals {
	@location(1) normals: u32,
	@location(2) tangents: u32,
};

struct VertexColors {
	@location(3) uv0: vec2<f32>,
	@location(4) uv1: vec2<f32>,
	@location(5) color: u32,
};

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) uv0: vec2<f32>,
	@location(1) uv1: vec2<f32>,
	@location(2) color: vec4<f32>,
};

@vertex
fn vs_main(
	@builtin(instance_index) instance_index: u32, 
	@location(0) position: vec3<f32>,
	normals: VertexNormals,
	color: VertexColors,
) -> VertexOutput {
	var out: VertexOutput;
	out.uv0 = color.uv0;
	out.uv1 = color.uv1;

	// out.uv0 = unpack2x16unorm(color.uv0);
	// out.uv1 = unpack2x16unorm(color.uv1);
	out.color = unpack4x8unorm(color.color);

	out.clip_position = camera.view_proj * model_transform.matrix * vec4<f32>(position, 1.0);
	return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(t_diffuse, s_diffuse, in.uv0);
}