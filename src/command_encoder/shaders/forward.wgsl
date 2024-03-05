const WIDTH = 50.0;
const VOXEL_SIZE = 0.09765625;

// Vertex Shader

struct CameraUniform {
	view_proj: mat4x4<f32>,
};

struct PointLight {
	position: vec3<f32>,
	color: vec3<f32>,
	intensity: f32,
};

const LIGHT: PointLight = PointLight(vec3(0.0, 1.5, 0.0), vec3(1.0, 1.0, 1.0), 1.0);

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
@group(3) @binding(1)
var voxels_color_s: sampler;

struct VertexNormals {
	@location(1) normals: vec3<f32>,
	@location(2) tangents: vec4<f32>,
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
	@location(3) uv0: vec2<f32>,
	@location(4) uv1: vec2<f32>,
	@location(5) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    normals: VertexNormals,
    color: VertexColors,
) -> VertexOutput {
    var out: VertexOutput;
	// out.uv0 = unpack2x16unorm(color.uv0);
	// out.uv1 = unpack2x16unorm(color.uv1);
    out.uv0 = color.uv0;
    out.uv1 = color.uv1;
    out.color = unpack4x8unorm(color.color);

	// out.normals = normalize(unpack4x8snorm(normals.normals).xyz);
	// out.tangents = normalize(unpack4x8snorm(normals.tangents));
    out.normals = normalize(normal_matrix * normals.normals);
    out.tangents = vec4(normalize(normal_matrix * normals.tangents.xyz), normals.tangents.w);
    out.world_position = (model_matrix * vec4(position, 1.0)).xyz;


    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(position, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var bitangents = normalize(normal_matrix * (cross(in.normals, in.tangents.xyz) * in.tangents.w));

    let tbn = mat3x3<f32>(
        normalize(in.tangents.xyz),
        normalize(bitangents),
        normalize(in.normals),
    );

    var dif: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.uv0);
    var nrm: vec4<f32> = normalize(textureSample(t_normal, s_normal, in.uv0) * 2.0 - 1.0);
    var pixel_normal: vec3<f32> = normalize(tbn * nrm.xyz);

	// Check for full transparency
    if dif.a == 0.0 {
		discard;
    }

    let color = dif.rgb * diffuse_trace(in.world_position, pixel_normal);

    return vec4(color, 1.0);
}

const DIFFUSE_OFFSET: f32 = 0.01;
const TILT_FACTOR: f32 = 0.5;
fn diffuse_trace(position: vec3<f32>, nrm: vec3<f32>) -> vec3<f32> {
    let origin = position + (nrm * DIFFUSE_OFFSET);

    let y = nrm;
    let x = normalize(cross(y, vec3(0.0, 0.0, 1.0)));
    let z = normalize(cross(y, x));

    var color = vec3(0.0);
    
    // Front
    color += trace_diffuse_cone(origin, y);

    // Sides
    color += trace_diffuse_cone(origin, x);
    color += trace_diffuse_cone(origin, -x);
    color += trace_diffuse_cone(origin, z);
    color += trace_diffuse_cone(origin, -z);

    // Intermediate
    color += trace_diffuse_cone(origin, mix(y, x, TILT_FACTOR));
    color += trace_diffuse_cone(origin, mix(y, -x, TILT_FACTOR));
    color += trace_diffuse_cone(origin, mix(y, z, TILT_FACTOR));
    color += trace_diffuse_cone(origin, mix(y, -z, TILT_FACTOR));

    return color * (5.0 / 9.0);
}

const CONE_SPREAD: f32 = 0.325;
const MAX_DISTANCE: f32 = 30.0;
fn trace_diffuse_cone(start: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    var dist: f32 = 0.1953125;

    var color: vec3<f32> = vec3(0.0);
    var occlusion = 0.0;
	while (dist < MAX_DISTANCE && occlusion < 1.0) {
		var c = start + (dir * dist);
        var diameter = CONE_SPREAD * dist;
        dist += diameter;
		
        let voxel = vol_sample(diameter, c);

        let v_color = voxel.rgb;
        let v_occlusion = voxel.a;

        if diameter * 1.2 >= abs(distance(c, LIGHT.position)) {
            let light_dir = normalize(LIGHT.position - start);
            let diffuse_strength = max(dot(dir, light_dir), 0.0);

            if dot(dir, light_dir) > 0.25 {
                return LIGHT.color * (1.0 - occlusion);
            }
        }

        color = occlusion*color + (1.0 - occlusion) * v_occlusion * v_color;
        occlusion = occlusion + (1.0 - occlusion) * v_occlusion;
	}

    return color;
}

fn vol_sample(diameter: f32, position: vec3<f32>) -> vec4<f32> {
    let voxels_dim = textureDimensions(voxels_color).x;
    let voxels_size = WIDTH / f32(voxels_dim);

    let pos = (position + (WIDTH / 2.0)) / voxels_size;

    var vlevel = log2(diameter / voxels_size);
    vlevel = min(f32(textureNumLevels(voxels_color) - 1u), vlevel);

    return textureSampleLevel(voxels_color, voxels_color_s, pos, vlevel);
}