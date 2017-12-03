use ash::vk;
use ash::Device;
use ash::version::{V1_0, DeviceV1_0};
use ash::util::Align;
use cgmath::Point3;
use object::{Drawable, Position};
use renderer::RenderState;
use std::mem::{align_of, size_of};
use std::rc::Rc;

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

    // Keep a pointer to the device for cleanup
    device: Rc<Device<V1_0>>,
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
    pub fn new_quad(
        rs: &RenderState,
        position: Point3<f64>,
        width: f32,
        height: f32,
    ) -> DrawObject {
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

        // Create buffer for vertices
        let buffersize: vk::DeviceSize = (size_of::<Vertex>() * vertices.len()) as u64;
        let (vert_buffer, vert_mem) = rs.create_vk_buffer(
            buffersize,
            vk::BUFFER_USAGE_VERTEX_BUFFER_BIT,
            vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT |
                vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
        );

        unsafe {
            let vert_ptr = rs.device
                .map_memory(vert_mem, 0, buffersize, vk::MemoryMapFlags::empty())
                .expect("Failed to map vertex memory");
            let mut vert_align = Align::new(vert_ptr, align_of::<Vertex>() as u64, buffersize);
            vert_align.copy_from_slice(&vertices);
            rs.device.unmap_memory(vert_mem);
        }

        // Create buffer for indices
        let buffersize: vk::DeviceSize = (size_of::<u16>() * indices.len()) as u64;
        let (idx_buffer, idx_mem) = rs.create_vk_buffer(
            buffersize,
            vk::BUFFER_USAGE_INDEX_BUFFER_BIT,
            vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT |
                vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
        );
        unsafe {
            let idx_ptr = rs.device
                .map_memory(idx_mem, 0, buffersize, vk::MemoryMapFlags::empty())
                .expect("Failed to map index memory");
            let mut idx_align = Align::new(idx_ptr, align_of::<u16>() as u64, buffersize);
            idx_align.copy_from_slice(&indices);
            rs.device.unmap_memory(idx_mem);
        }

        DrawObject {
            vertices: vert_buffer,
            vertex_mem: vert_mem,
            indices: idx_buffer,
            index_mem: idx_mem,
            position: position,
            device: Rc::clone(&rs.device),
        }
    }
}

impl Drop for DrawObject {
    fn drop(&mut self) {
        // We cannot have the last reference to device at this point
        debug_assert!(1 < Rc::strong_count(&self.device));

        unsafe {
            self.device.destroy_buffer(self.indices, None);
            self.device.free_memory(self.index_mem, None);
            self.device.destroy_buffer(self.vertices, None);
            self.device.free_memory(self.vertex_mem, None);
        }
    }
}
