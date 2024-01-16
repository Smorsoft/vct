const WIDTH = 50.0;

struct Voxel {
	center: vec3<f32>,
};

struct Vertex {
	position: vec3<f32>,
	grid_position: vec3<f32>,
	normal: VertexNormals,
	color: VertexColors,
};

struct Triangle {
	vertices: array<Vertex, 3>,
	bounds_min: vec3<f32>,
	bounds_max: vec3<f32>,
    dom_axis: i32,
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
fn main(@builtin(workgroup_id) workgroup_id: vec3<u32>) {
    var indices_index = (workgroup_id.x * 3u);

    var voxel_grid_dimension = f32(textureDimensions(voxels_color).x);
    var voxel_size = WIDTH / voxel_grid_dimension;

    var triangle = get_triangle(indices_index, voxel_size);

    voxelize_line(triangle, triangle.vertices[0].grid_position, triangle.vertices[1].grid_position);
    voxelize_line(triangle, triangle.vertices[1].grid_position, triangle.vertices[2].grid_position);
    voxelize_line(triangle, triangle.vertices[0].grid_position, triangle.vertices[2].grid_position);

    voxelize_interior(triangle);
}

fn get_triangle(indices_index: u32, voxel_size: f32) -> Triangle {
    var triangle: Triangle;

    for (var i: u32 = 0u; i < 3u; i++) {
        var vertex: Vertex;
        let index: u32 = v_indices[indices_index + i];
        vertex.position = (model_transform.matrix * vec4<f32>(read_vertex(index), 1.0)).xyz;
        vertex.grid_position = (vertex.position + (WIDTH / 2.0)) / voxel_size;
        vertex.normal = read_normal(index);
        vertex.color = read_color(index);
        triangle.vertices[i] = vertex;
    }

	// Calc bounds
    triangle.bounds_min = triangle.vertices[0].position;
    triangle.bounds_max = triangle.vertices[0].position;
    for (var i: i32 = 1; i < 3; i++) {
		// Min
        if triangle.vertices[i].position.x < triangle.bounds_min.x { triangle.bounds_min.x = triangle.vertices[i].position.x; }
        if triangle.vertices[i].position.y < triangle.bounds_min.y { triangle.bounds_min.y = triangle.vertices[i].position.y; }
        if triangle.vertices[i].position.z < triangle.bounds_min.z { triangle.bounds_min.z = triangle.vertices[i].position.z; }

		// Max
        if triangle.vertices[i].position.x > triangle.bounds_max.x { triangle.bounds_max.x = triangle.vertices[i].position.x; }
        if triangle.vertices[i].position.y > triangle.bounds_max.y { triangle.bounds_max.y = triangle.vertices[i].position.y; }
        if triangle.vertices[i].position.z > triangle.bounds_max.z { triangle.bounds_max.z = triangle.vertices[i].position.z; }
    }

    let delta_bounds = triangle.bounds_max - triangle.bounds_min;

    triangle.dom_axis = 0;
    if (delta_bounds.y > delta_bounds.x && delta_bounds.y > delta_bounds.z) {
        triangle.dom_axis = 1;
    } else if (delta_bounds.z > delta_bounds.x && delta_bounds.z > delta_bounds.y) {
        triangle.dom_axis = 2;
    }

    if triangle.vertices[0].position[triangle.dom_axis] > triangle.vertices[1].position[triangle.dom_axis] {
        let tmp = triangle.vertices[0];
        triangle.vertices[0] = triangle.vertices[1];
        triangle.vertices[1] = tmp;
    }
    if triangle.vertices[0].position[triangle.dom_axis] > triangle.vertices[2].position[triangle.dom_axis] {
        let tmp = triangle.vertices[0];
        triangle.vertices[0] = triangle.vertices[2];
        triangle.vertices[2] = tmp;
    }
    if triangle.vertices[1].position[triangle.dom_axis] > triangle.vertices[2].position[triangle.dom_axis] {
        let tmp = triangle.vertices[1];
        triangle.vertices[1] = triangle.vertices[2];
        triangle.vertices[2] = tmp;
    }

    return triangle;
}

fn voxelize_point(triangle: Triangle, in_v: vec3<i32>, f_v: vec3<f32>) {
    let v0 = triangle.vertices[1].grid_position - triangle.vertices[0].grid_position;
    let v1 = triangle.vertices[2].grid_position - triangle.vertices[0].grid_position;
    let v2 = f_v - triangle.vertices[0].grid_position;

    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    let denom = d00 * d11 - d01 * d01;
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    let uv0 = (triangle.vertices[0].color.uv0 * u) + (triangle.vertices[1].color.uv0 * v) + (triangle.vertices[2].color.uv0 * w);

    let color_r = textureGather(0, t_diffuse, s_diffuse, uv0);
    let color_g = textureGather(1, t_diffuse, s_diffuse, uv0);
    let color_b = textureGather(2, t_diffuse, s_diffuse, uv0);
    let color_a = textureGather(3, t_diffuse, s_diffuse, uv0);

    let color = vec4(
        (color_r.x + color_r.y + color_r.z + color_r.w) / 4.0,
        (color_g.x + color_g.y + color_g.z + color_g.w) / 4.0,
        (color_b.x + color_b.y + color_b.z + color_b.w) / 4.0,
        (color_a.x + color_a.y + color_a.z + color_a.w) / 4.0,
    );

    let texture_dim = textureDimensions(t_diffuse);

    textureStore(voxels_color, in_v, color);
}

fn voxelize_line(triangle: Triangle, v1: vec3<f32>, v2: vec3<f32>) {
    let dir = v2 - v1;
    let dir_length = length(dir);
    let unit_dir = normalize(dir);
    let sign_dir = sign(dir);

    let floored_start = floor(v1);
    let floored_end = floor(v2);
    var pos = vec3(i32(floored_start.x), i32(floored_start.y), i32(floored_start.z));
    let end_pos = vec3(i32(floored_end.x), i32(floored_end.y), i32(floored_end.z));

    voxelize_point(triangle, pos, v1);

    let next_plane = ceil_or_floor_vec3(dir, v1);

    var t = (next_plane - v1) / unit_dir;
    let t_step = 1.0 / abs(unit_dir);

    let max_step = abs(dir.x) + abs(dir.y) + abs(dir.z) + 3.0;

    var i = 0;
    var step_length = 0.0;

    while !compare_vec(pos, end_pos) && step_length < dir_length {
        let t_min = min(t.x, min(t.y, t.z));
        var axis = 0;
        if t_min == t.y {
            axis = 1;
        } else if t_min == t.z {
            axis = 2;
        }

        t = t - t_min;
        step_length += t_min;

        pos[axis] = pos[axis] + i32(sign_dir[axis]);
        voxelize_point(triangle, pos, v1 + (unit_dir * step_length));
        t[axis] = t_step[axis];
        i++;
    }
}

fn voxelize_interior(triangle: Triangle) {
    // TODO: Consider running a scanline between every dom_axis step while voxelizeing triangle edges
    let next_plane = floor(triangle.vertices[0].grid_position[triangle.dom_axis] + 1.0);
    let max_plane = triangle.vertices[2].grid_position[triangle.dom_axis] - 0.5;

    var plane = next_plane + 0.5;

    let dir01 = triangle.vertices[0].grid_position - triangle.vertices[1].grid_position;
    let dir02 = triangle.vertices[0].grid_position - triangle.vertices[2].grid_position;
    let dir12 = triangle.vertices[1].grid_position - triangle.vertices[2].grid_position;

    let unit_dir01 = normalize(dir01);
    let unit_dir02 = normalize(dir02);
    let unit_dir12 = normalize(dir12);

    let dom_length_01 = triangle.vertices[1].grid_position[triangle.dom_axis] - triangle.vertices[0].grid_position[triangle.dom_axis];
    let dom_length_02 = triangle.vertices[2].grid_position[triangle.dom_axis] - triangle.vertices[0].grid_position[triangle.dom_axis];
    let dom_length_12 = triangle.vertices[2].grid_position[triangle.dom_axis] - triangle.vertices[1].grid_position[triangle.dom_axis];

    while plane < max_plane {
        let t_02 = (plane - triangle.vertices[0].grid_position[triangle.dom_axis]) / dom_length_02;
        let p_02 = triangle.vertices[0].grid_position - (t_02 * dir02);

        var end: vec3<f32>;
        if triangle.vertices[1].grid_position[triangle.dom_axis] >= plane {
            let t_01 = (plane - triangle.vertices[0].grid_position[triangle.dom_axis]) / dom_length_01;
            let p_01 = triangle.vertices[0].grid_position - (t_01 * dir01);
            end = p_01;
        } else {
            let t_12 = (plane - triangle.vertices[1].grid_position[triangle.dom_axis]) / dom_length_12;
            let p_12 = triangle.vertices[1].grid_position - (t_12 * dir12);
            end = p_12;
        }

        voxelize_line(triangle, p_02, end);

        plane += 1.0;
    }
}

fn compare_vec(v1: vec3<i32>, v2: vec3<i32>) -> bool {
    if v1.x == v2.x && v1.y == v2.y && v1.z == v2.z {
        return true;
    } else {
        return false;
    }
}

fn i32v(in: vec3<f32>) -> vec3<i32> {
    return vec3(i32(in.x), i32(in.y), i32(in.z));
}

fn debug_by_color(color: vec3<f32>) {
    for (var x: i32 = 0; x < 50; x++) {
        for (var y: i32 = 0; y < 50; y++) {
            for (var z: i32 = 0; z < 50; z++) {
                textureStore(voxels_color, vec3(x, y, z), vec4(color, 1.0));
            }
        }
    }
}

fn ceil_or_floor(direction: f32, value: f32) -> f32 {
    if direction < 0.0 {
        return ceil(value - 1.0);
    } else {
        return floor(value + 1.0);
    }
}

fn ceil_or_floor_vec3(direction: vec3<f32>, vector: vec3<f32>) -> vec3<f32> {
    return vec3(
        ceil_or_floor(direction.x, vector.x),
        ceil_or_floor(direction.y, vector.y),
        ceil_or_floor(direction.z, vector.z),
    );
}