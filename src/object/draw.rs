use ash::vk;
use ash::Device;
use ash::version::{DeviceV1_0, V1_0};
use cgmath::{Deg, Matrix4, Point3, Quaternion, Rotation3, Vector3};
use object::{Drawable, Position, Rotation};
use renderer::{MainPass, RenderState, Texture};
use std::{mem, ptr, slice, f32};
use std::rc::Rc;

#[derive(Clone, Copy)]
#[allow(dead_code)] //not going to change vertices after creation
pub struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 3],
    bitangent: [f32; 3],
    tex_uv: [f32; 2],
}

pub struct DrawObject {
    vertices: vk::Buffer,
    vertex_mem: vk::DeviceMemory,
    indices: vk::Buffer,
    index_mem: vk::DeviceMemory,
    num_indices: u32,

    position: Point3<f32>,
    rotation: Quaternion<f32>,

    descriptor_sets: Vec<vk::DescriptorSet>,
    texture: Texture,
    normal_map: Texture,

    // Keep a pointer to the device for cleanup
    device: Rc<Device<V1_0>>,
}

impl Drawable for DrawObject {
    fn draw(
        &self,
        cmd_buf: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        view_matrix: &Matrix4<f32>,
        projection_matrix: &Matrix4<f32>,
    ) {
        let model_rotation_matrix = Matrix4::from(self.rotation);
        let model_translation_matrix = Matrix4::from_translation(
            self.get_position() - Point3::<f32> {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        // The order of multiplication here is important!
        let model_matrix = model_translation_matrix * model_rotation_matrix;
        let mvp_matrix = projection_matrix * view_matrix * model_matrix;
        let matrices = [model_matrix, mvp_matrix];

        let matrices_bytes;
        unsafe {
            matrices_bytes =
                slice::from_raw_parts(matrices.as_ptr() as *const u32, mem::size_of_val(&matrices));
        }

        unsafe {
            self.device.cmd_push_constants(
                cmd_buf,
                pipeline_layout,
                vk::SHADER_STAGE_VERTEX_BIT,
                0,
                matrices_bytes,
            );
            self.device.cmd_bind_descriptor_sets(
                cmd_buf,
                vk::PipelineBindPoint::Graphics,
                pipeline_layout,
                0,
                &self.descriptor_sets[..],
                &[],
            );
            self.device
                .cmd_bind_vertex_buffers(cmd_buf, 0, &[self.vertices], &[0]);
            self.device
                .cmd_bind_index_buffer(cmd_buf, self.indices, 0, vk::IndexType::Uint16);
            self.device
                .cmd_draw_indexed(cmd_buf, self.num_indices, 1, 0, 0, 1);
        }
    }
}

impl Position for DrawObject {
    fn get_position(&self) -> Point3<f32> {
        self.position
    }

    fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
    }
}

impl Rotation for DrawObject {
    fn rotate(&mut self, axis: Vector3<f32>, angle: Deg<f32>) {
        let rotation_quat = Quaternion::from_axis_angle(axis, angle);
        // The order here is important
        self.rotation = rotation_quat * self.rotation;
    }

    fn get_rotation(&self) -> Quaternion<f32> {
        self.rotation
    }
}

impl DrawObject {
    /// Creates a new quad draw object.
    //    pub fn new_quad(
    //        rs: &RenderState,
    //        position: Point3<f32>,
    //        width: f32,
    //        height: f32,
    //    ) -> DrawObject {
    //        let vertices = [
    //            Vertex {
    //                pos: [-width, -height, 0.0],
    //                normal: [0.0, 0.0, 1.0],
    //                tangent: [1.0, 0.0, 0.0],
    //                bitangent: [0.0, 1.0, 0.0],
    //                tex_uv: [0.0, 0.0],
    //            },
    //            Vertex {
    //                pos: [width, -height, 0.0],
    //                normal: [0.0, 0.0, 1.0],
    //                tangent: [1.0, 0.0, 0.0],
    //                bitangent: [0.0, 1.0, 0.0],
    //                tex_uv: [1.0, 0.0],
    //            },
    //            Vertex {
    //                pos: [-width, height, 0.0],
    //                normal: [0.0, 0.0, 1.0],
    //                tangent: [1.0, 0.0, 0.0],
    //                bitangent: [0.0, 1.0, 0.0],
    //                tex_uv: [0.0, 1.0],
    //            },
    //            Vertex {
    //                pos: [width, height, 0.0],
    //                normal: [0.0, 0.0, 1.0],
    //                tangent: [1.0, 0.0, 0.0],
    //                bitangent: [0.0, 1.0, 0.0],
    //                tex_uv: [1.0, 1.0],
    //            },
    //        ];
    //        let indices = [0u16, 1, 3, 0, 3, 2];
    //
    //        // Create buffer for vertices
    //        let (vert_buffer, vert_mem) = rs.create_buffer_and_upload(
    //            vk::BUFFER_USAGE_VERTEX_BUFFER_BIT,
    //            vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
    //            &vertices,
    //            true,
    //        );
    //
    //        // Create buffer for indices
    //        let (idx_buffer, idx_mem) = rs.create_buffer_and_upload(
    //            vk::BUFFER_USAGE_INDEX_BUFFER_BIT,
    //            vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
    //            &indices,
    //            true,
    //        );
    //
    //        DrawObject {
    //            vertices: vert_buffer,
    //            vertex_mem: vert_mem,
    //            indices: idx_buffer,
    //            index_mem: idx_mem,
    //            position: position,
    //            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
    //            num_indices: indices.len() as u32,
    //            device: Rc::clone(&rs.device),
    //        }
    //    }

