mod output;

use raw_window_handle::{RawDisplayHandle, WaylandDisplayHandle};
use std::ptr::NonNull;
use wayland_client::{
    protocol::{wl_compositor, wl_output, wl_registry, wl_surface},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1;
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer},
    zwlr_layer_surface_v1::Anchor,
};

struct Wgpu {
    output_manager: Option<zxdg_output_manager_v1::ZxdgOutputManagerV1>,
    compositor: Option<wl_compositor::WlCompositor>,
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    exit: bool,
    width: u32,
    height: u32,
    instance: wgpu::Instance,
    raw_display_handle: RawDisplayHandle,
    outputs: Vec<output::Output>,
}

impl Wgpu {
    fn new(conn: &Connection) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(conn.backend().display_ptr() as *mut _).unwrap(),
        ));

        Self {
            compositor: None,
            output_manager: None,
            layer_shell: None,
            exit: false,
            width: 1920,
            height: 50,
            instance,
            raw_display_handle,
            outputs: Vec::new(),
        }
    }
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();

    let qh = event_queue.handle();

    let mut wgpu = Wgpu::new(&conn);

    _ = display.get_registry(&qh, ());

    while !wgpu.exit {
        event_queue.blocking_dispatch(&mut wgpu).unwrap();
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for Wgpu {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(registry.bind::<wl_compositor::WlCompositor, _, _>(
                        name,
                        version,
                        qh,
                        (),
                    ));
                }
                "zxdg_output_manager_v1" => {
                    state.output_manager = Some(
                        registry.bind::<zxdg_output_manager_v1::ZxdgOutputManagerV1, _, _>(
                            name,
                            version,
                            qh,
                            (),
                        ),
                    );
                }
                "zwlr_layer_shell_v1" => {
                    state.layer_shell = Some(
                        registry.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(
                            name,
                            version,
                            qh,
                            (),
                        ),
                    );
                }
                "wl_output" => {
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());

                    let surface = state.compositor.as_ref().unwrap().create_surface(qh, ());

                    let layer_surface = state.layer_shell.as_ref().unwrap().get_layer_surface(
                        &surface,
                        Some(&output),
                        Layer::Top,
                        "status bar".to_string(),
                        qh,
                        (),
                    );
                    layer_surface.set_anchor(Anchor::Top);
                    layer_surface.set_size(state.width, state.height);
                    layer_surface.set_exclusive_zone(state.height as i32);
                    surface.commit();

                    state.outputs.push(output::Output::new(
                        output,
                        surface,
                        layer_surface,
                        &state.instance,
                        state.raw_display_handle,
                    ));
                }
                _ => {}
            },
            wl_registry::Event::GlobalRemove { name: _ } => {}
            _ => unreachable!(),
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for Wgpu {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: <wl_compositor::WlCompositor as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for Wgpu {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _event: <zwlr_layer_shell_v1::ZwlrLayerShellV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}
