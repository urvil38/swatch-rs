use std::process;
extern crate image;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use structopt::StructOpt;
mod lib;
use lib::{most_variant_color, order_by_luminance, quantize, Pixel};

use image::{GenericImageView, ImageBuffer, RgbImage};

#[derive(Debug)]
enum DebugType {
    JSON,
    HTML,
}

fn parse_output_type(s: &str) -> Result<DebugType, String> {
    match s {
        "html" => Ok(DebugType::HTML),
        "json" => Ok(DebugType::JSON),
        _ => Err(format!("{}. value can be (html | json)", s)),
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "swatch-rs",
    about = "utility to quantize image to N dominant color using a median cut algorithm."
)]
struct Opt {
    /// Path to an image
    #[structopt(short = "i", long = "image")]
    image_path: String,

    /// Number of colors needed in power of 2, ex: for 16 colors pass 4. (i.e. 2^4 = 16)
    #[structopt(short = "c", long = "colors", default_value = "4")]
    max_depth: u32,

    /// Debug type print quantized pixels to given format. value can be (html or json). "html" debug type will create a "swatch-${file_name}.html" and write HTML data into it.
    #[structopt(short = "d", long = "debug-type", parse(try_from_str = parse_output_type), case_insensitive = true, default_value = "json")]
    debug_type: DebugType,

    /// whether output quantized image or not. if true, it will create a file called "${filename}-quantized.png"
    #[structopt(short = "o", long = "output-image")]
    output: bool,

    /// Get most dominant color of an image.
    #[structopt(short = "m", long = "most")]
    most_dominant: bool,
}

fn main() -> Result<(), io::Error> {
    let opt = Opt::from_args();

    let image_path = Path::new(&opt.image_path);

    let img = image::open(image_path);
    let img = match img {
        Ok(img) => img,
        Err(error) => {
            println!("{} {}", error, opt.image_path);
            process::exit(1);
        }
    };
    let filename = image_path.file_stem().unwrap().to_str().unwrap();
    let pixel_count = img.pixels().count();

    let mut pixels: Vec<Pixel> = Vec::with_capacity(pixel_count);
    let mut img_buf: RgbImage = ImageBuffer::new(img.width(), img.height());

    for pixel in img.pixels() {
        let p = Pixel {
            r: pixel.2[0] as i64,
            g: pixel.2[1] as i64,
            b: pixel.2[2] as i64,
            x: pixel.0 as i64,
            y: pixel.1 as i64,
        };
        pixels.push(p);
    }

    let mut quantized_pixels = quantize(&mut pixels, opt.max_depth as usize, &mut img_buf);

    if opt.output {
        img_buf.save(format!("{}-quantized.png", filename)).unwrap();
    }

    order_by_luminance(&mut quantized_pixels);

    if opt.most_dominant {
        let p = most_variant_color(&quantized_pixels);
        quantized_pixels.clear();
        quantized_pixels.push(p);
    }

    write_to(opt.debug_type, &quantized_pixels, filename.to_string());

    Ok(())
}

fn write_to(output_type: DebugType, pixels: &[Pixel], filename: String) {
    match output_type {
        DebugType::HTML => print_html(pixels, filename).unwrap(),
        DebugType::JSON => print_json(pixels).unwrap(),
    }
}

fn print_html(pixels: &[Pixel], filename: String) -> Result<(), io::Error> {
    let html = r#"<html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{{:TITLE:}}</title>
        <style>
            html, body { width: 100%; height: 100%; margin: 0; padding: 0}
            body { display: flex; flex-wrap: wrap;}
            .color { width: {{:WIDTH:}}%; height: {{:HEIGHT:}}%;}
        </style>
    </head>
    <body>"#;

    let s;
    if pixels.len() > 1 {
        s = 25;
    } else {
        s = 100;
    }

    let mut body = html
        .replace("{{:TITLE:}}", &filename)
        .replace("{{:WIDTH:}}", &s.to_string())
        .replace("{{:HEIGHT:}}", &s.to_string());

    body.push('\n');
    for p in pixels {
        body.push_str(&format!(
            "\t<div class=\"color\" style=\"background-color: rgb({},{},{})\"></div>\n",
            p.r, p.g, p.b
        ))
    }
    body.push_str("</body>\n</html>");
    fs::write(format!("{}-swatch.html", filename), body.as_bytes())?;

    Ok(())
}

fn print_json(pixels: &[Pixel]) -> Result<(), io::Error> {
    let pixels_json = serde_json::to_string_pretty(pixels).unwrap();
    io::stdout().write_all(pixels_json.as_bytes())?;

    Ok(())
}
