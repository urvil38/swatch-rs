use std::process;
extern crate image;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use structopt::StructOpt;
mod lib;
use lib::{Pixel, quantize, most_variant_color, order_by_luminance};

use image::GenericImageView;

#[derive(Debug)]
enum OutputType {
    JSON,
    HTML,
    FILE
}

fn parse_output_type(s: &str) -> Result<OutputType, String> {
    match s {
        "html" => Ok(OutputType::HTML),
        "json" => Ok(OutputType::JSON),
        "file" => Ok(OutputType::FILE),
        _ => Err(format!(
            "Invalid value provided for output {}. value can be html | json | file",
            s
        )),
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "swatch-rs",
    about = "utility to quantize image to N dominant color using median cut algorithm."
)]
struct Opt {
    /// Image file path
    #[structopt(short = "i", long = "image", default_value = "")]
    image_path: String,

    /// Max depth
    #[structopt(short = "d", long = "max-depth", default_value = "4")]
    max_depth: i64,

    /// Output type
    #[structopt(short = "o", long = "output", parse(try_from_str = parse_output_type), default_value = "html", case_insensitive = true)]
    output: OutputType,
}

fn main() -> Result<(), io::Error> {
    let opt = Opt::from_args();

    if opt.image_path == "" {
        println!("error: please provide absolute path of an image!");
        process::exit(1);
    }

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

    for pixel in img.pixels() {
        let p = Pixel {
            r: pixel.2[0] as i64,
            g: pixel.2[1] as i64,
            b: pixel.2[2] as i64,
        };
        pixels.push(p);
    }

    let mut pixels = quantize(&mut pixels, 0, opt.max_depth as usize);
    order_by_luminance(&mut pixels);

    write_to(opt.output, &pixels, filename.to_string());

    Ok(())
}

fn write_to(output_type: OutputType, pixels: &Vec<Pixel>, filename: String) {
    match output_type {
        OutputType::HTML | OutputType::FILE => print_html(pixels, filename, output_type).unwrap(),
        OutputType::JSON => print_json(pixels).unwrap(),
    }
}

fn print_html(
    pixels: &Vec<Pixel>,
    filename: String,
    output_type: OutputType,
) -> Result<(), io::Error> {
    let html = r#"<html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{{:Title:}}</title>
        <style>
            html, body { width: 100%; height: 100%; margin: 0; padding: 0}
            body { display: flex; flex-wrap: wrap;}
            .color { width: 25%; height: 25%;}
        </style>
    </head>
    <body>"#;
    let primary = most_variant_color(&pixels);

    let mut body = String::new();
    body.push_str(&html.replace("{{:Title:}}", filename.as_ref()));
    body.push('\n');
    body.push_str(&format!(
        "\t<div class=\"color\" style=\"background-color: rgb({},{},{})\"></div>\n",
        primary.r, primary.g, primary.b
    ));
    for p in pixels {
        if *p == primary {
            continue;
        }
        body.push_str(&format!(
            "\t<div class=\"color\" style=\"background-color: rgb({},{},{})\"></div>\n",
            p.r, p.g, p.b
        ))
    }
    body.push_str("</body>\n</html>");
    match output_type {
        OutputType::HTML => io::stdout().write_all(body.as_bytes())?,
        OutputType::FILE => fs::write("swatch.html", body.as_bytes())?,
        _ => (),
    }

    Ok(())
}

fn print_json(pixels: &Vec<Pixel>) -> Result<(), io::Error> {
    let pixels_json = serde_json::to_string_pretty(pixels).unwrap();
    io::stdout().write_all(pixels_json.as_bytes())?;

    Ok(())
}
