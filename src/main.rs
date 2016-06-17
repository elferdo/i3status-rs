extern crate libc;
extern crate rustc_serialize;
extern crate time;

use std::time::Duration;
use std::thread;
use std::ffi::CString;
use std::{ptr,mem};

use libc::{c_double, size_t};

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

fn i3b_loadavg() -> I3BarBlock {
    let load_averages: [f64; 3] = unsafe {
        let mut lavgs: [c_double; 3] = [0f64, 0f64, 0f64];
        libc::getloadavg(lavgs.as_mut_ptr(), 3);
        lavgs
    };

    I3BarBlock {
        full_text: format!("{:.2}:{:.2}:{:.2}",
                           load_averages[0], load_averages[1],
                           load_averages[2]).to_string(),
        color:
            if load_averages[0] > 4f64 {
                color(0xff, 0x00, 0x00)
            } else {
                color(0xff, 0xff, 0xff)
            }.to_string(),
    }
}

fn coretemp_get_dev(i: usize) -> f32 {
    let name = CString::new(format!("dev.cpu.{}.temperature", i)).unwrap();
    let mut t: [i32; 1] = [0];
    let mut tlen: size_t = 4;

    unsafe {
        libc::sysctlbyname(name.as_ptr(), mem::transmute(&mut t), &mut tlen, ptr::null(), 0)
    };

    // so, like,
    // https://github.com/freebsd/freebsd/blob/master/sys/dev/coretemp/coretemp.c#L404
    ((t[0] as f32)-2731f32) / 10f32
}

fn get_ncpu() -> usize {
    // dunno why I can't do [CTL_HW, HW_NCPU]
    let name = CString::new("hw.ncpu").unwrap();
    let mut t: [i32; 1] = [0];
    let mut tlen: size_t = 4;

    unsafe {
        libc::sysctlbyname(name.as_ptr(), mem::transmute(&mut t), &mut tlen, ptr::null(), 0)
    };

    t[0] as usize
}

fn i3b_bsd_coretemp() -> I3BarBlock {
    let avg = {
        let cpus = get_ncpu();

        let mut roll = 0f32;
        for c in 0..cpus {
            roll += coretemp_get_dev(c);
        }
        roll /= cpus as f32;

        roll
    };

    let col: String =
        if avg >= 55f32 {
            color(0xff, 0x00, 0x00)
        } else {
            color(0xff, 0xff, 0xff)
        }.to_string();

    I3BarBlock {
        full_text: format!("{:.1} °C", avg).to_string(),
        color: col
    }
}

fn main() {
    let header: I3BarHeader = Default::default();
    println!("{}[", json::encode(&header).unwrap());

    let one_second = Duration::new(1, 0);

    loop {
        let fns: Vec<fn() -> I3BarBlock> = vec![i3b_bsd_coretemp, i3b_loadavg, i3b_clock];

        let bar: Vec<I3BarBlock> = fns.iter().map(|&f| f()).collect();
        println!("{},", json::encode(&bar).unwrap());

        thread::sleep(one_second);
    }
}
