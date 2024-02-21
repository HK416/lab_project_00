use std::fmt;
use glam::Vec4Swizzles;



/// #### 한국어 </br>
/// 게임 월드 좌표계에 존재하는 오브젝트의 인터페이스 입니다. </br>
/// 
/// #### English (Translation) </br>
/// This is an interface for objects that exists in the game world coordinate system. </br>
/// 
pub trait GameObject : fmt::Debug {
    /// #### 한국어 </br>
    /// 오브젝트의 위치를 가져옵니다. </br>
    /// 
    /// #### Enlglish (Translation) </br>
    /// Gets the position of an object. </br>
    /// 
    #[inline]
    fn get_position(&self) -> glam::Vec3 {
        self.ref_world_transform().w_axis.xyz()
    }

    /// #### 한국어 </br>
    /// 오브젝트의 위치를 설정합니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Set the position of the object. </br>
    /// 
    #[inline]
    fn set_position(&mut self, position: glam::Vec3) {
        self.mut_world_transform().w_axis = (position, 1.0).into();
    }

    /// #### 한국어 </br>
    /// 로컬 좌표계 기준으로 오브젝트를 이동시킵니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Move the object based on the local coordinate system. </br>
    /// 
    fn translate_local(&mut self, distance: glam::Vec3) {
        let right = self.ref_world_transform().x_axis.normalize_or_zero().xyz() * distance.x;
        let up = self.ref_world_transform().y_axis.normalize_or_zero().xyz() * distance.y;
        let look = self.ref_world_transform().z_axis.normalize_or_zero().xyz() * distance.z;
        self.translate_world(right + up + look);
    }

    /// #### 한국어 </br>
    /// 월드 좌표계 기준으로 오브젝트를 이동시킵니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Move the object based on the world coordinate system. </br>
    /// 
    #[inline]
    fn translate_world(&mut self, distance: glam::Vec3) {
        self.set_position(self.get_position() + distance);
    }

    /// #### 한국어 </br>
    /// 오브젝트의 회전을 가져옵니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Gets the rotation of an object. </br>
    /// 
    #[inline]
    fn get_rotation(&self) -> glam::Quat {
        glam::Quat::from_mat4(self.ref_world_transform())
    }

    /// #### 한국어 </br>
    /// 오브젝트의 회전을 설정합니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Sets the rotation of an object. </br>
    /// 
    #[inline]
    fn set_rotation(&mut self, rotation: glam::Quat) {
        let mat = glam::Mat4::from_quat(rotation.normalize());
        self.mut_world_transform().x_axis = mat.x_axis;
        self.mut_world_transform().y_axis = mat.y_axis;
        self.mut_world_transform().z_axis = mat.z_axis;
    }

    /// #### 한국어 </br>
    /// 점을 바라보도록 오브젝트의 회전을 설정합니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Sets the object's rotation to look at a point. </br>
    /// 
    fn look_at_point(&mut self, point: glam::Vec3) {
        let position = self.ref_world_transform().w_axis.xyz();
        let up = self.ref_world_transform().y_axis.xyz();
        let look = (position - point).normalize_or_zero();
        let right = up.cross(look).normalize_or_zero();
        let up = look.cross(right).normalize_or_zero();

        self.mut_world_transform().x_axis = (right, 0.0).into();
        self.mut_world_transform().y_axis = (up, 0.0).into();
        self.mut_world_transform().z_axis = (look, 0.0).into();
    }

    /// #### 한국어 </br>
    /// 오브젝트를 회전시킵니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Rotate the object. </br>
    /// 
    #[inline]
    fn rotate(&mut self, rotation: glam::Quat) {
        let mat = glam::Mat4::from_quat(rotation.normalize());
        *self.mut_world_transform() = mat.mul_mat4(self.ref_world_transform());
    }

    /// #### 한국어 </br>
    /// 오브젝트의 월드 변환 행렬을 빌려옵니다. (reference ver) </br>
    /// 
    /// #### English (Translation) </br>
    /// Borrows the object's world transformation matrix. (reference ver) </br>
    /// 
    fn ref_world_transform(&self) -> &glam::Mat4;

    /// #### 한국어 </br>
    /// 오브젝트의 월드 변환 행렬을 빌려옵니다. (mutable ver) </br>
    /// 
    /// #### English (Translation) </br>
    /// Borrows the object's world transformation matrix. (mutable ver) </br>
    /// 
    fn mut_world_transform(&mut self) -> &mut glam::Mat4;
}

pub trait GameCameraObject : GameObject {
    /// #### 한국어 </br>
    /// 카메라 오브젝트의 카메라 변환 행렬을 가져옵니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Gets the camera transformation matrix of a camera object. </br>
    /// 
    #[inline]
    fn get_camera_transform(&self) -> glam::Mat4 {
        let right = self.ref_world_transform().x_axis.xyz().normalize_or_zero();
        let up = self.ref_world_transform().y_axis.xyz().normalize_or_zero();
        let look = self.ref_world_transform().z_axis.xyz().normalize_or_zero();
        let position = self.ref_world_transform().w_axis.xyz();
        return glam::mat4(
            glam::vec4(right.x, up.x, look.x, 0.0), 
            glam::vec4(right.y, up.y, look.y, 0.0), 
            glam::vec4(right.z, up.z, look.z, 0.0), 
            glam::vec4(-position.dot(right), -position.dot(up), -position.dot(look), 1.0)
        );
    }

    /// #### 한국어 </br>
    /// 카메라 오브젝트의 투영 변환 행렬을 가져옵니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Gets the projection transformation matrix of a camera object. </br>
    /// 
    fn get_projection_transform(&self) -> glam::Mat4;
}


/// #### 한국어 </br>
/// 쉐이더 리소스에 대한 인터페이스 입니다. </br>
/// 
/// #### English (Translation) </br>
/// This is an interface to shader resources. </br>
/// 
pub trait ShaderResource : fmt::Debug {
    /// #### 한국어 </br>
    /// 쉐이더 리소스를 갱신합니다. </br>
    /// 이 함수는 리소스를 바로 갱신하지 않습니다. (참고: [wgpu::Queue]) </br>
    /// 
    /// #### English (Translation) </br>
    /// Updates the shader resource. </br>
    /// This function does not update resource immediately. (see also: [wgpu::Queue]) </br>
    /// 
    fn update_shader_resource(&self, queue: &wgpu::Queue);

    /// #### 한국어 </br>
    /// 바인드 그룹을 빌려옵니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Borrows a bind group. </br>
    /// 
    fn ref_bind_group(&self) -> &wgpu::BindGroup;
}
