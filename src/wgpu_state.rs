use raw_window_handle::{RawDisplayHandle, WaylandDisplayHandle};
use std::ptr::NonNull;
use wayland_client::Connection;

pub struct WgpuState {
    pub instance: wgpu::Instance,
    pub raw_display_handle: RawDisplayHandle,
}

impl WgpuState {
    pub fn new(conn: &Connection) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backend::Vulkan.into(),
            ..Default::default()
        });

        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(conn.backend().display_ptr() as *mut _).unwrap(),
        ));

        Self {
            instance,
            raw_display_handle,
        }
    }
}
