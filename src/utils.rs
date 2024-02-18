use std::thread;
use std::sync::Arc;
use std::task::{Context, Poll};

use winit::window::Window;


/// #### 한국어 </br>
/// 렌더링 시스템을 초기화 합니다. </br>
/// 
/// #### English (Translation) </br>
/// Initialize the rendering system. </br>
/// 
pub fn setup_rendering_system(window: Arc<Window>) -> (
    Arc<wgpu::Instance>, 
    Arc<wgpu::Surface<'static>>, 
    Arc<wgpu::Adapter>, 
    Arc<wgpu::Device>, 
    Arc<wgpu::Queue>, 
) {
    let instance = create_render_instance();
    let surface = create_render_surface(&instance, window.clone());
    let adapter = create_render_adapter(&instance, &surface);
    let (device, queue) = create_render_device_and_queue(&adapter);
    (instance, surface, adapter, device, queue)
}

/// #### 한국어 </br>
/// `wgpu` 렌더링 인스턴스를 생성합니다. </br>
/// 
/// #### English (Translation) </br>
/// Creates a `wgpu` rendering instance. </br>
/// 
#[inline]
fn create_render_instance() -> Arc<wgpu::Instance> {
    let instance_desc = if cfg!(target_os = "windows") {
        wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default(), 
            ..Default::default()
        }
    } else if cfg!(target_os = "linux") {
        wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        }
    } else if cfg!(target_os = "macos") {
        wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..Default::default()
        }
    } else {
        wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY, 
            ..Default::default()
        }
    };

    Arc::new(wgpu::Instance::new(instance_desc))
}

/// #### 한국어 </br>
/// `wgpu` 렌더링 표면을 생성합니다. </br>
/// 
/// #### English (Translation) </br>
/// Creates a `wgpu` rendering surface. </br>
/// 
#[inline]
fn create_render_surface(instance: &wgpu::Instance, window: Arc<Window>) -> Arc<wgpu::Surface<'static>> {
    Arc::new(instance.create_surface(wgpu::SurfaceTarget::from(window)).unwrap())
}

/// #### 한국어 </br>
/// `wgpu` 렌더링 어뎁터를 생성합니다. </br>
/// 
/// #### English (Translation) </br>
/// Creates a `wgpu` rendering adapter. </br>
/// 
#[inline]
fn create_render_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> Arc<wgpu::Adapter> {
    Arc::new(pollster::block_on(
        instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(surface), 
            force_fallback_adapter: false, 
            power_preference: wgpu::PowerPreference::default()
        }) 
    ).unwrap())
}

/// #### 한국어 </br>
/// `wgpu` 렌더링 장치와 명령어 대기열을 생성합니다. </br>
/// 
/// #### English (Translation) </br>
/// Creates a `wgpu` rendering device and command queue. </br>
/// 
#[inline]
fn create_render_device_and_queue(adapter: &wgpu::Adapter) -> (Arc<wgpu::Device>, Arc<wgpu::Queue>) {
    pollster::block_on(
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("DeviceDescriptor"), 
                required_features: wgpu::Features::empty(), 
                required_limits: wgpu::Limits::default()
                    .using_resolution(adapter.limits())
            }, 
            None
        )
    )
    .map(|(device, queue)| (Arc::new(device), Arc::new(queue)))
    .unwrap()
}

/// #### 한국어 </br>
/// 깊이-스텐실 버퍼를 생성합니다. </br>
/// 
/// #### English (Translation) </br>
/// Creates a depth-stencil buffer. </br>
/// 
#[inline]
fn create_depth_stencil_view(window: &Window, device: &wgpu::Device) -> Arc<wgpu::TextureView> {
    device.create_texture(
        &wgpu::TextureDescriptor {
            label: Some("Depth-Stencil Buffer"), 
            size: wgpu::Extent3d {
                width: window.inner_size().width, 
                height: window.inner_size().height, 
                depth_or_array_layers: 1,
            },
            mip_level_count: 1, 
            sample_count: 1, 
            dimension: wgpu::TextureDimension::D2, 
            format: wgpu::TextureFormat::Depth32Float, 
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
            view_formats: &[]
        }
    )
    .create_view(&wgpu::TextureViewDescriptor { ..Default::default() })
    .into()
}