    pub fn new_cuboid(
        rs: &RenderState,
        mp: &MainPass,
        position: Point3<f32>,
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
                pos: [-h_width, -h_height, h_depth], //Lower Left Front
                normal: [0.0, 0.0, 1.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.25, 2.0 / 3.0],
            },
            Vertex {
                pos: [h_width, -h_height, h_depth], //Lower Right Front
                normal: [0.0, 0.0, 1.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.5, 2.0 / 3.0],
            },
            Vertex {
                pos: [-h_width, h_height, h_depth], //Upper Left Front
                normal: [0.0, 0.0, 1.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.25, 1.0 / 3.0],
            },
            Vertex {
                pos: [h_width, h_height, h_depth], //Upper Right Front
                normal: [0.0, 0.0, 1.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.5, 1.0 / 3.0],
            },
            //Back
            Vertex {
                pos: [h_width, -h_height, -h_depth], //Lower Right Back
                normal: [0.0, 0.0, -1.0],
                tangent: [-1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.75, 2.0 / 3.0],
            },
            Vertex {
                pos: [-h_width, -h_height, -h_depth], //Lower Left Back
                normal: [0.0, 0.0, -1.0],
                tangent: [-1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [1.0, 2.0 / 3.0],
            },
            Vertex {
                pos: [h_width, h_height, -h_depth], //Upper Right Back
                normal: [0.0, 0.0, -1.0],
                tangent: [-1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.75, 1.0 / 3.0],
            },
            Vertex {
                pos: [-h_width, h_height, -h_depth], //Upper Left Back
                normal: [0.0, 0.0, -1.0],
                tangent: [-1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [1.0, 1.0 / 3.0],
            },
            //Top
            Vertex {
                pos: [-h_width, h_height, h_depth], //Upper Left Front
                normal: [0.0, 1.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, -1.0],
                tex_uv: [0.25, 1.0 / 3.0],
            },
            Vertex {
                pos: [h_width, h_height, h_depth], //Upper Right Front
                normal: [0.0, 1.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, -1.0],
                tex_uv: [0.5, 1.0 / 3.0],
            },
            Vertex {
                pos: [-h_width, h_height, -h_depth], //Upper Left Back
                normal: [0.0, 1.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, -1.0],
                tex_uv: [0.25, 0.0],
            },
            Vertex {
                pos: [h_width, h_height, -h_depth], //Upper Right Back
                normal: [0.0, 1.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, -1.0],
                tex_uv: [0.5, 0.0],
            },
            //Bottom
            Vertex {
                pos: [-h_width, -h_height, -h_depth], //Lower Left Back
                normal: [0.0, -1.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 1.0],
                tex_uv: [0.25, 1.0],
            },
            Vertex {
                pos: [h_width, -h_height, -h_depth], //Lower Right Back
                normal: [0.0, -1.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 1.0],
                tex_uv: [0.5, 1.0],
            },
            Vertex {
                pos: [-h_width, -h_height, h_depth], //Lower Left Front
                normal: [0.0, -1.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 1.0],
                tex_uv: [0.25, 2.0 / 3.0],
            },
            Vertex {
                pos: [h_width, -h_height, h_depth], //Lower Right Front
                normal: [0.0, -1.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 1.0],
                tex_uv: [0.5, 2.0 / 3.0],
            },
            //Right
            Vertex {
                pos: [h_width, -h_height, h_depth], //Lower Right Front
                normal: [1.0, 0.0, 0.0],
                tangent: [0.0, 0.0, -1.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.5, 2.0 / 3.0],
            },
            Vertex {
                pos: [h_width, -h_height, -h_depth], //Lower Right Back
                normal: [1.0, 0.0, 0.0],
                tangent: [0.0, 0.0, -1.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.75, 2.0 / 3.0],
            },
            Vertex {
                pos: [h_width, h_height, h_depth], //Upper Right Front
                normal: [1.0, 0.0, 0.0],
                tangent: [0.0, 0.0, -1.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.5, 1.0 / 3.0],
            },
            Vertex {
                pos: [h_width, h_height, -h_depth], //Upper Right Back
                normal: [1.0, 0.0, 0.0],
                tangent: [0.0, 0.0, -1.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.75, 1.0 / 3.0],
            },
            //Left
            Vertex {
                pos: [-h_width, -h_height, -h_depth], //Lower Left Back
                normal: [-1.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 1.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.0, 2.0 / 3.0],
            },
            Vertex {
                pos: [-h_width, -h_height, h_depth], //Lower Left Front
                normal: [-1.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 1.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.25, 2.0 / 3.0],
            },
            Vertex {
                pos: [-h_width, h_height, -h_depth], //Upper Left Back
                normal: [-1.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 1.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.0, 1.0 / 3.0],
            },
            Vertex {
                pos: [-h_width, h_height, h_depth], //Upper Left Front
                normal: [-1.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 1.0],
                bitangent: [0.0, 1.0, 0.0],
                tex_uv: [0.25, 1.0 / 3.0],
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

        let desc_alloc_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DescriptorSetAllocateInfo,
            p_next: ptr::null(),
            descriptor_pool: mp.descriptor_pool,
            descriptor_set_count: mp.descriptor_set_layouts.len() as u32,
            p_set_layouts: mp.descriptor_set_layouts.as_ptr(),
        };
        let descriptor_sets;
        unsafe {
            descriptor_sets = rs.device
                .allocate_descriptor_sets(&desc_alloc_info)
                .unwrap();
        }

        let texture = rs.load_image("assets/cubemap.png");
        let texture_descriptor = vk::DescriptorImageInfo {
            image_layout: texture.current_layout,
            image_view: texture.view,
            sampler: texture.sampler,
        };

        let normal_map = rs.load_image("assets/cubemap_normals.png");
        let normal_descriptor = vk::DescriptorImageInfo {
            image_layout: normal_map.current_layout,
            image_view: normal_map.view,
            sampler: normal_map.sampler,
        };

        let write_desc_sets = [
            vk::WriteDescriptorSet {
                s_type: vk::StructureType::WriteDescriptorSet,
                p_next: ptr::null(),
                dst_set: descriptor_sets[0],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::CombinedImageSampler,
                p_image_info: &texture_descriptor,
                p_buffer_info: ptr::null(),
                p_texel_buffer_view: ptr::null(),
            },
            vk::WriteDescriptorSet {
                s_type: vk::StructureType::WriteDescriptorSet,
                p_next: ptr::null(),
                dst_set: descriptor_sets[0],
                dst_binding: 1,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::CombinedImageSampler,
                p_image_info: &normal_descriptor,
                p_buffer_info: ptr::null(),
                p_texel_buffer_view: ptr::null(),
            },
        ];
        unsafe {
            rs.device.update_descriptor_sets(&write_desc_sets, &[]);
        }

        DrawObject {
            vertices: vert_buffer,
            vertex_mem: vert_mem,
            indices: idx_buffer,
            index_mem: idx_mem,
            num_indices: indices.len() as u32,
            position: position,
            rotation: Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), Deg(0.0)),
            descriptor_sets: descriptor_sets,
            texture: texture,
            normal_map: normal_map,
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
            self.device.destroy_sampler(self.normal_map.sampler, None);
            self.device.destroy_image_view(self.normal_map.view, None);
            self.device.destroy_image(self.normal_map.image, None);
            self.device.free_memory(self.normal_map.memory, None);

            self.device.destroy_sampler(self.texture.sampler, None);
            self.device.destroy_image_view(self.texture.view, None);
            self.device.destroy_image(self.texture.image, None);
            self.device.free_memory(self.texture.memory, None);

            self.device.destroy_buffer(self.indices, None);
            self.device.free_memory(self.index_mem, None);
            self.device.destroy_buffer(self.vertices, None);
            self.device.free_memory(self.vertex_mem, None);
        }
    }
}
