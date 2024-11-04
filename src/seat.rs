use wayland_client::{
    protocol::{wl_pointer, wl_seat},
    Connection, Dispatch, QueueHandle, WEnum,
};

use crate::StatusBar;

pub struct Pointer {
    pointer: Option<wl_pointer::WlPointer>,
    pub x: i64,
    pub y: i64,
}

pub struct Seat {
    pub seat: wl_seat::WlSeat,
    pub pointer: Option<Pointer>,
}

impl Dispatch<wl_seat::WlSeat, ()> for StatusBar {
    fn event(
        state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let Some(ref mut seat) = state.seat else {
            return;
        };

        match event {
            wl_seat::Event::Capabilities {
                capabilities: WEnum::Value(capabilities),
            } => {
                if capabilities.contains(wl_seat::Capability::Keyboard) {
                    seat.pointer = Some(Pointer {
                        pointer: Some(seat.seat.get_pointer(qh, ())),
                        x: 0,
                        y: 0,
                    })
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for StatusBar {
    fn event(
        state: &mut Self,
        _proxy: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let Some(ref mut seat) = state.seat else {
            return;
        };

        match event {
            wl_pointer::Event::Motion {
                time: _,
                surface_x,
                surface_y,
            } => {
                let pointer = seat.pointer.as_mut().unwrap();
                pointer.x = surface_x as i64;
                pointer.y = surface_y as i64;
            }
            wl_pointer::Event::Button {
                serial,
                time: _,
                button,
                state,
            } => {}
            _ => {}
        }
    }
}
