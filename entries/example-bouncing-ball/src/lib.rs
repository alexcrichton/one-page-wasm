extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use std::mem;
use wasm_bindgen::prelude::*;

#[derive(Copy, Clone, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
}

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

fn set_pixel(buf: &mut [u8], x: usize, y: usize, color: Color) {
    assert!(buf.len() as usize == WIDTH * HEIGHT * 4);
    buf[x * 4 + y * WIDTH * 4 + 0] = color.r;
    buf[x * 4 + y * WIDTH * 4 + 1] = color.g;
    buf[x * 4 + y * WIDTH * 4 + 2] = color.b;
    buf[x * 4 + y * WIDTH * 4 + 3] = color.a;
}

struct Ball {
    position: [isize; 2],
    velocity: [isize; 2],
    radius: isize,
}

impl Ball {
    fn draw(&self, buf: &mut [u8], color: Color) {
        for dy in -self.radius..self.radius {
            let r = self.radius as f64;
            let width = ((r * r) - (dy as f64 * dy as f64)).sqrt().round() as isize;
            for dx in -width..width {
                let x = self.position[0] + dx;
                let y = self.position[1] + dy;
                set_pixel(buf, x as usize, y as usize, color);
            }
        }
    }

    fn update(&mut self) -> bool {
        self.position = [
            self.position[0] + self.velocity[0],
            self.position[1] + self.velocity[1],
        ];

        let mut hit = false;

        if self.position[0] - self.radius <= 0 || self.position[0] + self.radius >= WIDTH as isize {
            self.velocity[0] = -self.velocity[0];
            self.position[0] += self.velocity[0];
            hit = true;
        }

        if self.position[1] - self.radius <= 0 || self.position[1] + self.radius >= HEIGHT as isize
        {
            self.velocity[1] = -self.velocity[1];
            self.position[1] += self.velocity[1];
            hit = true;
        }

        hit
    }
}

static mut BALL: Ball = Ball {
    position: [111, 37],
    velocity: [5, 3],
    radius: 10,
};

static mut BALL_COLOR: Color = Color::BLACK;
static mut BG_COLOR: Color = Color::WHITE;

#[wasm_bindgen]
pub fn frame(frame_buffer: &mut [u8]) {
    utils::set_panic_hook();

    for y in 0..WIDTH {
        for x in 0..HEIGHT {
            set_pixel(frame_buffer, x, y, unsafe { BG_COLOR });
        }
    }

    unsafe {
        BALL.draw(frame_buffer, unsafe { BALL_COLOR });
        if BALL.update() {
            mem::swap(&mut BALL_COLOR, &mut BG_COLOR);
        }
    }
}
