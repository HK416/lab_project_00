use std::mem;



/// #### 한국어 </br>
/// 불투명한 색상 오브젝트들을 그리는 그래픽스 파이프라인을 생성합니다. </br>
///
/// #### English (Translation) </br>
/// Create a graphics pipeline to draw opaque colored objects. </br>
/// 
pub fn create_opaque_pipeline(
    device: &wgpu::Device, 
    module: &wgpu::ShaderModule, 
    bind_group_layouts: &[&wgpu::BindGroupLayout]
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("PipelineLayout(ColoredObject(Opaque))"), 
            bind_group_layouts, 
            push_constant_ranges: &[], 
        },
    );

    let pipeline = device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: Some("RenderPipeline(ColoredObject(Opaque))"), 
            layout: Some(&pipeline_layout), 
            vertex: wgpu::VertexState {
                module: &module, 
                entry_point: "vs_main", 
                buffers: &[
                    wgpu::VertexBufferLayout {
                        step_mode: wgpu::VertexStepMode::Vertex, 
                        array_stride: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress, 
                        attributes: &[
                            wgpu::VertexAttribute {
                                shader_location: 0, 
                                format: wgpu::VertexFormat::Float32x3, 
                                offset: 0, 
                            },
                        ],
                    },
                ],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip, 
                strip_index_format: Some(wgpu::IndexFormat::Uint16), 
                polygon_mode: wgpu::PolygonMode::Fill, 
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float, 
                depth_write_enabled: true, 
                depth_compare: wgpu::CompareFunction::Less, 
                stencil: wgpu::StencilState::default(), 
                bias: wgpu::DepthBiasState::default(), 
            }),
            multisample: wgpu::MultisampleState::default(), 
            fragment: Some(wgpu::FragmentState {
                module: &module, 
                entry_point: "fs_opaque_main", 
                targets: &[
                    Some(wgpu::ColorTargetState {
                        blend: None, 
                        format: wgpu::TextureFormat::Bgra8Unorm, 
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            multiview: None,
        },
    );

    return pipeline;
}

/// #### 한국어 </br>
/// 투명한 색상 오브젝트를 그리는 기본 그래픽스 파이프라인을 생성합니다. </br>
/// 
/// #### English (Translation) </br>
/// Create a default graphics pipeline to draw transparent colored object. </br>
/// 
pub fn create_transparent_pipeline(
    device: &wgpu::Device, 
    module: &wgpu::ShaderModule, 
    bind_group_layouts: &[&wgpu::BindGroupLayout]
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("PipelineLayout(ColoredObject(Transparent))"), 
            bind_group_layouts, 
            push_constant_ranges: &[],
        },
    );

    let pipeline = device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: Some("RenderPipeline(ColoredObject(Transparent))"), 
            layout: Some(&pipeline_layout), 
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip, 
                strip_index_format: Some(wgpu::IndexFormat::Uint16), 
                polygon_mode: wgpu::PolygonMode::Fill, 
                ..Default::default()
            }, 
            vertex: wgpu::VertexState { 
                module: &module, 
                entry_point: "vs_main", 
                buffers: &[
                    wgpu::VertexBufferLayout {
                        step_mode: wgpu::VertexStepMode::Vertex, 
                        array_stride: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress, 
                        attributes: &[
                            wgpu::VertexAttribute {
                                shader_location: 0, 
                                format: wgpu::VertexFormat::Float32x3, 
                                offset: 0, 
                            },
                        ],
                    },
                ], 
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float, 
                depth_compare: wgpu::CompareFunction::Less, 
                depth_write_enabled: false, 
                stencil: wgpu::StencilState::default(), 
                bias: wgpu::DepthBiasState::default(), 
            }),
            multisample: wgpu::MultisampleState::default(), 
            fragment: Some(wgpu::FragmentState {
                module: &module, 
                entry_point: "fs_transparent_pass", 
                targets: &[
                    // (한국어) 
                    // 첫 번째 렌더 타겟: (RGB * 가중치, Alpha * 가중치)를 RGBA로 저장하하는 누적 값.
                    // 최소 `Rgba16Float`의 정밀도를 가져야 한다.
                    // 
                    // (English Translation)
                    // First Render Target: Accumulated value (RGB * Weight, Alpha * Weight) stored as RGBA.
                    // It must have a precision of at least `Rgba16Float`.
                    //
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float, 
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One, 
                                dst_factor: wgpu::BlendFactor::One, 
                                operation: wgpu::BlendOperation::Add, 
                            }, 
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One, 
                                dst_factor: wgpu::BlendFactor::One, 
                                operation: wgpu::BlendOperation::Add, 
                            }, 
                        }), 
                        write_mask: wgpu::ColorWrites::ALL, 
                    }),
                    // (한국어)
                    // 두 번째 렌더 타겟: 이전의 색이 얼마만큼 노출이 될 수 있는지에 대한 노출 값.
                    // 최소 `R8`의 정밀도를 가져야 한다.
                    // 
                    // (English Translation)
                    // Second Render Target: Revealage value of how much of the previous color can be exposed. 
                    // It must have a precision of at least `R8`.
                    // 
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R8Unorm, 
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::Zero, 
                                dst_factor: wgpu::BlendFactor::OneMinusSrc, 
                                operation: wgpu::BlendOperation::Add, 
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::Zero, 
                                dst_factor: wgpu::BlendFactor::OneMinusSrc, 
                                operation: wgpu::BlendOperation::Add, 
                            }
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            multiview: None
        },
    );

    return pipeline;
}

/// #### 한국어 </br>
/// 불투명한 색상 오브젝트와 투명한 색상 오브젝트를 합성하는 그래픽스 파이프라인을 생성합니다. </br>
/// 
/// #### English (Translation) </br>
/// Create a graphics pipeline to composite opaque and transparent colored objects. </br>
/// 
pub fn create_composite_pipeline(
    device: &wgpu::Device, 
    module: &wgpu::ShaderModule, 
    bind_group_layouts: &[&wgpu::BindGroupLayout], 
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("PipelineLayout(ColoredObject(Composite))"), 
            bind_group_layouts, 
            push_constant_ranges: &[],
        },
    );

    let pipeline = device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: Some("RenderPipeline(ColoredObject(Composite))"), 
            layout: Some(&pipeline_layout), 
            vertex: wgpu::VertexState {
                module, 
                entry_point: "vs_composite_pass", 
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip, 
                strip_index_format: Some(wgpu::IndexFormat::Uint16), 
                polygon_mode: wgpu::PolygonMode::Fill, 
                ..Default::default()
            }, 
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float, 
                depth_compare: wgpu::CompareFunction::Less, 
                depth_write_enabled: true, 
                stencil: wgpu::StencilState::default(), 
                bias: wgpu::DepthBiasState::default(),
            }), 
            multisample: wgpu::MultisampleState::default(), 
            fragment: Some(wgpu::FragmentState {
                module, 
                entry_point: "fs_composite_pass", 
                targets: &[
                    Some(wgpu::ColorTargetState {
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING), 
                        format: wgpu::TextureFormat::Bgra8Unorm, 
                        write_mask: wgpu::ColorWrites::ALL, 
                    }),
                ],
            }),
            multiview: None,
        },
    );
    
    return pipeline;
}
