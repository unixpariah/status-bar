pub mod buffers;
pub mod math;
mod output;
mod rectangle;
mod seat;
mod wgpu_state;

use wayland_client::{
    delegate_noop,
    protocol::{wl_compositor, wl_output, wl_registry, wl_seat},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::{self, Layer};
use wgpu_state::WgpuState;

struct StatusBar {
    output_manager: Option<zxdg_output_manager_v1::ZxdgOutputManagerV1>,
    compositor: Option<wl_compositor::WlCompositor>,
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    seat: Option<seat::Seat>,
    outputs: Vec<output::Output>,
    wgpu: wgpu_state::WgpuState,
    exit: bool,
}

impl StatusBar {
    fn new(conn: &Connection) -> Self {
        Self {
            seat: None,
            compositor: None,
            output_manager: None,
            layer_shell: None,
            outputs: Vec::new(),
            wgpu: WgpuState::new(conn),
            exit: false,
        }
    }

    fn render(&self) {
        self.outputs.iter().for_each(|output| output.render());
    }
}

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().expect("Connection to wayland failed");
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let mut status_bar = StatusBar::new(&conn);

    _ = display.get_registry(&qh, ());
    event_queue.dispatch_pending(&mut status_bar).unwrap();
    event_queue.roundtrip(&mut status_bar).unwrap();

    while !status_bar.exit {
        event_queue.dispatch_pending(&mut status_bar).unwrap();
        status_bar.render();
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for StatusBar {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
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
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ());

                    state.seat = Some(seat::Seat {
                        seat,
                        pointer: None,
                    });
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

                    let xdg_output =
                        state
                            .output_manager
                            .as_ref()
                            .unwrap()
                            .get_xdg_output(&output, qh, ());

                    state.outputs.push(output::Output::new(
                        output,
                        xdg_output,
                        surface,
                        layer_surface,
                        name,
                        state.wgpu.raw_display_handle,
                        &state.wgpu.instance,
                    ));
                }
                _ => {}
            },
            wl_registry::Event::GlobalRemove { name } => {
                let index = state
                    .outputs
                    .iter()
                    .enumerate()
                    .find(|(_, output)| output.info.id == name)
                    .map(|(index, _)| index);

                if let Some(index) = index {
                    state.outputs.swap_remove(index);
                }
            }
            _ => unreachable!(),
        }
    }
}

delegate_noop!(StatusBar: zxdg_output_manager_v1::ZxdgOutputManagerV1);
delegate_noop!(StatusBar: zwlr_layer_shell_v1::ZwlrLayerShellV1);
delegate_noop!(StatusBar: wl_compositor::WlCompositor);
