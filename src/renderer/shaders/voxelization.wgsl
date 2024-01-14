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
	y_order: array<u32, 3>,
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
fn main(@builtin(workgroup_id) workgroup_id: vec3<u32>) {
    var index = workgroup_id.x * 3u;

    var voxel_grid_dimension = f32(textureDimensions(voxels_color).x);
    var voxel_size = WIDTH / voxel_grid_dimension;

    var triangle = get_triangle(index, voxel_size);

    voxelize(triangle.vertices[0].grid_position, triangle.vertices[1].grid_position, triangle.vertices[2].grid_position);
}

fn get_triangle(index: u32, voxel_size: f32) -> Triangle {
    var triangle: Triangle;

    for (var i: u32 = 0u; i < 3u; i++) {
        var vertex: Vertex;
        vertex.position = (model_transform.matrix * vec4<f32>(read_vertex(index + i), 1.0)).xyz;
        vertex.grid_position = (vertex.position + (WIDTH / 2.0)) / voxel_size;
        vertex.normal = read_normal(index + i);
        vertex.color = read_color(index + i);
        triangle.vertices[i] = vertex;
    }

	// Calc bounds
    triangle.bounds_min = triangle.vertices[0].position;
    triangle.bounds_max = triangle.vertices[2].position;
    for (var i: u32 = 1u; i < 3u; i++) {
		// Min
        if triangle.vertices[i].position.x < triangle.bounds_min.x { triangle.bounds_min.x = triangle.vertices[i].position.x; }
        if triangle.vertices[i].position.y < triangle.bounds_min.y { triangle.bounds_min.y = triangle.vertices[i].position.y; }
        if triangle.vertices[i].position.z < triangle.bounds_min.z { triangle.bounds_min.z = triangle.vertices[i].position.z; }

		// Max
        if triangle.vertices[i].position.x > triangle.bounds_max.x { triangle.bounds_max.x = triangle.vertices[i].position.x; }
        if triangle.vertices[i].position.y > triangle.bounds_max.y { triangle.bounds_max.y = triangle.vertices[i].position.y;}
        if triangle.vertices[i].position.z > triangle.bounds_max.z { triangle.bounds_max.z = triangle.vertices[i].position.z; }
    }


    var dx = triangle.bounds_max.x - triangle.bounds_min.x;
    var dy = triangle.bounds_max.y - triangle.bounds_min.y;
    var dz = triangle.bounds_max.z - triangle.bounds_min.z;

    if dx > dy && dx > dz {
    } else if dy > dx && dy > dz {
    } else {
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

struct ScanlineCache {
	v1: vec3<f32>,
	v2: vec3<f32>,
	v3: vec3<f32>,

	unit_dir_12: vec3<f32>,
	unit_dir_13: vec3<f32>,
	unit_dir_23: vec3<f32>,

	proj_unit_dir_12: vec2<f32>,
	proj_unit_dir_13: vec2<f32>,
	proj_unit_dir_23: vec2<f32>,

	length_dir_12: f32,
	length_dir_13: f32,
	min_length_dir_23: f32,
	length_dir_23: f32,

	scanline_length: f32,
	scanline_inv_dir_axis: f32,
	scanline_max_length: f32,

	sl_inv_dot_dir_12: f32,
	sl_inv_dot_dir_13: f32,
	sl_inv_dot_dir_23: f32,
};

fn voxelize_point(v: vec3<i32>) {
    var color = vec4(textureGather(0, t_diffuse, s_diffuse, vec2(0.1)).rgb, 1.0);

    var voxel_grid_dimension = textureDimensions(voxels_color);

    if v.x < 0 || v.x >= i32(voxel_grid_dimension.x) || v.y < 0 || v.y >= i32(voxel_grid_dimension.y) || v.z < 0 || v.z >= i32(voxel_grid_dimension.z) {
        return;
    }

    textureStore(voxels_color, v, color);
}

fn voxelize_line(v1: vec3<f32>, v2: vec3<f32>) {
    let dir = v2 - v1;
    let dir_length = length(dir);
    let unit_dir = normalize(dir);
    let sign_dir = sign(dir);

    var plane_x: f32;
    var pos_x: i32;
    if dir.x < 0.0 {
        plane_x = ceil(v1.x - 1.0);
        pos_x = i32(plane_x);
    } else {
        plane_x = floor(v1.x + 1.0);
        pos_x = i32(plane_x - 1.0);
    }

    var plane_y: f32;
    var pos_y: i32;
    if dir.y < 0.0 {
        plane_y = ceil(v1.y - 1.0);
        pos_y = i32(plane_y);
    } else {
        plane_y = floor(v1.y + 1.0);
        pos_y = i32(plane_y - 1.0);
    }

    var plane_z: f32;
    var pos_z: i32;
    if dir.z < 0.0 {
        plane_z = ceil(v1.z - 1.0);
        pos_z = i32(plane_z);
    } else {
        plane_z = floor(v1.z + 1.0);
        pos_z = i32(plane_z - 1.0);
    }

    let next_plane = vec3(plane_x, plane_y, plane_z);

    var pos = vec3(pos_x, pos_y, pos_z);
    voxelize_point(pos);

    var t = (next_plane - v1) / unit_dir;
    let t_step = 1.0 / abs(unit_dir);

    let max_step = abs(dir.x) + abs(dir.y) + abs(dir.z) + 3.0;

    var i = 0;
    var step_length = 0.0;

    while (pos.x != i32(v2.x) || pos.y != i32(v2.y) || pos.z != i32(v2.z)) && step_length < dir_length {
        let t_min = min_from_vec(t);
        var axis = 0;
        if t_min == t.y {
            axis = 1;
        } else if t_min == t.z {
            axis = 2;
        }

        t = t - t_min;
        step_length += t_min;

        pos[axis] = pos[axis] + i32(sign_dir[axis]);
        voxelize_point(pos);
        t[axis] = t_step[axis];
        i++;
    }
}

fn calculate_scanline(v1: vec3<f32>, v2: vec3<f32>, v3: vec3<f32>, axis: i32, cache: ptr<function, ScanlineCache>) {
    var dir12 = v2 - v1;
    var dir13 = v3 - v1;
    var dir23 = v3 - v2;

    (*cache).unit_dir_12 = normalize(dir12);
    (*cache).unit_dir_13 = normalize(dir13);
    (*cache).unit_dir_23 = normalize(dir23);

    let axis_x = (axis + 1) % 3; // Not really the x axis
    let axis_y = (axis + 2) % 3; // Not really the y axis

    (*cache).proj_unit_dir_12 = vec2((*cache).unit_dir_12[axis_x], (*cache).unit_dir_12[axis_y]);
    (*cache).proj_unit_dir_13 = vec2((*cache).unit_dir_13[axis_x], (*cache).unit_dir_13[axis_y]);
    (*cache).proj_unit_dir_23 = vec2((*cache).unit_dir_23[axis_x], (*cache).unit_dir_23[axis_y]);

    var sl_dir: vec3<f32>;
    if v1[axis] != v3[axis] {
        sl_dir = cross(dir12, dir13);
        let z = sl_dir[axis];
        sl_dir = -sl_dir * sign(z) / length(vec2(sl_dir[axis_x], sl_dir[axis_y]));
        sl_dir[axis] = abs(1.0 / sl_dir[axis]);
    } else {
        sl_dir = dir13 / length(vec2(dir13[axis_x], dir13[axis_y]));
    }

	// exact scanline length can cause missing scanlines due to rounding error
    (*cache).scanline_length = (abs(sl_dir[axis_x]) + abs(sl_dir[axis_y])) * 0.999;

    let proj_sl_dir = vec2(sl_dir[axis_x], sl_dir[axis_y]);

	// Recalculate v2 so that the scanline always start from v1
	// I.e. make dot(p2-v2, proj_sl_dir) = 0
    let v2_new = v2 - (*cache).unit_dir_23 * dot(proj_sl_dir, vec2(dir12[axis_x], dir12[axis_y])) / dot(proj_sl_dir, (*cache).proj_unit_dir_23);
    dir23 = v3 - v2_new;

    (*cache).v1 = v1;
    (*cache).v2 = v2_new;
    (*cache).v3 = v3;
    (*cache).sl_inv_dot_dir_12 = 1.0 / dot(proj_sl_dir, (*cache).proj_unit_dir_12);
    (*cache).sl_inv_dot_dir_13 = 1.0 / dot(proj_sl_dir, (*cache).proj_unit_dir_13);
    (*cache).sl_inv_dot_dir_23 = 1.0 / dot(proj_sl_dir, (*cache).proj_unit_dir_23);
    (*cache).length_dir_12 = length(dir12);
    (*cache).length_dir_13 = length(dir13);
    (*cache).min_length_dir_23 = length(v2 - v2_new);
    (*cache).length_dir_23 = length(v3 - v2_new);
    (*cache).scanline_inv_dir_axis = abs(1.0 / sl_dir[axis]);
    (*cache).scanline_max_length = dot(proj_sl_dir, vec2(dir13[axis_x], dir13[axis_y]));
}

fn voxelize_scan_line(cache: ptr<function, ScanlineCache>, sl_length: f32, axis: i32, height: f32) {
    var from_pos = (*cache).v1;
    var from_dir = (*cache).unit_dir_12;
    var inv_dot = (*cache).sl_inv_dot_dir_12;

    if sl_length * (*cache).sl_inv_dot_dir_12 >= (*cache).length_dir_12 || sl_length * (*cache).sl_inv_dot_dir_12 < 0.0 {
		// If this also out of range assume we are outside the triangle
        if sl_length * (*cache).sl_inv_dot_dir_23 >= (*cache).length_dir_23 || sl_length * (*cache).sl_inv_dot_dir_23 <= (*cache).min_length_dir_23 {
            return;
        }

        from_pos = (*cache).v2;
        from_dir = (*cache).unit_dir_23;
        inv_dot = (*cache).sl_inv_dot_dir_23;
    }

    var from_vec = from_pos + from_dir * sl_length * inv_dot;
    var to = (*cache).v1 + (*cache).unit_dir_13 * sl_length * (*cache).sl_inv_dot_dir_13;

    from_vec[axis] = height;
    to[axis] = height;

    voxelize_line(from_vec, to);
}

fn voxelize_interior(v1: vec3<f32>, v2: vec3<f32>, v3: vec3<f32>, axis: i32) {
    let next_plane = floor(v1[axis] + 1.0);

    var cache: ScanlineCache;
    calculate_scanline(v1, v2, v3, axis, &cache);

    var plane_t = (next_plane - v1[axis]) * cache.scanline_inv_dir_axis;
    var t = cache.scanline_length;

    var plane = next_plane - 0.5;

	// Triangle froms a line
    if cache.scanline_max_length <= 0.0 || cache.scanline_max_length >= 99.0 {
        return;
    }

    while true {
        t = min(t, min(plane_t, cache.scanline_max_length));
        if t >= cache.scanline_max_length { // Fully voxelized
			break;
		}
        if t == plane_t { // New scanline slice
            voxelize_scan_line(&cache, t, axis, plane);
            plane += 1.0;
            plane_t += cache.scanline_inv_dir_axis;
		}

        voxelize_scan_line(&cache, t, axis, plane);
        t += cache.scanline_length;
    }
}

fn voxelize(in_v1: vec3<f32>, in_v2: vec3<f32>, in_v3: vec3<f32>) {
    var v1 = in_v1;
    var v2 = in_v2;
    var v3 = in_v3;


    let normal = abs(cross(v2 - v1, v3 - v1));
    let dom_normal = max_from_vec(normal);
    var axis = 0;
    if dom_normal == normal.x {
        axis = 0;
    } else if dom_normal == normal.y {
        axis = 1;
    } else if dom_normal == normal.z {
        axis = 2;
    }

	// ----- Sort by most dominant axis
    if v1[axis] > v2[axis] {
        let tmp = v1;
        v1 = v2;
        v2 = tmp;
    }
    if v1[axis] > v3[axis] {
        let tmp = v1;
        v1 = v3;
        v3 = v1;
    }
    if v2[axis] > v3[axis] {
        let tmp = v2;
        v2 = v3;
        v3 = tmp;
    }

    voxelize_line(v1, v2);
    voxelize_line(v2, v3);
    voxelize_line(v1, v3);

    voxelize_interior(v1, v2, v3, axis);
}

fn min_from_vec(v: vec3<f32>) -> f32 {
    return min(v.x, min(v.y, v.z));
}

fn max_from_vec(v: vec3<f32>) -> f32 {
    return max(v.x, max(v.y, v.z));
}