const WIDTH = 50.0;

struct Vertex {
	position: vec3<f32>,
	normal: VertexNormals,
	color: VertexColors,
};

struct Triangle {
	vertices: array<Vertex, 3>,
	bounds_min: vec3<f32>,
	bounds_max: vec3<f32>,
};

struct ModelUniform {
	matrix: mat4x4<f32>,
};

struct VertexNormals {
	normals: u32,
	tangents: u32,
};

struct VertexColors {
	uv0: vec2<f32>,
	uv1: vec2<f32>,
	color: u32,
};

@group(0) @binding(0)
var voxels_color: texture_storage_3d<rgba8unorm, write>;
// @group(0) @binding(1)
// var voxels_normal: texture_storage_3d<rgba8snorm, write>;
// @group(0) @binding(2)
// var voxels_emissive: texture_storage_3d<rgba8unorm, write>;

@group(1) @binding(0)
var<storage, read> v_indices: array<u32>;
@group(1) @binding(1)
var<storage, read> v_positions: array<f32>;
@group(1) @binding(2)
var<storage, read> v_normals: array<u32>;
@group(1) @binding(3)
var<storage, read> v_colors: array<u32>;

@group(2) @binding(0)
var<uniform> model_transform: ModelUniform;

@group(3) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(3) @binding(1)
var s_diffuse: sampler;
@group(3) @binding(2)
var t_metal: texture_2d<f32>;
@group(3) @binding(3)
var s_metal: sampler;
@group(3) @binding(4)
var t_normal: texture_2d<f32>;
@group(3) @binding(5)
var s_normal: sampler;

// Data per side
// Color RGBA8Unorm - alpha translucency
// Normal RGBA8Snorm - alpha metallic (reinterperet as unorm)
// Emissive RGBA8Unorm - alpha roughness



@compute @workgroup_size(1)
fn main(@builtin(workgroup_id) workgroup_id : vec3<u32>,) {
	var index = workgroup_id.x * 3u;
	var triangle = get_triangle(index);

	// var dx = triangle.bounds_max.x - triangle.bounds_min.x;
	// var dy = triangle.bounds_max.y - triangle.bounds_min.y;
	// var dz = triangle.bounds_max.z - triangle.bounds_min.z;

	var voxel_grid_dimension = f32(textureDimensions(voxels_color).x);

	var voxel_size = WIDTH / voxel_grid_dimension;

	var min = (triangle.bounds_min + (WIDTH / 2.0)) / voxel_size;
	var max = ((triangle.bounds_max + (WIDTH / 2.0)) / voxel_size);

	var color1 = vec4(textureGather(0, t_diffuse, s_diffuse, triangle.vertices[0].color.uv0).rgb, 1.0);
	var color2 = vec4(textureGather(0, t_diffuse, s_diffuse, triangle.vertices[1].color.uv0).rgb, 1.0);
	var color3 = vec4(textureGather(0, t_diffuse, s_diffuse, triangle.vertices[2].color.uv0).rgb, 1.0);

	var one = (triangle.vertices[0].position + (WIDTH / 2.0)) / voxel_size;
	var two = (triangle.vertices[1].position + (WIDTH / 2.0)) / voxel_size;
	var three = (triangle.vertices[2].position + (WIDTH / 2.0)) / voxel_size;

	textureStore(voxels_color, vec3(u32(one.x), u32(one.y), u32(one.z)), color1);
	textureStore(voxels_color, vec3(u32(two.x), u32(two.y), u32(two.z)), color2);
	textureStore(voxels_color, vec3(u32(three.x), u32(three.y), u32(three.z)), color3);

	// for (var x: u32 = u32(min.x); x < u32(max.x); x++) {
	// 	for (var y: u32 = u32(min.y); y < u32(max.y); y++) {
	// 		for (var z: u32 = u32(min.z); z < u32(max.z); z++) {
	// 			textureStore(voxels_color, vec3(x, y, z), color);
	// 		}
	// 	}
	// }
}

fn get_triangle(index: u32) -> Triangle {
	var triangle: Triangle;

	for (var i: u32 = 0u; i < 3u; i++) {
		var vertex: Vertex;
		vertex.position = (model_transform.matrix * vec4<f32>(read_vertex(index + i), 1.0)).xyz;
		vertex.normal = read_normal(index + i);
		vertex.color = read_color(index + i);
		triangle.vertices[i] = vertex;
	}

	// Calc bounds
	triangle.bounds_min = triangle.vertices[0].position;
	triangle.bounds_max = triangle.vertices[2].position;
	for (var i: u32 = 1u; i < 3u; i++) {
		// Min
		if (triangle.vertices[i].position.x < triangle.bounds_min.x) { triangle.bounds_min.x = triangle.vertices[i].position.x; }
		if (triangle.vertices[i].position.y < triangle.bounds_min.y) { triangle.bounds_min.y = triangle.vertices[i].position.y; }
		if (triangle.vertices[i].position.z < triangle.bounds_min.z) { triangle.bounds_min.z = triangle.vertices[i].position.z; }

		// Max
		if (triangle.vertices[i].position.x > triangle.bounds_max.x) { triangle.bounds_max.x = triangle.vertices[i].position.x; }
		if (triangle.vertices[i].position.y > triangle.bounds_max.y) { triangle.bounds_max.y = triangle.vertices[i].position.y; }
		if (triangle.vertices[i].position.z > triangle.bounds_max.z) { triangle.bounds_max.z = triangle.vertices[i].position.z; }
	}

	return triangle;
}

fn read_vertex(index: u32) -> vec3<f32> {
	return vec3(
		v_positions[index * 3u],
		v_positions[index * 3u + 1u],
		v_positions[index * 3u + 2u],
	);
}

fn read_normal(index: u32) -> VertexNormals {
	var normals: VertexNormals;

	normals.normals = v_normals[index * 2u];
	normals.tangents = v_normals[index * 2u + 1u];

	return normals;
}

fn read_color(index: u32) -> VertexColors {
	var colors: VertexColors;

	var new_index = index * 5u;

	colors.uv0 = vec2(bitcast<f32>(v_colors[new_index]), bitcast<f32>(v_colors[new_index + 1u]));
	colors.uv1 = vec2(bitcast<f32>(v_colors[new_index + 2u]), bitcast<f32>(v_colors[new_index + 3u]));
	colors.color = v_colors[new_index + 4u];

	return colors;
}