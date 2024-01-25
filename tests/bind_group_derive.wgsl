@group(0)
@binding(0)
var<storage, read_write> v_indices: mat4x4<f32>; // this is used as both input and output for convenience

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
	v_indices = v_indices * 2.0;
}