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
        // Can't be called if seat wasn't created
        let seat = state.seat.as_mut().unwrap();

        let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        else {
            return;
        };

        if capabilities.contains(wl_seat::Capability::Pointer) {
            seat.pointer = Some(Pointer {
                pointer: Some(seat.seat.get_pointer(qh, ())),
                x: 0,
                y: 0,
            });
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
        // Can't be called if pointer wasn't created
        let pointer = state.seat.as_mut().unwrap().pointer.as_mut().unwrap();

        match event {
            wl_pointer::Event::Motion {
                time: _,
                surface_x,
                surface_y,
            } => {
                pointer.x = surface_x as i64;
                pointer.y = surface_y as i64;
            }
            wl_pointer::Event::Button {
                serial: _,
                time: _,
                button,
                state,
            } => {
                println!("x: {}, y: {}", pointer.x, pointer.y);
                println!("{} {:#?}", button, state);
            }
            _ => {}
        }
    }
}
