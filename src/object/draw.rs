use ash::vk;
use ash::Device;
use ash::version::{DeviceV1_0, V1_0};
use cgmath::Point3;
use object::{Drawable, Position};
use renderer::RenderState;
use std::rc::Rc;


#[derive(Clone, Copy)]
#[allow(dead_code)] //not going to change vertices after creation
pub struct Vertex {
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

    num_indices: u32,

    // Keep a pointer to the device for cleanup
    device: Rc<Device<V1_0>>,
}

impl Drawable for DrawObject {
    fn draw(&self, cmd_buf: vk::CommandBuffer) {
        unsafe {
            self.device
                .cmd_bind_vertex_buffers(cmd_buf, 0, &[self.vertices], &[0]);
            self.device
                .cmd_bind_index_buffer(cmd_buf, self.indices, 0, vk::IndexType::Uint32);
            self.device
                .cmd_draw_indexed(cmd_buf, self.num_indices, 1, 0, 0, 1);
        }
    }
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
    /// Creates a new quad draw object.
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
        let (vert_buffer, vert_mem) = rs.create_buffer_and_upload(
            vk::BUFFER_USAGE_VERTEX_BUFFER_BIT,
            vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            &vertices,
            true,
        );

        // Create buffer for indices
        let (idx_buffer, idx_mem) = rs.create_buffer_and_upload(
            vk::BUFFER_USAGE_INDEX_BUFFER_BIT,
            vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            &indices,
            true,
        );

