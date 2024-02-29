struct DrawCmd {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: u32,
    first_instance: u32,
};

struct CmdBuffer {
    commands: array<DrawCmd>,
};