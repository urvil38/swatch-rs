use std::cmp;
extern crate image;
use serde::{Deserialize, Serialize};

/// Pixel represent by R(RED), G(GREEN) and B(BLUE) color
#[derive(Clone, Debug, Copy, Eq, Serialize, Deserialize)]
pub struct Pixel {
    pub r: i64,
    pub g: i64,
    pub b: i64,
    #[serde(skip)]
    pub x: i64,
    #[serde(skip)]
    pub y: i64,
}

impl PartialEq for Pixel {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}

#[derive(Debug)]
pub enum Color {
    RED,
    GREEN,
    BLUE,
}

impl Pixel {
    /// adjust pixel R, G, B value by provided factor
    fn adjust_color(&mut self, factor: f64) {
        self.r = clamp((self.r as f64 * factor) as i64, 0, 255);
        self.g = clamp((self.g as f64 * factor) as i64, 0, 255);
        self.b = clamp((self.b as f64 * factor) as i64, 0, 255);
    }

    /// lighter lighten the pixel by provided percent
    fn lighter(&mut self, percent: f64) {
        let factor = 1.0 + (percent / 100.0);
        self.adjust_color(factor);
    }

    /// darker darken the pixel by provided percent
    fn darker(&mut self, percent: f64) {
        let factor = 1.0 - (percent / 100.0);
        self.adjust_color(factor);
    }
}

/// clamp value v between a(min) and b(max)
fn clamp(v: i64, a: i64, b: i64) -> i64 {
    cmp::min(b, cmp::max(a, v))
}

/// find biggest color range of given pixels.
/// it's return RED, GREEN or BLUE if biggest range of pixels is red, green, blue respectively.  
fn find_biggest_range(pixels: &Vec<Pixel>) -> Color {
    let mut r_min = std::i64::MAX;
    let mut r_max = std::i64::MIN;

    let mut g_min = std::i64::MAX;
    let mut g_max = std::i64::MIN;

    let mut b_min = std::i64::MAX;
    let mut b_max = std::i64::MIN;

    for p in pixels {
        r_min = cmp::min(r_min, p.r);
        r_max = cmp::max(r_max, p.r);
        g_min = cmp::min(g_min, p.g);
        g_max = cmp::max(g_max, p.g);
        b_min = cmp::min(b_min, p.b);
        b_max = cmp::max(b_max, p.b);
    }

    let r_range = r_max - r_min;
    let g_range = g_max - g_min;
    let b_range = b_max - b_min;

    let biggest_range = cmp::max(cmp::max(r_range, g_range), b_range);

    if biggest_range == r_range {
        Color::RED
    } else if biggest_range == g_range {
        Color::GREEN
    } else {
        Color::BLUE
    }
}

/// quantize(reduce RGB color space to dominant N number of colors) given pixels using median cut algorithm.
/// it's devide pixels into (2^max_depth) bucket and recursively apply median cut algorithm.
/// https://en.wikipedia.org/wiki/Median_cut
pub fn quantize(
    pixels: &mut Vec<Pixel>,
    depth: usize,
    img_buf: &mut image::RgbImage,
) -> Vec<Pixel> {
    if depth == 0 {
        let mut pixel_median = Pixel {
            r: 0,
            g: 0,
            b: 0,
            x: 0,
            y: 0,
        };
        for p in pixels.iter() {
            pixel_median.r += p.r;
            pixel_median.g += p.g;
            pixel_median.b += p.b;
        }
        pixel_median.r = pixel_median.r / pixels.len() as i64;
        pixel_median.g = pixel_median.g / pixels.len() as i64;
        pixel_median.b = pixel_median.b / pixels.len() as i64;

        for p in pixels.iter() {
            let px = image::Rgb([
                pixel_median.r as u8,
                pixel_median.g as u8,
                pixel_median.b as u8,
            ]);
            img_buf.put_pixel(p.x as u32, p.y as u32, px);
        }
        return vec![pixel_median];
    };

    let biggest_range = find_biggest_range(pixels);

    match biggest_range {
        Color::RED => {
            pixels.sort_unstable_by(|p1, p2| p1.r.cmp(&p2.r));
        }
        Color::GREEN => {
            pixels.sort_unstable_by(|p1, p2| p1.g.cmp(&p2.g));
        }
        Color::BLUE => {
            pixels.sort_unstable_by(|p1, p2| p1.b.cmp(&p2.b));
        }
    };

    let mid = (pixels.len() >> 1) as usize;

    let mut left = quantize(&mut pixels[0..mid].to_vec(), depth - 1, img_buf);
    let mut right = quantize(&mut pixels[mid..].to_vec(), depth - 1, img_buf);

    let mut v = Vec::with_capacity(left.len() + right.len());
    v.append(&mut left);
    v.append(&mut right);

    v
}

/// order given pixel by their luminance in descending order. i.e most lighter to most
/// darker color
pub fn order_by_luminance(pixels: &mut Vec<Pixel>) {
    pixels.sort_unstable_by(|p1, p2| {
        calc_luminance(*p1)
            .partial_cmp(&calc_luminance(*p2))
            .unwrap()
    });
}

/// find out most variant color from the given pixels.
pub fn most_variant_color(pixels: &Vec<Pixel>) -> Pixel {
    let mut hi = 0;
    let mut max = std::i64::MIN;

    for (i, p) in pixels.iter().enumerate() {
        let v = cmp::max(cmp::max(p.r, p.g), p.b) - cmp::min(cmp::min(p.r, p.g), p.b);
        if v > max {
            max = v;
            hi = i;
        }
    }

    pixels[hi]
}

/// calculate luminance from the RGB pixel
fn calc_luminance(p: Pixel) -> f64 {
    0.2126 * p.r as f64 + 0.7152 * p.g as f64 + 0.0722 * p.b as f64
}
