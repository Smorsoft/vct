struct TestHostShareable {
	first: u32,
	second: mat4x4<f32>,
};

@group(0)
@binding(0)
var<storage, read_write> test_in: TestHostShareable; // this is used as both input and output for convenience

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
	test_in.first = test_in.first * 2u;
	test_in.second = test_in.second * 3.0;
}