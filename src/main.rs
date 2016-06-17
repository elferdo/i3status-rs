extern crate libc;
extern crate rustc_serialize;
extern crate time;

use std::time::Duration;
use std::thread;

use libc::{c_int, c_double};

use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable)]
struct I3BarHeader {
    version: usize,
    stop_signal: usize,
    cont_signal: usize,
    click_events: bool
}

impl Default for I3BarHeader {
    fn default() -> Self {
        I3BarHeader {
            version: 1,
            stop_signal: 0,
            cont_signal: 0,
            click_events: false,
        }
    }
}

#[derive(RustcEncodable, RustcDecodable)]
struct I3BarBlock {
    full_text: String,
    color: String,
}

struct Color { r: u8, g: u8, b: u8 }
impl ToString for Color {
    fn to_string(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b).to_string()
    }
}

fn color(r: u8, g: u8, b: u8) -> Color {
    Color { r: r, g: g, b: b }
}

fn i3b_clock() -> I3BarBlock {
    let now = time::now();
    let nowstr = time::strftime("%Y-%m-%d %H:%M:%S", &now).unwrap();

    I3BarBlock {
        full_text: nowstr,
        color: color(0xff, 0xff, 0xff).to_string(),
    }
}

extern {
    fn getloadavg(lavg: *mut c_double, lavg_len: c_int);
}

fn i3b_loadavg() -> I3BarBlock {
    let load_averages: [f64; 3] = unsafe {
        let mut lavgs: [c_double; 3] = [0f64, 0f64, 0f64];
        getloadavg(lavgs.as_mut_ptr(), 3);
        lavgs
    };

    I3BarBlock {
        full_text: format!("{:.2}:{:.2}:{:.2}",
                           load_averages[0], load_averages[1],
                           load_averages[2]).to_string(),
        color: color(0xff, 0xff, 0xff).to_string(),
    } 
}

fn main() {
    let header: I3BarHeader = Default::default();
    println!("{}[", json::encode(&header).unwrap());

    let one_second = Duration::new(1, 0);

    loop {
        let fns: Vec<fn() -> I3BarBlock> = vec![i3b_loadavg, i3b_clock];

        let bar: Vec<I3BarBlock> = fns.iter().map(|&f| f()).collect();
        println!("{},", json::encode(&bar).unwrap());

        thread::sleep(one_second);
    }
}

