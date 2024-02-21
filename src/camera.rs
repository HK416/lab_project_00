use std::mem;
use crate::interfaces::{
    GameObject, 
    GameCameraObject, 
    ShaderResource, 
};



/// #### 한국어 </br>
/// 쉐이더에 전달되는 카메라 유니폼 데이터 레이아웃 입니다. </br>
/// 
/// #### English (Translation) </br>
/// This is the camera uniform data layout passed to the shader. </br>
/// 
#[repr(C, align(16))]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CameraUniformLayout {
    pub camera_matrix: glam::Mat4, 
    pub projection_matrix: glam::Mat4, 
}

/// #### 한국어 </br>
/// 원근 투영 카메라를 생성하는 빌더입니다. </br>
/// 
/// #### English (Translation) </br>
/// A builder that creates a perspective projection camera. </br>
/// 
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveCameraBuilder {
    pub translation: glam::Vec3, 
    pub rotation: glam::Quat,  
    pub fov_y_radians: f32, 
    pub aspect_ratio: f32, 
    pub z_near: f32, 
    pub z_far: f32, 
}

#[allow(dead_code)]
impl PerspectiveCameraBuilder {
    #[inline]
    pub fn new(fov_y_radians: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        Self { 
            translation: glam::Vec3::ZERO, 
            rotation: glam::Quat::IDENTITY, 
            fov_y_radians, 
            aspect_ratio, 
            z_near, 
            z_far 
        }
    }

    #[inline]
    pub fn set_translation(mut self, translation: glam::Vec3) -> Self {
        self.translation = translation;
        self
    }

    pub fn translate_local(self, distance: glam::Vec3) -> Self {
        let mat = glam::Mat3::from_quat(self.rotation.normalize());
        let right = mat.x_axis.normalize_or_zero() * distance.x;
        let up = mat.y_axis.normalize_or_zero() * distance.y;
        let look = mat.z_axis.normalize_or_zero() * distance.z;
        self.translate_world(right + up + look)
    }

    #[inline]
    pub fn translate_world(mut self, distance: glam::Vec3) -> Self {
        self.translation += distance;
        self
    }

    #[inline]
    pub fn set_rotation(mut self, rotation: glam::Quat) -> Self {
        self.rotation = rotation.normalize();
        self
    }
    
    pub fn look_at_point(mut self, point: glam::Vec3) -> Self {
        let mat = glam::Mat3::from_quat(self.rotation.normalize());
        let up = mat.y_axis.normalize_or_zero();
        let look = (self.translation - point).normalize_or_zero();
        let right = up.cross(look);
        let up = look.cross(right);
        self.rotation = glam::Quat::from_mat3(&glam::Mat3::from_cols(right, up, look)).normalize();
        self
    }

    #[inline]
    pub fn rotate(mut self, rotation: glam::Quat) -> Self {
        self.rotation *= rotation.normalize();
        self
    }

    pub fn build(self, device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> PerspectiveCamera {
        let buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("UniformBuffer(PerspectiveCamera)"), 
                mapped_at_creation: false, 
                size: mem::size_of::<CameraUniformLayout>() as wgpu::BufferAddress, 
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, 
            },
        );

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("BindGroup(PerspectiveCamera)"), 
                layout: &bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0, 
                        resource: wgpu::BindingResource::Buffer(
                            buffer.as_entire_buffer_binding()
                        ),
                    },
                ],
            },
        );

        PerspectiveCamera {
            fov_y_radians: self.fov_y_radians, 
            aspect_ratio: self.aspect_ratio, 
            z_near: self.z_near, 
            z_far: self.z_far, 
            transform: glam::Mat4::from_rotation_translation(
                self.rotation.normalize(), 
                self.translation
            ), 
            buffer, 
            bind_group, 
        }
    }
}

/// #### 한국어 </br>
/// 게임 월드 좌표계에 존재하는 카메라 입니다. </br>
/// 
/// #### English (Translation) </br>
/// This is a camera that exists in the game world coordinate system. </br>
/// 
#[derive(Debug)]
pub struct PerspectiveCamera {
    fov_y_radians: f32, 
    aspect_ratio: f32, 
    z_near: f32, 
    z_far: f32, 
    transform: glam::Mat4, 
    buffer: wgpu::Buffer, 
    bind_group: wgpu::BindGroup, 
}

impl GameObject for PerspectiveCamera {
    #[inline]
    fn ref_world_transform(&self) -> &glam::Mat4 {
        &self.transform
    }

    #[inline]
    fn mut_world_transform(&mut self) -> &mut glam::Mat4 {
        &mut self.transform
    }
}

impl GameCameraObject for PerspectiveCamera {
    #[inline]
    fn get_projection_transform(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fov_y_radians, self.aspect_ratio, self.z_near, self.z_far)
    }
}

impl ShaderResource for PerspectiveCamera {
    fn update_shader_resource(&self, queue: &wgpu::Queue) {
        let data = CameraUniformLayout {
            camera_matrix: self.get_camera_transform(), 
            projection_matrix: self.get_projection_transform(), 
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&data));
    }

    #[inline]
    fn ref_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
