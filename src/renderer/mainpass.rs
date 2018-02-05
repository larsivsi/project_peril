use ash::vk;
use ash::Device;
use ash::extensions::Surface;
use ash::version::{DeviceV1_0, V1_0};
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;

use renderer::RenderState;
use renderer::PresentPass;

use config::Config;

pub struct MainPass {
    renderpass: vk::RenderPass,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
    descriptor_sets: Vec<vk::DescriptorSet>,
    pipeline_layout: vk::PipelineLayout,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    pipeline: vk::Pipeline,
    //one framebuffer/commandbuffer per image
    framebuffers: Vec<vk::Framebuffer>,
    commandbuffers: Vec<vk::CommandBuffer>,

    //ImageView to render to.
    pub render_image: vk::Image,
    pub render_image_view: vk::ImageView,
    render_mem: vk::DeviceMemory,
    pub render_sampler: vk::Sampler,

    // Keep a pointer to the device for cleanup
    device: Rc<Device<V1_0>>,

    texture_image: vk::Image,
    texture_image_view: vk::ImageView,
    texture_mem: vk::DeviceMemory,
    texture_sampler: vk::Sampler,
}

impl MainPass {
    fn create_renderimages(
        rs: &RenderState,
        surface_format: &vk::SurfaceFormatKHR,
        render_size: &vk::Rect2D,
    ) -> (vk::Image, vk::DeviceMemory, vk::ImageView, vk::Sampler) {
        let image_extent;
        {
            let image_dims = (render_size.extent.width, render_size.extent.height);
            image_extent = vk::Extent3D {
                width: image_dims.0,
                height: image_dims.1,
                depth: 1,
            };
        }
        let (image, image_mem, image_view, image_sampler) = rs.create_texture(
            image_extent,
            vk::ImageType::Type2d,
            vk::ImageViewType::Type2d,
            surface_format.format,
            vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT | vk::IMAGE_USAGE_SAMPLED_BIT,
            vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
            vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            vk::ImageLayout::ColorAttachmentOptimal,
            None,
        );

        (image, image_mem, image_view, image_sampler)
    }

