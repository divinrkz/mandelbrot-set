use std::fs::File;
use std::path::Path;

#[derive(Clone, Copy)]
pub struct Keyframe {
    pub x_center: f32,
    pub y_center: f32,
    pub x_size: f32,
    pub y_size: f32,
    pub index: usize,
}

impl Keyframe {
    fn interpolate(&self, other: Keyframe, idx: usize) -> Self {
        let t = (idx - self.index) as f32 / (other.index - self.index) as f32;
        let flerp = |a, b| a + (b - a) * t;
        Keyframe {
            x_center: flerp(self.x_center, other.x_center),
            y_center: flerp(self.y_center, other.y_center),
            x_size: flerp(self.x_size, other.x_size),
            y_size: flerp(self.y_size, other.y_size),
            index: idx,
        }
    }

    pub fn get_coordinate(&self, x: u32, y: u32, width: u32, height: u32) -> (f32, f32) {
        let x_offset = self.x_center - self.x_size / 2.0;
        let x = (x as f32 / width as f32) * self.x_size + x_offset;

        let y_offset = self.y_center + self.y_size / 2.0;
        let y = y_offset - (y as f32 / height as f32) * self.y_size;

        (x, y)
    }
}

pub fn get_interpolated_frames(keyframes: &[Keyframe]) -> Vec<Keyframe> {
    keyframes
        .windows(2)
        .flat_map(|window| {
            let start = window[0];
            let end = window[1];
            (start.index..end.index).map(move |idx| start.interpolate(end, idx))
        })
        .collect()
}

#[derive(Debug)]
pub enum AnimationError {
    FileCreateError,
    EncoderError,
    FrameCreateError,
    FrameEncodeError,
}

pub struct Animation {
    delay: u16,
    encoder: gif::Encoder<File>,
    frames: Vec<gif::Frame<'static>>,
}

impl Animation {
    pub fn new(
        path: impl AsRef<Path>,
        width: u16,
        height: u16,
        framerate: f32,
    ) -> Result<Self, AnimationError> {
        let file = File::create(path).map_err(|_| AnimationError::FileCreateError)?;
        let encoder = gif::Encoder::new(file, width, height, &[])
            .map_err(|_| AnimationError::EncoderError)?;

        let delay = (100.0 / framerate) as u16;

        let frames = Vec::new();

        Ok(Self {
            encoder,
            delay,
            frames,
        })
    }

    pub fn add_frames(&mut self, frames: Vec<Frame>) {
        self.frames.extend(frames.into_iter().map(|f| f.inner));
    }

    pub fn write_animation(self) -> Result<(), AnimationError> {
        let mut encoder = self.encoder;
        let delay = self.delay;
        self.frames
            .into_iter()
            .map(|mut frame| {
                frame.delay = delay;
                encoder.write_frame(&frame)
            })
            .collect::<Result<(), _>>()
            .map_err(|_| AnimationError::FrameEncodeError)
    }
}

pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        let r = (255.0 * r) as u8;
        let g = (255.0 * g) as u8;
        let b = (255.0 * b) as u8;

        Self { r, g, b, a: 255 }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    inner: gif::Frame<'static>,
}

impl Frame {
    pub fn empty() -> Self {
        Self {
            inner: gif::Frame::from_rgb(0, 0, &[]),
        }
    }

    pub fn from_pixels(width: u16, height: u16, pixels: Vec<Pixel>) -> Self {
        assert!(pixels.len() == width as usize * height as usize);

        let mut buffer = Vec::with_capacity(4 * pixels.len());
        for pixel in pixels {
            buffer.push(pixel.r);
            buffer.push(pixel.g);
            buffer.push(pixel.b);
            buffer.push(pixel.a);
        }

        let frame = gif::Frame::from_rgba(width, height, &mut buffer);

        Self { inner: frame }
    }
}