        DrawObject {
            vertices: vert_buffer,
            vertex_mem: vert_mem,
            indices: idx_buffer,
            index_mem: idx_mem,
            position: position,
            num_indices: indices.len() as u32,
            device: Rc::clone(&rs.device),
        }
    }

    pub fn new_cuboid(
        rs: &RenderState,
        position: Point3<f64>,
        width: f32,
        height: f32,
        depth: f32,
    ) -> DrawObject {
        let h_width = width / 2.0;
        let h_height = height / 2.0;
        let h_depth = depth / 2.0;
        let vertices = [
            //Front
            Vertex {
                pos: [-h_width, -h_height, -h_depth, 1.0], //Lower Left Front
                normal: [0.0, 0.0, -1.0, 1.0],
                tex_coord: [0.0, 0.0],
            },
            Vertex {
                pos: [h_width, -h_height, -h_depth, 1.0], //Lower Right Front
                normal: [0.0, 0.0, -1.0, 1.0],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                pos: [-h_width, h_height, -h_depth, 1.0], //Upper Left Front
                normal: [0.0, 0.0, -1.0, 1.0],
                tex_coord: [0.0, 1.0],
            },
            Vertex {
                pos: [h_width, h_height, -h_depth, 1.0], //Upper Right Front
                normal: [0.0, 0.0, -1.0, 1.0],
                tex_coord: [1.0, 1.0],
            },
            //Back
            Vertex {
                pos: [-h_width, -h_height, h_depth, 1.0], //Lower Left Back
                normal: [0.0, 0.0, 1.0, 1.0],
                tex_coord: [0.0, 0.0],
            },
            Vertex {
                pos: [h_width, -h_height, h_depth, 1.0], //Lower Right Back
                normal: [0.0, 0.0, 1.0, 1.0],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                pos: [-h_width, h_height, h_depth, 1.0], //Upper Left Back
                normal: [0.0, 0.0, 1.0, 1.0],
                tex_coord: [0.0, 1.0],
            },
            Vertex {
                pos: [h_width, h_height, h_depth, 1.0], //Upper Right Back
                normal: [0.0, 0.0, 1.0, 1.0],
                tex_coord: [1.0, 1.0],
            },
            //Top
            Vertex {
                pos: [-h_width, h_height, -h_depth, 1.0], //Upper Left Front
                normal: [0.0, 1.0, 0.0, 1.0],
                tex_coord: [0.0, 1.0],
            },
            Vertex {
                pos: [h_width, h_height, -h_depth, 1.0], //Upper Right Front
                normal: [0.0, 1.0, 0.0, 1.0],
                tex_coord: [1.0, 1.0],
            },
            Vertex {
                pos: [-h_width, h_height, h_depth, 1.0], //Upper Left Back
                normal: [0.0, 1.0, 0.0, 1.0],
                tex_coord: [0.0, 1.0],
            },
            Vertex {
                pos: [h_width, h_height, h_depth, 1.0], //Upper Right Back
                normal: [0.0, 1.0, 0.0, 1.0],
                tex_coord: [1.0, 1.0],
            },
            //Bottom
            Vertex {
                pos: [-h_width, -h_height, -h_depth, 1.0], //Lower Left Front
                normal: [0.0, -1.0, 0.0, 1.0],
                tex_coord: [0.0, 0.0],
            },
            Vertex {
                pos: [h_width, -h_height, -h_depth, 1.0], //Lower Right Front
                normal: [0.0, -1.0, 0.0, 1.0],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                pos: [-h_width, -h_height, h_depth, 1.0], //Lower Left Back
                normal: [0.0, -1.0, 0.0, 1.0],
                tex_coord: [0.0, 0.0],
            },
            Vertex {
                pos: [h_width, -h_height, h_depth, 1.0], //Lower Right Back
                normal: [0.0, -1.0, 0.0, 1.0],
                tex_coord: [1.0, 0.0],
            },
            //Right
            Vertex {
                pos: [h_width, -h_height, -h_depth, 1.0], //Lower Right Front
                normal: [1.0, 0.0, 0.0, 1.0],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                pos: [h_width, -h_height, h_depth, 1.0], //Lower Right Back
                normal: [1.0, 0.0, 0.0, 1.0],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                pos: [h_width, h_height, -h_depth, 1.0], //Upper Right Front
                normal: [1.0, 0.0, 0.0, 1.0],
                tex_coord: [1.0, 1.0],
            },
            Vertex {
                pos: [h_width, h_height, h_depth, 1.0], //Upper Right Back
                normal: [1.0, 0.0, 0.0, 1.0],
                tex_coord: [1.0, 1.0],
            },
            //Left
            Vertex {
                pos: [-h_width, -h_height, -h_depth, 1.0], //Lower Left Front
                normal: [-1.0, 0.0, 0.0, 1.0],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                pos: [-h_width, -h_height, h_depth, 1.0], //Lower Left Back
                normal: [-1.0, 0.0, 0.0, 1.0],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                pos: [-h_width, h_height, -h_depth, 1.0], //Upper Left Front
                normal: [-1.0, 0.0, 0.0, 1.0],
                tex_coord: [1.0, 1.0],
            },
            Vertex {
                pos: [-h_width, h_height, h_depth, 1.0], //Upper Left Back
                normal: [-1.0, 0.0, 0.0, 1.0],
                tex_coord: [1.0, 1.0],
            },
        ];
        let indices = [
            //Front
            0u16,
            1,
            2,
            2,
            1,
            3,
            //Back
            4,
            5,
            6,
            6,
            5,
            7,
            //Top
            8,
            9,
            10,
            10,
            9,
            11,
            //Bottom
            12,
            13,
            14,
            14,
            13,
            15,
            //Right
            16,
            17,
            18,
            18,
            17,
            19,
            //Left
            20,
            21,
            22,
            22,
            21,
            23,
        ];

        // Create buffer for vertices
        let (vert_buffer, vert_mem) = rs.create_buffer_and_upload(
            vk::BUFFER_USAGE_VERTEX_BUFFER_BIT,
            vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            &vertices,
            true,
        );

        // Create buffer for indices
        let (idx_buffer, idx_mem) = rs.create_buffer_and_upload(
            vk::BUFFER_USAGE_INDEX_BUFFER_BIT,
            vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            &indices,
            true,
        );

        DrawObject {
            vertices: vert_buffer,
            vertex_mem: vert_mem,
            indices: idx_buffer,
            index_mem: idx_mem,
            position: position,
            num_indices: indices.len() as u32,
            device: Rc::clone(&rs.device),
        }
    }
}

impl Drop for DrawObject {
    /// Drops the DrawObject by freeing the index and vertex buffers.
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
