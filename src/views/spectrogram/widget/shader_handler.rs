use iced::widget::shader::{self, Pipeline, Viewport};
use iced::wgpu::util::DeviceExt;
use iced::wgpu;
use iced::Rectangle;

use crate::views::spectrogram::widget::{color_maps, SpectrogramView};

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
#[repr(C)]
pub struct Uniforms {
    resolution: [f32; 2],
    min_value: f32,
    max_value: f32,
    x_count: u32,
    y_count: u32,
    color_map_size: u32,
    _pad: u32,
}

pub struct SpectrogramViewPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    x_storage_buffer: wgpu::Buffer,
    y_storage_buffer: wgpu::Buffer,
    val_storage_buffer: wgpu::Buffer,
    color_map_storage_buffer: wgpu::Buffer,
    buffer_sizes: [usize; 4],
    uniform_bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout
}

#[derive(Debug)]
pub struct SpectrogramViewPrimitive {
    spectrogram: SpectrogramView,
}

impl SpectrogramViewPrimitive {
    pub fn new(spectrogram: SpectrogramView) -> Self {
        Self {
            spectrogram
        }
    }
}

impl shader::Primitive for SpectrogramViewPrimitive {
    type Pipeline = SpectrogramViewPipeline;

    fn prepare(&self, pipeline: &mut Self::Pipeline, device: &wgpu::Device, queue: &wgpu::Queue, bounds: &Rectangle, _viewport: &Viewport) {
        let colormap = color_maps::get_color_map(&self.spectrogram.color_map);
        pipeline.update(
            device,
            queue,
            &Uniforms {
                resolution: [bounds.width as f32, bounds.height as f32],
                x_count: self.spectrogram.time.len() as u32,
                y_count: self.spectrogram.freqs.len() as u32,
                min_value: self.spectrogram.vmin as f32,
                max_value: self.spectrogram.vmax as f32,
                color_map_size: colormap.len() as u32,
                ..Default::default()
            },
            &self.spectrogram.time,
            &self.spectrogram.freqs,
            &self.spectrogram.result,
            colormap
        );
    }

    fn render(&self, pipeline: &Self::Pipeline, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, viewport: &Rectangle<u32>) {
        pipeline.render(target, encoder, viewport);
    }
}

impl Pipeline for SpectrogramViewPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self where Self: Sized {
        SpectrogramViewPipeline::new(device, format)
    }

    fn trim(&mut self) { }
}

impl SpectrogramViewPipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("FragmentShaderPipeline shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "spectrogram.wgsl"
            ))),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                resolution: [0.0, 0.0],
                x_count: 0,
                y_count: 0,
                min_value: 0.0,
                max_value: 0.0,
                color_map_size: 0,
                ..Default::default()
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create storage buffer
        let x_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("X Storage Buffer"),
            contents: bytemuck::cast_slice(&vec![0.0; 1]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let y_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Y Storage Buffer"),
            contents: bytemuck::cast_slice(&vec![0.0; 1]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let val_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Val Storage Buffer"),
            contents: bytemuck::cast_slice(&vec![0.0; 1]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let color_map_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Color map Storage Buffer"),
            contents: bytemuck::cast_slice(&vec![0.0; 1]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("bind_group_layout"),
        });

        // Create bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: x_storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: y_storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: val_storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: color_map_storage_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });

        // Create pipeline layout
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("FragmentShaderPipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None
        });

        Self {
            pipeline,
            uniform_buffer,
            x_storage_buffer,
            y_storage_buffer,
            val_storage_buffer,
            color_map_storage_buffer,
            uniform_bind_group,
            bind_group_layout,
            buffer_sizes: [0, 0, 0, 0]
        }
    }

    fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uniforms: &Uniforms,
        time: &Vec<f32>,
        freqs: &Vec<f32>,
        data: &Vec<f32>,
        color_map: Vec<[f32; 4]>
    ) {
        let min_time_buffer_len = size_of::<f32>() * time.len();
        let min_freqs_buffer_len = size_of::<f32>() * freqs.len();
        let min_data_buffer_len = size_of::<f32>() * data.len();
        let min_color_map_buffer_len = size_of::<f32>() * color_map.len();

        let resize_time = self.buffer_sizes[0] < min_time_buffer_len;
        let resize_freqs = self.buffer_sizes[1] < min_freqs_buffer_len;
        let resize_data = self.buffer_sizes[2] < min_data_buffer_len;
        let resize_color_map = self.buffer_sizes[3] < min_color_map_buffer_len;

        // In case any of the buffer sizes has to be extended, create the new buffer and update the bind group
        if resize_time || resize_freqs || resize_data || resize_color_map {
            // Resize time buffer
            if resize_time {
                self.buffer_sizes[0] = min_time_buffer_len;
                self.x_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("X Storage Buffer"),
                    contents: bytemuck::cast_slice(&vec![0.0; min_time_buffer_len]),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                });
            }

            // Resize freqs buffer
            if resize_freqs {
                self.buffer_sizes[1] = min_freqs_buffer_len;
                self.y_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Y Storage Buffer"),
                    contents: bytemuck::cast_slice(&vec![0.0; min_freqs_buffer_len]),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                });
            }

            // Resize data buffer
            if resize_data {
                self.buffer_sizes[2] = min_data_buffer_len;
                self.val_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Val Storage Buffer"),
                    contents: bytemuck::cast_slice(&vec![0.0; min_data_buffer_len]),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                });
            }

            // Resize color map buffer
            if resize_color_map {
                self.buffer_sizes[3] = min_color_map_buffer_len;
                self.color_map_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Color map Storage Buffer"),
                    contents: bytemuck::cast_slice(&vec![0.0; min_color_map_buffer_len]),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                });
            }

            // Create new bind group with updated buffer sizes
            self.uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Uniform bind group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding()
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.x_storage_buffer.as_entire_binding()
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.y_storage_buffer.as_entire_binding()
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.val_storage_buffer.as_entire_binding()
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.color_map_storage_buffer.as_entire_binding()
                    }
                ]
            });
        }

        // Update buffer values
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
        queue.write_buffer(&self.x_storage_buffer, 0, bytemuck::cast_slice(&time));
        queue.write_buffer(&self.y_storage_buffer, 0, bytemuck::cast_slice(&freqs));
        queue.write_buffer(&self.val_storage_buffer, 0, bytemuck::cast_slice(&data));
        queue.write_buffer(&self.color_map_storage_buffer, 0, bytemuck::cast_slice(&color_map));
    }

    fn render(&self, target: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, viewport: &Rectangle<u32>) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_viewport(
            viewport.x as f32,
            viewport.y as f32,
            viewport.width as f32,
            viewport.height as f32,
            0.0,
            1.0,
        );
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.draw(0..4, 0..1);
    }
}
