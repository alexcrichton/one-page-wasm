extern crate wasm_bindgen;

use std::mem;

use wasm_bindgen::prelude::*;

struct State {
    time: u32,
    dead: bool,
    bird: u32,
    velocity: i32,
    obstacles: [Obstacle; NOBSTACLE],
}

#[derive(Copy, Clone)]
struct Obstacle {
    pos: u32,
    gap_width: u32,
    gap_pos: u32,
    active: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

struct Frame {
    buf: [[Color; WIDTH as usize]; HEIGHT as usize],
}


const BIRD_HEIGHT: u32 = 7;
const BIRD_WIDTH: u32 = 10;
const BIRD_X: u32 = 10;
const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;
const DAMPEN: u32 = 8;
const INCREASE_VELOCITY: i32 = 20;
const NOBSTACLE: usize = 2;
const OBSTACLE_WIDTH: u32 = 20;
const OBSTACLE_SPEED: u32 = 2;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

const INIT: State = State {
    dead: false,
    time: 0,
    bird: (HEIGHT / 4) * DAMPEN,
    velocity: 0,
    obstacles: [Obstacle {
        pos: 0,
        gap_width: 0,
        gap_pos: 0,
        active: false,
    }; NOBSTACLE],
};

#[wasm_bindgen]
pub fn frame(frame_buffer: &mut [u8], key_down: bool) {
    unsafe {
        static mut STATE: State = INIT;
        if frame_buffer.len() != mem::size_of::<Frame>() {
            debug_assert_eq!(1, 0);
        }
        STATE.frame(&mut *(frame_buffer.as_ptr() as *mut Frame), key_down)
    }
}

impl State {
    fn frame(&mut self, buf: &mut Frame, key_down: bool) {
        match (self.dead, key_down) {
            (true, true) => *self = INIT,
            (true, false) => return,
            (_, false) => self.velocity += 1,
            (_, true) => self.velocity = -INCREASE_VELOCITY,
        }
        self.step();
        self.collide();
        self.draw_bird(buf);
        self.time += 1;
    }

    fn step(&mut self) {
        let next = self.bird as i32 + self.velocity;
        if 0 <= next && (next as u32) < (HEIGHT - BIRD_HEIGHT) * DAMPEN {
            self.bird = next as u32;
        } else {
            self.dead = true;
        }

        for obstacle in self.obstacles.iter_mut() {
            if !obstacle.active {
                continue
            }
            if obstacle.pos < OBSTACLE_SPEED {
                obstacle.active = false;
            } else {
                obstacle.pos -= OBSTACLE_SPEED;
            }
        }

        match self.obstacles.iter().filter(|o| o.active).map(|o| o.pos).max() {
            Some(n) if WIDTH - n < (WIDTH / (NOBSTACLE as u32)) => {}

            // spawn an obstacle
            _ => {
                if let Some(s) = self.obstacles.iter_mut().find(|o| !o.active) {
                    s.active = true;
                    s.pos = WIDTH;

                    let width = 30 + (random() * 20.0) as u32;
                    s.gap_width = width;
                    s.gap_pos = 20 + (random() * (HEIGHT - width - 40) as f64) as u32;
                }
            }
        }
    }

    fn collide(&mut self) {
        let x = BIRD_X;
        let y = self.bird_y();

        for o in self.obstacles.iter().filter(|o| o.active) {
            if self.dead {
                continue
            }
            let collides_top = intersects(
                x, y, BIRD_WIDTH, BIRD_HEIGHT,
                o.pos, 0, OBSTACLE_WIDTH, o.gap_pos,
            );
            let collides_bot = intersects(
                x, y, BIRD_WIDTH, BIRD_HEIGHT,
                o.pos, o.gap_pos + o.gap_width, OBSTACLE_WIDTH,
                    HEIGHT - o.gap_pos - o.gap_width,
            );
            self.dead = collides_top || collides_bot;
        }
    }

