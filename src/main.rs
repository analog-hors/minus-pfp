use std::time::Duration;

use image::{DynamicImage, GenericImage, Rgba, Pixel, GenericImageView};
use image::imageops::{FilterType, overlay};
use noise::{Fbm, NoiseFn, OpenSimplex};

// Parameters
const TIME: Duration = Duration::from_secs(5);
const UPS: f64 = 0.6;
const FPS: f64 = 30.0;
const X_SCALE: f64 = 1.0 / 50.0 * 2.0;
const Y_SCALE: f64 = 1.0 / 50.0;
const RESIZE_OUTPUT: bool = true;
const APPLY_BORDER: bool = false;

fn noise_fn() -> impl NoiseFn<f64, 4> {
    let mut noise = Fbm::<OpenSimplex>::new(0xd9e);
    noise.octaves = 4;
    noise.persistence = 0.75;
    noise
}

fn rescale_noise(n: f64) -> f64 {
    n * 0.25 + 0.75
}
// End parameters

fn num_frames() -> u32 {
    (TIME.as_secs_f64() * FPS) as u32
}

fn main() {
    let noise = noise_fn();
    let gradient = load_image(include_bytes!("gradient.png")).unwrap();
    let text = load_image(include_bytes!("text.png")).unwrap();
    let border = load_image(include_bytes!("border.png")).unwrap();

    for frame in 0..num_frames() {
        let mut image = DynamicImage::new_rgba8(gradient.width(), gradient.height());
        write_noise(&mut image, &noise, frame);
        apply_gradient(&mut image, &gradient);
        overlay_text(&mut image, &text);
        if RESIZE_OUTPUT || APPLY_BORDER {
            resize_to_match_border(&mut image, &border);
        }
        if APPLY_BORDER {
            apply_border(&mut image, &border);
        }
        image.save(&format!("frames/{frame:05}.png")).unwrap();
    }
}

fn write_noise(image: &mut DynamicImage, noise: &impl NoiseFn<f64, 4>, frame: u32) {
    let angle = frame as f64 / num_frames() as f64 * std::f64::consts::TAU;
    let radius = TIME.as_secs_f64() * UPS / std::f64::consts::TAU;
    for y in 0..image.height() {
        for x in 0..image.width() {
            let n = noise.get([
                angle.cos() * radius,
                angle.sin() * radius,
                x as f64 * X_SCALE,
                y as f64 * Y_SCALE
            ]);
            let n = (rescale_noise(n) * u8::MAX as f64).floor() as u8;
            image.put_pixel(x, y, Rgba([n, n, n, u8::MAX]));
        }
    }
}

fn apply_gradient(image: &mut DynamicImage, gradient: &DynamicImage) {
    for y in 0..image.height() {
        for x in 0..image.width() {
            let top = image.get_pixel(x, y);
            let bottom = gradient.get_pixel(x, y);
            let out = top.map2(&bottom, |t, b| { // Color burn
                let t = t as f64 / u8::MAX as f64;
                let b = b as f64 / u8::MAX as f64;
                let o = if t + b <= 1.0 {
                    0.0
                } else if t == 0.0 {
                    1.0
                } else {
                    ((t + b - 1.0) / t).clamp(0.0, 1.0)
                };
                (o * u8::MAX as f64).floor() as u8
            });
            image.put_pixel(x, y, out);
        }
    }
}

fn overlay_text(image: &mut DynamicImage, text: &DynamicImage) {
    overlay(image, text, 0, 0);
}

fn resize_to_match_border(image: &mut DynamicImage, border: &DynamicImage) {
    *image = image.resize(border.width(), border.height(), FilterType::Nearest);
}

fn apply_border(image: &mut DynamicImage, border: &DynamicImage) {
    for y in 0..border.height() {
        for x in 0..border.width() {
            let Rgba([_, _, _, alpha]) = border.get_pixel(x, y);
            if alpha == u8::MAX {
                break;
            }
            image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
        }
        for x in (0..border.width()).rev() {
            let Rgba([_, _, _, alpha]) = border.get_pixel(x, y);
            if alpha == u8::MAX {
                break;
            }
            image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
        }
    }
    overlay(image, border, 0, 0);
}

fn load_image(buf: &[u8]) -> image::ImageResult<DynamicImage> {
    image::io::Reader::new(std::io::Cursor::new(buf)).with_guessed_format()?.decode()
}
