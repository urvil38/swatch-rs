use std::process;
extern crate image;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use structopt::StructOpt;
mod lib;
use lib::{order_by_luminance, quantize, Pixel};

use image::{GenericImageView, ImageBuffer, RgbImage};

#[derive(Debug)]
enum DebugType {
    JSON,
    HTML,
    FILE,
}

fn parse_output_type(s: &str) -> Result<DebugType, String> {
    match s {
        "html" => Ok(DebugType::HTML),
        "json" => Ok(DebugType::JSON),
        "file" => Ok(DebugType::FILE),
        _ => Err(format!("{}. value can be (html | json | file)", s)),
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "swatch-rs",
    about = "utility to quantize image to N dominant color using median cut algorithm."
)]
struct Opt {
    /// Path to an image
    #[structopt(short = "i", long = "image")]
    image_path: String,

    /// Number of colors needed in power of 2, ex: for 16 colors pass 4. (i.e. 2^4 = 16)
    #[structopt(short = "c", long = "colors", default_value = "4")]
    max_depth: u32,

    /// Debug type print quantized pixels to given format. value can be (html, json or file). "file" debug type will create a "swatch.html" and write HTML data into it.
    #[structopt(short = "d", long = "debug-type", parse(try_from_str = parse_output_type), case_insensitive = true)]
    debug_type: Option<DebugType>,

    #[structopt(
        short = "o",
        long = "output-image",
        default_value = "quantize-image.png"
    )]
    output: String,
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
    let filename = image_path.file_name().unwrap().to_str().unwrap();
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

    let mut pixels = quantize(&mut pixels, opt.max_depth as usize, &mut img_buf);

    img_buf.save(opt.output).unwrap();

    order_by_luminance(&mut pixels);

    match opt.debug_type {
        None => (),
        Some(d) => write_to(d, &pixels, filename.to_string()),
    }

    Ok(())
}

fn write_to(output_type: DebugType, pixels: &Vec<Pixel>, filename: String) {
    match output_type {
        DebugType::HTML | DebugType::FILE => print_html(pixels, filename, output_type).unwrap(),
        DebugType::JSON => print_json(pixels).unwrap(),
    }
}

fn print_html(
    pixels: &Vec<Pixel>,
    filename: String,
    output_type: DebugType,
) -> Result<(), io::Error> {
    let html = r#"<html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{{:TITLE:}}</title>
        <style>
            html, body { width: 100%; height: 100%; margin: 0; padding: 0}
            body { display: flex; flex-wrap: wrap;}
            .color { width: 25%; height: 25%;}
        </style>
    </head>
    <body>"#;

    let mut body = String::new();
    body.push_str(&html.replace("{{:TITLE:}}", filename.as_ref()));
    body.push('\n');
    for p in pixels {
        body.push_str(&format!(
            "\t<div class=\"color\" style=\"background-color: rgb({},{},{})\"></div>\n",
            p.r, p.g, p.b
        ))
    }
    body.push_str("</body>\n</html>");
    match output_type {
        DebugType::HTML => io::stdout().write_all(body.as_bytes())?,
        DebugType::FILE => fs::write("swatch.html", body.as_bytes())?,
        _ => (),
    }

    Ok(())
}

fn print_json(pixels: &Vec<Pixel>) -> Result<(), io::Error> {
    let pixels_json = serde_json::to_string_pretty(pixels).unwrap();
    io::stdout().write_all(pixels_json.as_bytes())?;

    Ok(())
}
