use ash::vk;
use ash::version::DeviceV1_0;
use ash::util::Align;
use cgmath::Point3;
use object::{Drawable, Position};
use renderer::RenderState;
use std::ptr;
use std::mem::{align_of, size_of};

#[derive(Clone, Copy)]
struct Vertex {
    pos: [f32; 4],
    normal: [f32; 4],
    tex_coord: [f32; 2],
}

pub struct DrawObject {
    vertices: vk::Buffer,
    vertex_mem: vk::DeviceMemory,
    indices: vk::Buffer,
    index_mem: vk::DeviceMemory,

    position: Point3<f64>,
}

//TODO: move somewhere else
unsafe fn create_vk_buffer(
    rs: &RenderState,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> (vk::Buffer, vk::DeviceMemory) {
    let bufferinfo = vk::BufferCreateInfo {
        s_type: vk::StructureType::BufferCreateInfo,
        p_next: ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size: size,
        usage: usage,
        sharing_mode: vk::SharingMode::Exclusive,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
    };
    let buffer = rs.device.create_buffer(&bufferinfo, None).expect(
        "Failed to create buffer",
    );

    let mem_req = rs.device.get_buffer_memory_requirements(buffer);
    let alloc_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MemoryAllocateInfo,
        p_next: ptr::null(),
        allocation_size: mem_req.size,
        memory_type_index: 0, //TODO! FIX THIS!
    };
    let memory = rs.device.allocate_memory(&alloc_info, None).expect(
        "Failed to allocate buffer memory",
    );

    rs.device.bind_buffer_memory(buffer, memory, 0);

    (buffer, memory)
}

impl Drawable for DrawObject {
    fn draw(&self) {}
}

impl Position for DrawObject {
    fn get_position(&self) -> Point3<f64> {
        self.position
    }

    fn set_position(&mut self, position: Point3<f64>) {
        self.position = position;
    }
}

impl DrawObject {
    fn new_quad(rs: &RenderState, position: Point3<f64>, width: f32, height: f32) -> DrawObject {
        let vertices = [
            Vertex {
                pos: [-width, -height, 0.0, 1.0],
                normal: [0.0, 0.0, 1.0, 1.0],
                tex_coord: [0.0, 0.0],
            },
            Vertex {
                pos: [width, -height, 0.0, 1.0],
                normal: [0.0, 0.0, 1.0, 1.0],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                pos: [-width, height, 0.0, 1.0],
                normal: [0.0, 0.0, 1.0, 1.0],
                tex_coord: [0.0, 1.0],
            },
            Vertex {
                pos: [width, height, 0.0, 1.0],
                normal: [0.0, 0.0, 1.0, 1.0],
                tex_coord: [1.0, 1.0],
            },
        ];
        let indices = [0u16, 1, 3, 0, 3, 2];

        unsafe {
            // Create buffer for vertices
            let buffersize: vk::DeviceSize = (size_of::<Vertex>() * vertices.len()) as u64;
            let (vert_buffer, vert_mem) = create_vk_buffer(
                rs,
                buffersize,
                vk::BUFFER_USAGE_VERTEX_BUFFER_BIT,
                vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT |
                    vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
            );
            let vert_ptr = rs.device
                .map_memory(vert_mem, 0, buffersize, vk::MemoryMapFlags::empty())
                .expect("Failed to map vertex memory");
            let mut vert_align = Align::new(vert_ptr, align_of::<Vertex>() as u64, buffersize);
            vert_align.copy_from_slice(&vertices);
            rs.device.unmap_memory(vert_mem);

            // Create buffer for indices
            let buffersize: vk::DeviceSize = (size_of::<u16>() * indices.len()) as u64;
            let (idx_buffer, idx_mem) = create_vk_buffer(
                rs,
                buffersize,
                vk::BUFFER_USAGE_INDEX_BUFFER_BIT,
                vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT |
                    vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
            );
            let idx_ptr = rs.device
                .map_memory(idx_mem, 0, buffersize, vk::MemoryMapFlags::empty())
                .expect("Failed to map index memory");
            let mut idx_align = Align::new(idx_ptr, align_of::<u16>() as u64, buffersize);
            idx_align.copy_from_slice(&indices);
            rs.device.unmap_memory(idx_mem);

            DrawObject {
                vertices: vert_buffer,
                vertex_mem: vert_mem,
                indices: idx_buffer,
                index_mem: idx_mem,
                position: position,
            }
        }
    }
}
