// Vertex Shader

struct CameraUniform {
	view_proj: mat4x4<f32>,
};

struct PointLight {
	position: vec3<f32>,
	color: vec3<f32>,
	intensity: f32,
};

const LIGHT: PointLight = PointLight(vec3(0.0, 1.0, 0.0), vec3(0.05, 0.05, 0.05), 1.0);

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model_matrix: mat4x4<f32>;

@group(1) @binding(1)
var<uniform> normal_matrix: mat3x3<f32>;

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

@group(3) @binding(0)
var voxels_color: texture_3d<f32>;



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
	@location(0) world_position: vec3<f32>,
	@location(1) normals: vec3<f32>,
	@location(2) tangents: vec4<f32>,
	@location(3) bitangents: vec3<f32>,
	@location(4) uv0: vec2<f32>,
	@location(5) uv1: vec2<f32>,
	@location(6) color: vec4<f32>,
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

	out.normals = unpack4x8snorm(normals.normals).xyz;
	out.tangents = unpack4x8snorm(normals.tangents);
	out.bitangents = cross(out.normals.xyz, out.tangents.xyz) * out.tangents.w;
	out.world_position = (model_matrix * vec4(position, 1.0)).xyz;

	// out.uv0 = unpack2x16unorm(color.uv0);
	// out.uv1 = unpack2x16unorm(color.uv1);
    out.color = unpack4x8unorm(color.color);

    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(position, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var diffuse: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.uv0);
	// Check for full transparency
    if diffuse.a == 0.0 {
		discard;
    }

	let tbn = mat3x3<f32>(
		normalize((model_matrix * normalize(in.tangents)).xyz), 
		normalize((model_matrix * normalize(vec4(in.bitangents, 0.0))).xyz),
		normalize((model_matrix * normalize(vec4(in.normals, 0.0))).xyz)
	);

	let ambient_strength = 0.1;
	let ambient_color = LIGHT.color * ambient_strength;

	let light_dir = normalize(LIGHT.position - in.world_position);

	let diffuse_strength = max(dot(normal_matrix * in.normals.xyz, light_dir), 0.0);
	let diffuse_color = LIGHT.color * diffuse_strength;

	let result = (ambient_color + diffuse_color) * diffuse.rgb;



    // var lv: vec4<f32> = vec4(0.0);
	// while (lv.a < 1.0) {

	// }



    return vec4(result, diffuse.a);
}