    fn draw_bird(&mut self, buf: &mut Frame) {
        for row in buf.buf.iter_mut() {
            for cell in row.iter_mut() {
                if self.dead {
                    *cell = Color::new(0xff, 0xbe, 0xbe);
                } else {
                    *cell = Color::new(0x80, 0xe2, 0x7e);
                }
            }
        }
        for xi in 0..BIRD_WIDTH {
            for yi in 0..BIRD_HEIGHT {
                let x = BIRD_X + xi;
                let y = self.bird_y() + yi;
                if x < WIDTH && y < HEIGHT {
                    let color = BIRD[yi as usize][xi as usize];
                    if color.r != 0xff || color.g != 0xff || color.b != 0xff {
                        buf.set(x, y, color);
                    }
                }
            }
        }
        for o in self.obstacles.iter().filter(|o| o.active) {
            for x in 0..OBSTACLE_WIDTH {
                let x = x + o.pos;
                if x >= WIDTH {
                    continue
                }
                for y in 0..HEIGHT {
                    if y <= o.gap_pos || y >= o.gap_width + o.gap_pos {
                        // bd1616
                        if x == o.pos ||
                            y % 8 == 0 ||
                            x == o.pos + OBSTACLE_WIDTH - 1 ||
                            (y / 8 % 2 == 0) && x == o.pos + OBSTACLE_WIDTH / 2
                        {
                            buf.set(x, y, Color::new(0x88, 0x88, 0x88));
                        } else {
                            buf.set(x, y, Color::new(0xbd, 0x16, 0x16));
                        }
                    }
                }
            }
        }
    }

    fn bird_y(&self) -> u32 {
        self.bird / DAMPEN
    }
}

impl Frame {
    fn set(&mut self, x: u32, y: u32, c: Color) {
        self.buf[y as usize][x as usize] = c;
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }
}

fn intersects(
    x1: u32, y1: u32, w1: u32, h1: u32,
    x2: u32, y2: u32, w2: u32, h2: u32,
) -> bool {
    intersects_range(x1, x1 + w1, x2, x2 + w2) &&
        intersects_range(y1, y1 + h1, y2, y2 + h2)
}

fn intersects_range(a1: u32, b1: u32, a2: u32, b2: u32) -> bool {
    assert!(a1 <= b1);
    assert!(a2 <= b2);
    if b1 - a1 >= a2 - b2 {
        (a1 <= a2 && a2 <= b1) || (a1 <= b2 && b2 <= b1)
    } else {
        (a2 <= a1 && a1 <= b2) || (a2 <= b1 && b1 <= b2)
    }
}

const BIRD: [[Color; BIRD_WIDTH as usize]; BIRD_HEIGHT as usize] = {
    macro_rules! c {
        ($e:expr) => {
            Color {
                r: ($e >> 16) as u8,
                g: ($e >> 8) as u8,
                b: ($e >> 0) as u8,
                a: 0xff,
            }
        }
    }
    [
        [
            c!(0xffffff),
            c!(0xffffff),
            c!(0xffffff),
            c!(0x99b274),
            c!(0xb2b455),
            c!(0x8d8f64),
            c!(0xa6a7a6),
            c!(0xffffff),
            c!(0xffffff),
            c!(0xffffff),
        ],
        [
            c!(0xffffff),
            c!(0xffffff),
            c!(0x929139),
            c!(0xf4ec50),
            c!(0xe7e04c),
            c!(0xcac7be),
            c!(0xfffffe),
            c!(0xbeb9b9),
            c!(0xffffff),
            c!(0xffffff),
        ],
        [
            c!(0x9db492),
            c!(0xd0cac7),
            c!(0xcac8c0),
            c!(0xded780),
            c!(0xd4c843),
            c!(0xc7c3c7),
            c!(0xfdfffd),
            c!(0xd0ced0),
            c!(0xa5aea4),
            c!(0xffffff),
        ],
        [
            c!(0xa5a77e),
            c!(0xffffe5),
            c!(0xffffef),
            c!(0xbebe9a),
            c!(0xe0d949),
            c!(0xd9d26d),
            c!(0xbfb5b8),
            c!(0xcbc0be),
            c!(0x94827e),
            c!(0xffffff),
        ],
        [
            c!(0xffffff),
            c!(0xbfbd67),
            c!(0xd1cf76),
            c!(0xceb94b),
            c!(0xeada55),
            c!(0xbd9c49),
            c!(0xa83129),
            c!(0xab4a2e),
            c!(0xb34841),
            c!(0x8e7656),
        ],
        [
            c!(0xffffff),
            c!(0x637146),
            c!(0xb38032),
            c!(0xf5c550),
            c!(0xf1c450),
            c!(0xbf844e),
            c!(0xb05051),
            c!(0xb65750),
            c!(0xa3424b),
            c!(0xffffff),
        ],
        [
            c!(0xffffff),
            c!(0xffffff),
            c!(0xffffff),
            c!(0xa4884b),
            c!(0xb18d4a),
            c!(0xa38a4a),
            c!(0x6f8252),
            c!(0xffffff),
            c!(0xffffff),
            c!(0xffffff),
        ],
    ]
};
