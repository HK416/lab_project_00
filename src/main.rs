mod camera;
mod interfaces;
mod objects;
mod pipeline;
mod timer;
mod utils;

use std::mem;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering as MemOrdering};
use crossbeam_queue::SegQueue;
use winit::{
    keyboard::{KeyCode, PhysicalKey},
    event::{Event, WindowEvent}, 
    window::{Window, WindowBuilder},
    event_loop::{EventLoop, ControlFlow},
};
use crate::interfaces::{
    GameObject, 
    ShaderResource, 
};

/// #### 한국어 </br>
/// 현재 애플리케이션이 실행 중인 경우 `true`값을 가집니다. </br>
/// 
/// #### English (Translation) </br>
/// Has the value `true` if the application is currently running. </br>
/// 
static IS_RUNNING: AtomicBool = AtomicBool::new(true);

/// #### 한국어 </br>
/// 렌더링 루프로 보내는 창 이벤트 대기열 입니다. </br>
/// 
/// #### English (Translation) </br>
/// This is the window event queue that is sent to the rendering loop. </br>
/// 
static EVENT_QUEUE: SegQueue<Event<()>> = SegQueue::new();



fn render_loop(
    window: Arc<Window>, 
    instance: Arc<wgpu::Instance>, 
    surface: Arc<wgpu::Surface>, 
    _adapter: Arc<wgpu::Adapter>, 
    device: Arc<wgpu::Device>, 
    queue: Arc<wgpu::Queue>
) {
    // (한국어) 카메라의 쉐이더 레이아웃을 생성합니다. 
    // (English Translation) Create a shader layout for the camera. 
    let camera_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("BindGroupLayout(Camera)"), 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, 
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, 
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    }, 
                    count: None,
                },
            ],
        },
    );

    // (한국어) 카메라를 생성합니다. 
    // (English Translation) Create a camera. 
    let mut camera = camera::PerspectiveCameraBuilder::new(
        60.0f32.to_radians(), 
        window.inner_size().width as f32 / window.inner_size().height as f32, 
        0.001, 
        1000.0
    )
    .set_translation((0.0, 3.0, 15.0).into())
    .look_at_point((0.0, 0.0, 0.0).into())
    .build(&device, &camera_bind_group_layout);
    camera.update_shader_resource(&queue);

    // (한국어) 사각형 메쉬를 생성합니다.
    // (English Translation) Creates a quad mesh.
    const MESH_DATA: [[f32; 3]; 4] = [[-1.0, -1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0]];
    let quad_mesh_strip = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("VertexBuffer(QuadMesh)"), 
            mapped_at_creation: false, 
            size: mem::size_of::<[[f32; 3]; 4]>() as wgpu::BufferAddress, 
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, 
        }
    );
    queue.write_buffer(&quad_mesh_strip, 0, bytemuck::cast_slice(&MESH_DATA));

    // (한국어) 색상 오브젝트의 쉐이더 레이아웃을 생성합니다.
    // (English Translation) Create a shader layout for the colored object. 
    let object_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("BindGroupLayout(ColoredObject)"), 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, 
                    visibility: wgpu::ShaderStages::VERTEX, 
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    }, 
                    count: None,
                },
            ],
        },
    );

    // (한국어) 색상 오브젝트들을 생성합니다. 
    // (English Translation) Create color objects. 
    let mut opaque_objects = Vec::new();
    let mut transparent_objects = Vec::new();
    let gray_plain = objects::ColordObjectBuilder::new()
        .set_color((0.5, 0.5, 0.5, 1.0).into())
        .set_scale((8.0, 8.0, 1.0).into())
        .set_translation((0.0, 0.0, 0.0).into())
        .look_at_point((0.0, 1.0, 0.0).into())
        .build(&device, &object_bind_group_layout);
    gray_plain.update_shader_resource(&queue);
    opaque_objects.push(gray_plain);

    let wall = objects::ColordObjectBuilder::new()
        .set_color((0.7, 0.7, 0.7, 1.0).into())
        .set_scale((1.0, 1.0, 1.0).into())
        .set_translation((3.0, 1.0, 0.0).into())
        .build(&device, &object_bind_group_layout);
    wall.update_shader_resource(&queue);
    opaque_objects.push(wall);

    let red_glass = objects::ColordObjectBuilder::new()
        .set_color((1.0, 0.0, 0.0, 0.3).into())
        .set_scale((1.0, 1.0, 1.0).into())
        .set_translation((0.0, 1.0, 0.0).into())
        .build(&device, &object_bind_group_layout);
    red_glass.update_shader_resource(&queue);
    transparent_objects.push(red_glass);

    let wall = objects::ColordObjectBuilder::new()
        .set_color((0.7, 0.7, 0.7, 1.0).into())
        .set_scale((1.0, 1.0, 1.0).into())
        .set_translation((-2.0, 1.0, 5.0).into())
        .build(&device, &object_bind_group_layout);
    wall.update_shader_resource(&queue);
    opaque_objects.push(wall);

    let green_glass = objects::ColordObjectBuilder::new()
        .set_color((0.0, 1.0, 0.0, 0.3).into())
        .set_scale((1.0, 1.0, 1.0).into())
        .set_translation((1.0, 1.0, 3.0).into())
        .build(&device, &object_bind_group_layout);
    green_glass.update_shader_resource(&queue);
    transparent_objects.push(green_glass);

    let blue_glass = objects::ColordObjectBuilder::new()
        .set_color((0.0, 0.0, 1.0, 0.3).into())
        .set_scale((1.0, 1.0, 1.0).into())
        .set_translation((-2.0, 1.0, -5.0).into())
        .build(&device, &object_bind_group_layout);
    blue_glass.update_shader_resource(&queue); 
    transparent_objects.push(blue_glass);

    let wall = objects::ColordObjectBuilder::new()
        .set_color((0.7, 0.7, 0.7, 1.0).into())
        .set_scale((1.0, 1.0, 1.0).into())
        .set_translation((-0.5, 1.0, -2.5).into())
        .build(&device, &object_bind_group_layout);
    wall.update_shader_resource(&queue);
    opaque_objects.push(wall);


    // (한국어) 누적 값을 저장할 텍스처 뷰를 생성합니다.
    // (English Translation) Create a texture view to store accumulated values.
    let mut accum_texture_view = device.create_texture(
        &wgpu::TextureDescriptor {
            label: Some("Accumulate"), 
            size: wgpu::Extent3d {
                width: window.inner_size().width, 
                height: window.inner_size().height, 
                depth_or_array_layers: 1, 
            }, 
            format: wgpu::TextureFormat::Rgba16Float, 
            dimension: wgpu::TextureDimension::D2, 
            mip_level_count: 1, 
            sample_count: 1, 
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
            view_formats: &[], 
        },
    )
    .create_view(&wgpu::TextureViewDescriptor {
        ..Default::default()
    });

    // (한국어) 노출 값을 저장할 텍스처 뷰를 생성합니다.
    // (English Translation) Create a texture view to store revealage values. 
    let mut reveal_texture_view = device.create_texture(
        &wgpu::TextureDescriptor {
            label: Some("Revealage"), 
            size: wgpu::Extent3d {
                width: window.inner_size().width, 
                height: window.inner_size().height, 
                depth_or_array_layers: 1, 
            }, 
            format: wgpu::TextureFormat::R8Unorm, 
            dimension: wgpu::TextureDimension::D2, 
            mip_level_count: 1, 
            sample_count: 1, 
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
            view_formats: &[],
        },
    )
    .create_view(&wgpu::TextureViewDescriptor {
        ..Default::default()
    });

    // (한국어) 누적 값과 노출 값의 바인드 그룹을 생성합니다. 
    // (English Translation) Creates a bind group of accumulated and revealage values. 
    let oit_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("BindGroupLayout(WeightedBlendedOIT)"), 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, 
                    visibility: wgpu::ShaderStages::FRAGMENT, 
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, 
                    visibility: wgpu::ShaderStages::FRAGMENT, 
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    }, 
                    count: None, 
                },
            ],
        },
    );
    let mut oit_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: Some("BindGroup(WeightedBlendedOIT)"), 
            layout: &oit_bind_group_layout, 
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0, 
                    resource: wgpu::BindingResource::TextureView(&accum_texture_view), 
                }, 
                wgpu::BindGroupEntry {
                    binding: 1, 
                    resource: wgpu::BindingResource::TextureView(&reveal_texture_view), 
                },
            ],
        },
    );

    // (한국어) 색상 오브젝트를 그리는 그래픽스 파이프라인을 생성합니다.
    // (English Translation) Create a graphics pipeline to draw colored object. 
    let module = device.create_shader_module(
        wgpu::include_wgsl!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/shader.wgsl"))
    );
    let bind_group_layouts = [&camera_bind_group_layout, &object_bind_group_layout];
    let opaque_pipeline = pipeline::create_opaque_pipeline(&device, &module, &bind_group_layouts);
    let transparent_pipeline = pipeline::create_transparent_pipeline(&device, &module, &bind_group_layouts);

    let bind_group_layouts = [&oit_bind_group_layout];
    let composite_pipeline = pipeline::create_composite_pipeline(&device, &module, &bind_group_layouts);
    

    // (한국어) 스왑체인 및 프레임 버퍼를 설정합니다.
    // (English Translation) Sets the swapchain and frame buffer. 
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT, 
        format: wgpu::TextureFormat::Bgra8Unorm, 
        width: window.inner_size().width, 
        height: window.inner_size().height, 
        present_mode: wgpu::PresentMode::AutoVsync, 
        desired_maximum_frame_latency: 2, 
        alpha_mode: wgpu::CompositeAlphaMode::Auto, 
        view_formats: vec![], 
    };
    surface.configure(&device, &config);

    // (한국어) 깊이-스텐실 텍스처 뷰를 생성합니다.
    // (English Translation) Create the depth-stencil texture view.
    let mut depth_stencil_view = device.create_texture(
        &wgpu::TextureDescriptor {
            label: Some("DepthStencilBuffer"), 
            size: wgpu::Extent3d {
                width: window.inner_size().width, 
                height: window.inner_size().height, 
                depth_or_array_layers: 1, 
            },
            format: wgpu::TextureFormat::Depth32Float, 
            dimension: wgpu::TextureDimension::D2, 
            mip_level_count: 1, 
            sample_count: 1, 
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
            view_formats: &[],
        },
    )
    .create_view(&wgpu::TextureViewDescriptor { 
        ..Default::default()
    });

    // (한국어) 렌더링 루프를 실행합니다.
    // (English Translation) Run the rendering loop.
    log::info!("Run Rendering loop.");
    let mut timer = timer::GameTimer::<50>::new();
    while IS_RUNNING.load(MemOrdering::Acquire) {
        // (한국어) 타이머를 갱신합니다.
        // (English Translation) Updates the timer. 
        timer.tick();

        // (한국어) 창 이벤트를 처리합니다.
        // (English Translation) Handles window events. 
        while let Some(event) = EVENT_QUEUE.pop() {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        if size.width > 0 && size.height > 0 {
                            // (한국어) 모든 작업이 끝날 때 까지 기다립니다.
                            // (English Translation) Wait until all operations are completed.
                            instance.poll_all(true);

                            // (한국어) 스왑체인 및 프레임 버퍼를 재설정합니다.
                            // (English Translation) Reset swapchain and frame buffer. 
                            config.width = size.width;
                            config.height = size.height;
                            surface.configure(&device, &config);

                            // (한국어) 누적 값을 저장할 텍스처 뷰를 재생성합니다.
                            // (English Translation) Recreate a texture view to store accumulated values.
                            accum_texture_view = device.create_texture(
                                &wgpu::TextureDescriptor {
                                    label: Some("Accumulate"), 
                                    size: wgpu::Extent3d {
                                        width: size.width, 
                                        height: size.height, 
                                        depth_or_array_layers: 1, 
                                    }, 
                                    format: wgpu::TextureFormat::Rgba16Float, 
                                    dimension: wgpu::TextureDimension::D2, 
                                    mip_level_count: 1, 
                                    sample_count: 1, 
                                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
                                    view_formats: &[], 
                                },
                            )
                            .create_view(&wgpu::TextureViewDescriptor {
                                ..Default::default()
                            });
                        
                            // (한국어) 노출 값을 저장할 텍스처 뷰를 재생성합니다.
                            // (English Translation) Recreate a texture view to store revealage values. 
                            reveal_texture_view = device.create_texture(
                                &wgpu::TextureDescriptor {
                                    label: Some("Revealage"), 
                                    size: wgpu::Extent3d {
                                        width: size.width, 
                                        height: size.height, 
                                        depth_or_array_layers: 1, 
                                    }, 
                                    format: wgpu::TextureFormat::R8Unorm, 
                                    dimension: wgpu::TextureDimension::D2, 
                                    mip_level_count: 1, 
                                    sample_count: 1, 
                                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
                                    view_formats: &[],
                                },
                            )
                            .create_view(&wgpu::TextureViewDescriptor {
                                ..Default::default()
                            });

                            oit_bind_group = device.create_bind_group(
                                &wgpu::BindGroupDescriptor {
                                    label: Some("BindGroup(WeightedBlendedOIT)"), 
                                    layout: &oit_bind_group_layout, 
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0, 
                                            resource: wgpu::BindingResource::TextureView(&accum_texture_view), 
                                        }, 
                                        wgpu::BindGroupEntry {
                                            binding: 1, 
                                            resource: wgpu::BindingResource::TextureView(&reveal_texture_view), 
                                        },
                                    ],
                                },
                            );

                            // (한국어) 깊이-스텐실 텍스처 뷰를 재생성합니다.
                            // (English Translation) Recreate the depth-stencil texture view. 
                            depth_stencil_view = device.create_texture(
                                &wgpu::TextureDescriptor {
                                    label: Some("DepthStencilBuffer"), 
                                    size: wgpu::Extent3d {
                                        width: size.width, 
                                        height: size.height, 
                                        depth_or_array_layers: 1, 
                                    },
                                    format: wgpu::TextureFormat::Depth32Float, 
                                    dimension: wgpu::TextureDimension::D2, 
                                    mip_level_count: 1, 
                                    sample_count: 1, 
                                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
                                    view_formats: &[],
                                },
                            )
                            .create_view(&wgpu::TextureViewDescriptor { 
                                ..Default::default()
                            });
                        }
                    },
                    WindowEvent::KeyboardInput { event, .. } => {
                        if let PhysicalKey::Code(code) = event.physical_key {
                            if KeyCode::ArrowLeft == code && event.state.is_pressed() {
                                camera.rotate(glam::Quat::from_rotation_y(-180.0f32.to_radians() * timer.elapsed_time_sec()));
                                camera.update_shader_resource(&queue);
                            } else if KeyCode::ArrowRight == code && event.state.is_pressed() {
                                camera.rotate(glam::Quat::from_rotation_y(180.0f32.to_radians() * timer.elapsed_time_sec()));
                                camera.update_shader_resource(&queue);
                            }
                        }
                    },
                    _ => { /*--- empty ---*/ }
                },
                _ => { /*--- empty ---*/ }
            }
        }

        // (한국어) 오브젝트들을 그립니다.
        // (English Translation) Draws the objects.
        window.pre_present_notify();
        
        // (한국어) 이전 작업이 끝날 때 까지 기다립니다.
        // (English Translation) Wait until the previous operation is finished.
        device.poll(wgpu::Maintain::Wait);

        // (한국어) 다음 프레임을 가져옵니다.
        // (English Translation) Get the next frame.
        let frame = surface.get_current_texture().unwrap();

        // (한국어) 렌더 타겟의 텍스처 뷰를 생성합니다.
        // (English Translation) Creates a texture view of render target.
        let render_target_view = frame.texture.create_view(&wgpu::TextureViewDescriptor { 
            ..Default::default()
        });

        // (한국어) 커맨드 버퍼를 생성합니다.
        // (English Translation) Creates a command buffer. 
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            // <1>
            // (한국어) 
            // 불투명한 색상 오브젝트들을 그립니다.
            // 
            // 이때, 깊이 버퍼를 이용하여 오브젝트들의 깊이 값을 저장합니다.
            // 
            // (English Translation) 
            // Draws opaque colored objects. 
            //
            // At this time, the depth value of the objects is stored using the depth buffer. 
            // 
            let mut rpass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("RenderPass(Opaque)"), 
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &render_target_view, 
                            resolve_target: None, 
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), 
                                store: wgpu::StoreOp::Store, 
                            }, 
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_stencil_view, 
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0), 
                            store: wgpu::StoreOp::Store, 
                        }), 
                        stencil_ops: None, 
                    }), 
                    timestamp_writes: None, 
                    occlusion_query_set: None, 
                },
            );

            rpass.set_pipeline(&opaque_pipeline);
            rpass.set_bind_group(0, camera.ref_bind_group(), &[]);
            rpass.set_vertex_buffer(0, quad_mesh_strip.slice(..));
            for object in opaque_objects.iter() {
                rpass.set_bind_group(1, object.ref_bind_group(), &[]);
                rpass.draw(0..4, 0..1);
            }
        }

        {
            // <2>
            // (한국어) 
            // 투명한 색상의 오브젝트들을 그립니다.
            // 
            // 누적 값을 저장하는 버퍼는 0으로, 노출 값을 저장하는 버퍼는 1로 초기화 합니다.
            //
            // 깊이 버퍼를 읽어서 투명한 오브젝트가 가려지는지 확인하고, 가려지는 투명한 오브젝트는 그리지 않습니다.
            // 
            // (English Translation) 
            // Draws transparent colored objects. 
            // 
            // The buffer that stores the accumulate value is initialized to 0, 
            // and the buffer that stores the revealage value is initialized to 1.
            // 
            // Reads the depth buffer to determine whether transparent objects are occluded, 
            // and does not draw transparent objects that are occluded.
            // 
            let mut rpass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("RenderPass(Transparent)"), 
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &accum_texture_view, 
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { 
                                    r: 0.0, 
                                    g: 0.0, 
                                    b: 0.0, 
                                    a: 0.0, 
                                }), 
                                store: wgpu::StoreOp::Store,
                            },
                            resolve_target: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: &reveal_texture_view, 
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 1.0, 
                                    g: 1.0, 
                                    b: 1.0, 
                                    a: 1.0,
                                }), 
                                store: wgpu::StoreOp::Store, 
                            },
                            resolve_target: None,
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load, 
                            store: wgpu::StoreOp::Store, 
                        }),
                        view: &depth_stencil_view, 
                        stencil_ops: None,
                    }), 
                    timestamp_writes: None,
                    occlusion_query_set: None, 
                }
            );

            rpass.set_pipeline(&transparent_pipeline);
            rpass.set_bind_group(0, camera.ref_bind_group(), &[]);
            rpass.set_vertex_buffer(0, quad_mesh_strip.slice(..));
            for object in transparent_objects.iter() {
                rpass.set_bind_group(1, object.ref_bind_group(), &[]);
                rpass.draw(0..4, 0..1);
            }
        }

        {
            // <3>
            // (한국어) 불투명한 색상의 오브젝트와 투명한 색상의 오브젝트를 합성합니다. 
            // (English Translation) Combines opaque colored objects with transparent colored objects. 
            let mut rpass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("RenderPass(Composite)"), 
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &render_target_view, 
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load, 
                                store: wgpu::StoreOp::Store,
                            },
                            resolve_target: None,
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_stencil_view, 
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load, 
                            store: wgpu::StoreOp::Store, 
                        }), 
                        stencil_ops: None, 
                    }), 
                    timestamp_writes: None,
                    occlusion_query_set: None, 
                }
            );

            rpass.set_pipeline(&composite_pipeline);
            rpass.set_bind_group(0, &oit_bind_group, &[]);
            rpass.set_vertex_buffer(0, quad_mesh_strip.slice(..));
            rpass.draw(0..4, 0..1);
        }

        // (한국어) 명령 대기열에 커맨드 버퍼를 제출하고, 프레임 버퍼를 출력합니다.
        // (English Translation) Submit command buffer to the queue and output to the framebuffer. 
        queue.submit(Some(encoder.finish()));
        frame.present();
    }

    log::info!("Finish Rendering loop.");
}