    /// Creates a main renderpass.
    ///
    /// * `rs`              The RenderState.
    /// * `surface_format`  The format of the surface.
    fn create_renderpass(
        rs: &RenderState,
        surface_format: &vk::SurfaceFormatKHR,
    ) -> vk::RenderPass {
        // One attachment, color only. Will produce the presentable image.
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: surface_format.format,
                flags: vk::AttachmentDescriptionFlags::empty(),
                samples: vk::SAMPLE_COUNT_1_BIT,
                load_op: vk::AttachmentLoadOp::Clear,
                store_op: vk::AttachmentStoreOp::Store,
                stencil_load_op: vk::AttachmentLoadOp::DontCare,
                stencil_store_op: vk::AttachmentStoreOp::DontCare,
                initial_layout: vk::ImageLayout::ColorAttachmentOptimal,
                final_layout: vk::ImageLayout::ShaderReadOnlyOptimal,
            },
        ];
        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::ColorAttachmentOptimal,
        };
        let subpass = vk::SubpassDescription {
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: ptr::null(),
            flags: Default::default(),
            pipeline_bind_point: vk::PipelineBindPoint::Graphics,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        };
        let renderpass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RenderPassCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            attachment_count: renderpass_attachments.len() as u32,
            p_attachments: renderpass_attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 0,
            p_dependencies: ptr::null(),
        };
        let renderpass;
        unsafe {
            renderpass = rs.device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap();
        }

        renderpass
    }

    /// Creates a pipeline for the renderpass.
    ///
    /// Very straigt forward pipeline: Loads some hard-coded shaders that will draw a triangle.
    ///
    /// * `rs`            The RenderState.
    /// * `surface_size`  The size of the surface to render to.
    /// * `renderpass`    The renderpass to produce the pipeline for (these have to match).
    fn create_pipeline(
        rs: &RenderState,
        surface_size: vk::Rect2D,
        renderpass: vk::RenderPass,
        texture_view: vk::ImageView,
        texture_sampler: vk::Sampler,
    ) -> (
        vk::DescriptorPool,
        Vec<vk::DescriptorSetLayout>,
        Vec<vk::DescriptorSet>,
        vk::PipelineLayout,
        vk::Viewport,
        vk::Rect2D,
        vk::Pipeline,
    ) {
        // Descriptors
        let descriptor_sizes = [
            vk::DescriptorPoolSize {
                typ: vk::DescriptorType::CombinedImageSampler,
                descriptor_count: 1,
            },
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DescriptorPoolCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            pool_size_count: descriptor_sizes.len() as u32,
            p_pool_sizes: descriptor_sizes.as_ptr(),
            max_sets: 1,
        };
        let descriptor_pool;
        unsafe {
            descriptor_pool = rs.device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .unwrap();
        }
        let desc_layout_bindings = [
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::CombinedImageSampler,
                descriptor_count: 1,
                stage_flags: vk::SHADER_STAGE_FRAGMENT_BIT,
                p_immutable_samplers: ptr::null(),
            },
        ];
        let descriptor_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DescriptorSetLayoutCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            binding_count: desc_layout_bindings.len() as u32,
            p_bindings: desc_layout_bindings.as_ptr(),
        };
        let descriptor_set_layouts;
        unsafe {
            descriptor_set_layouts = [
                rs.device
                    .create_descriptor_set_layout(&descriptor_info, None)
                    .unwrap(),
            ];
        }
        let desc_alloc_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DescriptorSetAllocateInfo,
            p_next: ptr::null(),
            descriptor_pool: descriptor_pool,
            descriptor_set_count: descriptor_set_layouts.len() as u32,
            p_set_layouts: descriptor_set_layouts.as_ptr(),
        };
        let descriptor_sets;
        unsafe {
            descriptor_sets = rs.device
                .allocate_descriptor_sets(&desc_alloc_info)
                .unwrap();
        }
        let tex_descriptor = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::General,
            image_view: texture_view,
            sampler: texture_sampler,
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
                p_image_info: &tex_descriptor,
                p_buffer_info: ptr::null(),
                p_texel_buffer_view: ptr::null(),
            },
        ];
        unsafe {
            rs.device.update_descriptor_sets(&write_desc_sets, &[]);
        }

        let layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PipelineLayoutCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            set_layout_count: descriptor_set_layouts.len() as u32,
            p_set_layouts: descriptor_set_layouts.as_ptr(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };

        let pipeline_layout;
        unsafe {
            pipeline_layout = rs.device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap();
        }

        let vertex_shader_module = rs.load_shader("shaders/final_pass_vert.spv");
        let fragment_shader_module = rs.load_shader("shaders/final_pass_frag.spv");

        let shader_entry_name = CString::new("main").unwrap();
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PipelineShaderStageCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                module: vertex_shader_module,
                p_name: shader_entry_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::SHADER_STAGE_VERTEX_BIT,
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PipelineShaderStageCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                module: fragment_shader_module,
                p_name: shader_entry_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::SHADER_STAGE_FRAGMENT_BIT,
            },
        ];
        let vertex_input_binding_descriptions = [];
        let vertex_input_attribute_descriptions = [];
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PipelineVertexInputStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            vertex_attribute_description_count: vertex_input_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr(),
            vertex_binding_description_count: vertex_input_binding_descriptions.len() as u32,
            p_vertex_binding_descriptions: vertex_input_binding_descriptions.as_ptr(),
        };
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PipelineInputAssemblyStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            primitive_restart_enable: 0,
            topology: vk::PrimitiveTopology::TriangleList,
        };
        let viewport = vk::Viewport {
            x: surface_size.offset.x as f32,
            y: surface_size.offset.y as f32,
            width: surface_size.extent.width as f32,
            height: surface_size.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = surface_size.clone();
        let viewport_state_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PipelineViewportStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            scissor_count: 1,
            p_scissors: &scissor,
            viewport_count: 1,
            p_viewports: &viewport,
        };
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PipelineRasterizationStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            cull_mode: vk::CULL_MODE_NONE,
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_enable: 0,
            depth_bias_slope_factor: 0.0,
            depth_clamp_enable: 0,
            front_face: vk::FrontFace::CounterClockwise,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::Fill,
            rasterizer_discard_enable: 0,
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PipelineMultisampleStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            rasterization_samples: vk::SAMPLE_COUNT_1_BIT,
            sample_shading_enable: 0,
            min_sample_shading: 0.0,
            p_sample_mask: ptr::null(),
            alpha_to_one_enable: 0,
            alpha_to_coverage_enable: 0,
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::Keep,
            pass_op: vk::StencilOp::Keep,
            depth_fail_op: vk::StencilOp::Keep,
            compare_op: vk::CompareOp::Always,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PipelineDepthStencilStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LessOrEqual,
            depth_bounds_test_enable: 0,
            stencil_test_enable: 0,
            front: noop_stencil_state.clone(),
            back: noop_stencil_state.clone(),
            max_depth_bounds: 1.0,
            min_depth_bounds: 0.0,
        };
        let color_blend_attachment_states = [
            vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SrcColor,
                dst_color_blend_factor: vk::BlendFactor::OneMinusDstColor,
                color_blend_op: vk::BlendOp::Add,
                src_alpha_blend_factor: vk::BlendFactor::Zero,
                dst_alpha_blend_factor: vk::BlendFactor::Zero,
                alpha_blend_op: vk::BlendOp::Add,
                color_write_mask: vk::ColorComponentFlags::all(),
            },
        ];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PipelineColorBlendStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            logic_op_enable: 0,
            logic_op: vk::LogicOp::Clear,
            attachment_count: color_blend_attachment_states.len() as u32,
            p_attachments: color_blend_attachment_states.as_ptr(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
        };
        let dynamic_state = [vk::DynamicState::Viewport, vk::DynamicState::Scissor];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PipelineDynamicStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            dynamic_state_count: dynamic_state.len() as u32,
            p_dynamic_states: dynamic_state.as_ptr(),
        };
        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GraphicsPipelineCreateInfo,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stage_create_infos.len() as u32,
            p_stages: shader_stage_create_infos.as_ptr(),
            p_vertex_input_state: &vertex_input_state_info,
            p_input_assembly_state: &vertex_input_assembly_state_info,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_state_info,
            p_rasterization_state: &rasterization_info,
            p_multisample_state: &multisample_state_info,
            p_depth_stencil_state: &depth_state_info,
            p_color_blend_state: &color_blend_state,
            p_dynamic_state: &dynamic_state_info,
            layout: pipeline_layout,
            render_pass: renderpass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: 0,
        };
        let graphics_pipelines;
        unsafe {
            graphics_pipelines = rs.device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info],
                    None,
                )
                .expect("Unable to create graphics pipeline");

            // Graphics pipeline created, we no longer need the shader modules
            rs.device
                .destroy_shader_module(fragment_shader_module, None);
            rs.device.destroy_shader_module(vertex_shader_module, None);
        }

        (
            descriptor_pool,
            descriptor_set_layouts.to_vec(),
            descriptor_sets,
            pipeline_layout,
            viewport,
            scissor,
            graphics_pipelines[0],
        )
    }

    /// Creates framebuffers for the presentable images, one per image.
    ///
    /// * `rs`                   The RenderState.
    /// * `surface_size`         The size of the surface to render to.
    /// * `present_image_views`  Imageviews to produce framebuffers for (one
    ///                          framebuffer per imageview).
    /// * `renderpass`           The renderpass to produce framebuffers for.
    fn create_framebuffer(
        rs: &RenderState,
        surface_size: vk::Rect2D,
        image_view: vk::ImageView,
        renderpass: vk::RenderPass,
    ) -> Vec<vk::Framebuffer> {
        let mut framebuffers: Vec<vk::Framebuffer> = Vec::new();

        let framebuffer_attachments = [image_view];
        let frame_buffer_create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FramebufferCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            render_pass: renderpass,
            attachment_count: framebuffer_attachments.len() as u32,
            p_attachments: framebuffer_attachments.as_ptr(),
            width: surface_size.extent.width,
            height: surface_size.extent.height,
            layers: 1,
        };
        let framebuffer;
        unsafe {
            framebuffer = rs.device
                .create_framebuffer(&frame_buffer_create_info, None)
                .unwrap();
            framebuffers.push(framebuffer);
        }
        framebuffers
    }

    /// Creates commandbuffers for the presentable images, one per image.
    ///
    /// * `rs`            The RenderState.
    /// * `framebuffers`  Framebuffers for the presentable images.
    fn create_commandbuffer(
        rs: &RenderState,
        framebuffers: &Vec<vk::Framebuffer>,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::CommandBufferAllocateInfo,
            p_next: ptr::null(),
            command_buffer_count: framebuffers.len() as u32,
            command_pool: rs.commandpool,
            level: vk::CommandBufferLevel::Primary,
        };
        let command_buffers;
        unsafe {
            command_buffers = rs.device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap();
        }

        command_buffers
    }


    /// Initializes the MainPass based on a RenderState
    ///
    /// This will set up the renderpass, etc.
    ///
    /// * `rs`  The RenderState.
    pub fn init(rs: &RenderState, cfg: &Config) -> MainPass {
        // Surface
        // TODO: Maybe find a way to get this another way. Currently Straight copy from
        // PresentPass
        let surface_loader =
            Surface::new(&rs.entry, &rs.instance).expect("Unable to load the Surface extension");
        let surface = PresentPass::create_surface(&rs.entry, &rs.instance, &rs.window).unwrap();
        assert!(surface_loader.get_physical_device_surface_support_khr(
            rs.pdevice,
            rs.queue_family_index,
            surface,
        ));
        let surface_formats = surface_loader
            .get_physical_device_surface_formats_khr(rs.pdevice, surface)
            .unwrap();
        let surface_format = surface_formats
            .iter()
            .map(|sfmt| match sfmt.format {
                vk::Format::Undefined => vk::SurfaceFormatKHR {
                    format: vk::Format::B8g8r8Unorm,
                    color_space: sfmt.color_space,
                },
                _ => sfmt.clone(),
            })
            .nth(0)
            .expect("Unable to find suitable surface format.");

        let (texture_image, texture_mem, texture_view, texture_sampler) =
            rs.load_image("assets/project_peril_logo.png");

        let render_size = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: cfg.render_dimensions.0,
                height: cfg.render_dimensions.1,
            },
        };

        //Create image to render to.
        let (render_image, render_mem, render_image_view, render_sampler) =
            MainPass::create_renderimages(rs, &surface_format, &render_size);

        let renderpass = MainPass::create_renderpass(rs, &surface_format);
        let (
            descriptor_pool,
            descriptor_set_layouts,
            descriptor_sets,
            pipeline_layout,
            viewport,
            scissor,
            pipeline,
        ) = MainPass::create_pipeline(rs, render_size, renderpass, texture_view, texture_sampler);
        let framebuffers =
            MainPass::create_framebuffer(rs, render_size, render_image_view, renderpass);
        let command_buffers = MainPass::create_commandbuffer(rs, &framebuffers);

        MainPass {
            renderpass: renderpass,
            descriptor_pool: descriptor_pool,
            descriptor_set_layouts: descriptor_set_layouts,
            descriptor_sets: descriptor_sets,
            pipeline_layout: pipeline_layout,
            viewport: viewport,
            scissor: scissor,
            pipeline: pipeline,
            //one framebuffer/commandbuffer per image
            framebuffers: framebuffers,
            commandbuffers: command_buffers,

            //ImageView to render to.
            render_image: render_image,
            render_image_view: render_image_view,
            render_mem: render_mem,
            render_sampler: render_sampler,

            // Keep a pointer to the device for cleanup
            device: Rc::clone(&rs.device),

            //TODO: remove later
            //ImageView to render to.
            texture_image: texture_image,
            texture_image_view: texture_view,
            texture_mem: texture_mem,
            texture_sampler: texture_sampler,
        }
    }
    ///Begin main render pass
    ///
    ///returns a command buffer to be used in rendering.
    pub fn begin_frame(&mut self, rs: &RenderState) -> Option<vk::CommandBuffer> {
        // Begin commandbuffer
        let cmd_buf_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::CommandBufferBeginInfo,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT,
        };
        let cmd_buf = self.commandbuffers[0];
        unsafe {
            rs.device
                .begin_command_buffer(cmd_buf, &cmd_buf_begin_info)
                .expect("Begin commandbuffer");
        }

        // Begin renderpass
        let clear_values = [
            vk::ClearValue::new_color(vk::ClearColorValue::new_float32([0.0, 1.0, 0.0, 1.0])),
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RenderPassBeginInfo,
            p_next: ptr::null(),
            render_pass: self.renderpass,
            framebuffer: self.framebuffers[0],
            render_area: self.scissor,
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            // Start the render pass
            rs.device.cmd_begin_render_pass(
                cmd_buf,
                &render_pass_begin_info,
                vk::SubpassContents::Inline,
            );

            rs.device.cmd_bind_descriptor_sets(
                cmd_buf,
                vk::PipelineBindPoint::Graphics,
                self.pipeline_layout,
                0,
                &self.descriptor_sets[..],
                &[],
            );

            // Bind pipeline
            rs.device
                .cmd_bind_pipeline(cmd_buf, vk::PipelineBindPoint::Graphics, self.pipeline);

            rs.device.cmd_set_viewport(cmd_buf, &[self.viewport]);
            rs.device.cmd_set_scissor(cmd_buf, &[self.scissor]);
        }

        Some(cmd_buf)
    }

    ///End the main render frame and returns an Image.
    pub fn end_frame_and_present(&mut self, rs: &RenderState) {
        let cmd_buf = self.commandbuffers[0];

        unsafe {
            // End render pass and command buffer
            rs.device.cmd_end_render_pass(cmd_buf);
        }

        // Transition the mainpass output to a samplable image
        let image_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::ImageMemoryBarrier,
            p_next: ptr::null(),
            src_access_mask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT
                | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
            dst_access_mask: vk::ACCESS_SHADER_READ_BIT,
            old_layout: vk::ImageLayout::ShaderReadOnlyOptimal,
            new_layout: vk::ImageLayout::ShaderReadOnlyOptimal,
            src_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
            image: self.render_image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::IMAGE_ASPECT_COLOR_BIT,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };
        unsafe {
            rs.device.cmd_pipeline_barrier(
                cmd_buf,
                vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
                vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier],
            );

            rs.device
                .end_command_buffer(cmd_buf)
                .expect("End commandbuffer");
        }

        // Send the work off to the GPU
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SubmitInfo,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 1,
            p_command_buffers: &cmd_buf,
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        };
        unsafe {
            rs.device
                .queue_submit(rs.graphics_queue, &[submit_info], vk::Fence::null())
                .expect("queue submit failed.");
        }
    }
}

impl Drop for MainPass {
    fn drop(&mut self) {
        // We cannot have the last reference to device at this point
        debug_assert!(1 < Rc::strong_count(&self.device));

        unsafe {
            // Always wait for device idle
            self.device.device_wait_idle().unwrap();

            self.device.destroy_sampler(self.render_sampler, None);
            self.device.destroy_image_view(self.render_image_view, None);
            self.device.destroy_image(self.render_image, None);
            self.device.free_memory(self.render_mem, None);

            self.device.destroy_sampler(self.texture_sampler, None);
            self.device
                .destroy_image_view(self.texture_image_view, None);
            self.device.destroy_image(self.texture_image, None);
            self.device.free_memory(self.texture_mem, None);


            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            for &dset_layout in self.descriptor_set_layouts.iter() {
                self.device.destroy_descriptor_set_layout(dset_layout, None);
            }

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.device.destroy_render_pass(self.renderpass, None);
        }
    }
}
