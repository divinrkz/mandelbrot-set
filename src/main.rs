use core::f32::consts::LOG2_10;
use std::ops::{Add, Mul};
use std::sync::{Arc, Mutex};
use std::thread;

use rayon::prelude::*;

use mandelbrot::*;

const WIDTH: u16 = 500;
const HEIGHT: u16 = 500;

const FRAMERATE: f32 = 24.0;

const KEYFRAMES: [Keyframe; 3] = [
    Keyframe {
        x_center: -0.75,
        y_center: 0.0,
        x_size: 3.5,
        y_size: 3.5,
        index: 0,
    },
    Keyframe {
        x_center: -1.35,
        y_center: 0.0,
        x_size: 0.2,
        y_size: 0.2,
        index: 100,
    },
    Keyframe {
        x_center: -0.75,
        y_center: 0.0,
        x_size: 3.5,
        y_size: 3.5,
        index: 300,
    },
];

const MAX_ITER: usize = 255;

fn main() {
    let mut animation =
        Animation::new("anim.gif", WIDTH, HEIGHT, FRAMERATE).expect("Error creating animation.");

    println!("Collecting frames...");
    let frames = frames_native();
    // let frames = frames_rayon();

    animation.add_frames(frames);
    animation
        .write_animation()
        .expect("Error saving animation.");
}

/// Parallel frame builder that only uses Rust threads and synchronization primitives.
pub fn frames_native() -> Vec<Frame> {
    let keyframes = &KEYFRAMES;
    let interpolated_frames = get_interpolated_frames(keyframes);

    let frames: Vec<Frame> = interpolated_frames
        .iter()
        .map(|_| Frame::empty())
        .collect::<Vec<Frame>>();

    let frames_arc = Arc::new(Mutex::new(frames));

    let mut handles = vec![];

    for (index, keyframe) in interpolated_frames.iter().enumerate() {
        let frames_clone = Arc::clone(&frames_arc);
        let keyframe = *keyframe;

        let handle = thread::spawn(move || {
            let pixel_data = draw_frame(WIDTH as u32, HEIGHT as u32, keyframe);
            let frame = Frame::from_pixels(WIDTH, HEIGHT, pixel_data);

            let mut frames = frames_clone.lock().unwrap();
            frames[index] = frame;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panick.");
    }
    Arc::try_unwrap(frames_arc).unwrap().into_inner().unwrap()
}

/// Parallel frame builder that uses Rayon.
pub fn frames_rayon() -> Vec<Frame> {
    let keyframes = &KEYFRAMES;
    let interpolated_frames: Vec<Keyframe> = get_interpolated_frames(keyframes);

    interpolated_frames
        .par_iter()
        .map(|keyframe| {
            let pixel_data = draw_frame(WIDTH as u32, HEIGHT as u32, *keyframe);
            Frame::from_pixels(WIDTH, HEIGHT, pixel_data)
        })
        .collect()
}

pub fn calc_pixel((x, y): (f32, f32)) -> Pixel {
    let c = Complex::new(x, y);
    let mut z = Complex::new(0.0, 0.0);
    let mut iters = 0;

    while z.norm() < 8192.0 && iters < MAX_ITER {
        z = z * z + c;
        iters += 1;
    }
    if iters < MAX_ITER {
        let log_zn = (z.norm().log2() / 2.0).log2() / LOG2_10;
        let nu = log_zn;
        let intensity = (iters as f32 + 1.0 - nu) / MAX_ITER as f32;
        let r = intensity.powi(2);
        let g = intensity;
        let b = intensity.sqrt();
        Pixel::from_rgb(r, g, b)
    } else {
        Pixel::from_rgb(0.0, 0.0, 0.0)
    }
}

pub fn draw_frame(width: u32, height: u32, keyframe: Keyframe) -> Vec<Pixel> {
    let mut pixels = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let (cx, cy) = keyframe.get_coordinate(x, y, width, height);
            let pixel = calc_pixel((cx, cy));
            pixels.push(pixel);
        }
    }
    pixels
}

#[derive(Clone, Copy, Debug)]
struct Complex {
    x: f32,
    y: f32,
}

impl Complex {
    pub fn new(x: f32, y: f32) -> Self {
        Complex { x, y }
    }

    pub fn norm(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}

impl Add for Complex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Complex {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul for Complex {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Complex {
            x: self.x * rhs.x - self.y * rhs.y,
            y: self.x * rhs.y + self.y * rhs.x,
        }
    }
}