fn main() {
    env_logger::init();
    log::info!("❖ Application Launching ❖");
    
    // (한국어) 창 시스템을 초기화 합니다.
    // (English Translation) Initializes the window system.
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_visible(true)
            .with_resizable(true)
            .with_title("Lab Project 00")
            .build(&event_loop)
            .unwrap()
    );

    // (한국어) 렌더링 시스템을 초기화 합니다.
    // (English Translation) Initialize the rendering system.
    let window_cloned = window.clone();
    let (instance, surface, adapter, device, queue) = utils::setup_rendering_system(window_cloned);

    // (한국어) 새로운 스레드에서 렌더링 루프를 실행합니다.
    // (English Translation) Runs the rendering loop in a new thread.
    let window_cloned = window.clone();
    let instance_cloned = instance.clone();
    let mut join = Some(thread::spawn(move || render_loop(
        window_cloned, 
        instance_cloned, 
        surface, 
        adapter, 
        device, 
        queue
    )));

    // (한국어) 윈도우 메시지 루프를 실행합니다.
    // (English Translation) Runs the window message loop.
    log::info!("Run Window message loop.");
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run(move |event, elwt| {
        // (한국어) 현재 렌더링 스레드가 실행 중인지 확인합니다.
        // (English Translation) Checks if the current rendering thread is running.
        if join.as_ref().is_some_and(|join| join.is_finished()) {
            // (한국어) 렌더링 스레드를 join 합니다.
            // (English Translation) Join the rendering thread.
            join.take().unwrap().join().unwrap();

            // (한국어) 애플리케이션을 종료합니다.
            // (English Translation) Quit the application.
            elwt.exit();
            return;
        }

        // (한국어) 윈도우 이벤트를 처리합니다.
        // (English Translation) Handles window events. 
        let event_cloned = event.clone();
        match event_cloned {
            Event::NewEvents(_) | Event::AboutToWait => {
                return;
            },
            Event::WindowEvent { window_id, event } 
            if window_id == window.id() => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    IS_RUNNING.store(false, MemOrdering::Release);
                    elwt.exit();
                    return;
                },
                _ => { /* empty */ }
            },
            _ => { /* empty */ }
        }

        // (한국어) 창 이벤트를 이벤트 대기열에 추가합니다.
        // (English Translation) Add a window event to the event queue. 
        EVENT_QUEUE.push(event);
    }).unwrap();

    instance.poll_all(true);
    log::info!("❖ Application Terminate ❖");
}
