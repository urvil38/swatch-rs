# swatch-rs

![CI](https://github.com/urvil38/swatch-rs/workflows/CI/badge.svg)

Quantize RGB color space of an image to N dominant colors using a [median cut](https://en.wikipedia.org/wiki/Median_cut) algorithm.

```
swatch-rs 1.0.0
utility to quantize image to N dominant color using a median cut algorithm.

USAGE:
    swatch-rs [FLAGS] [OPTIONS] --image <image-path>

FLAGS:
    -h, --help            Prints help information
    -m, --most            Get most dominant color of an image
    -o, --output-image    whether output quantized image or not. if true, it will create a file called
                          "${filename}-quantized.png"
    -V, --version         Prints version information

OPTIONS:
    -d, --debug-type <debug-type>    Debug type print quantized pixels to given format. value can be (html or json).
                                     "html" debug type will create a "swatch-${file_name}.html" and write HTML data into
                                     it [default: json]
    -i, --image <image-path>         Path to an image
    -c, --colors <max-depth>         Number of colors needed in power of 2, ex: for 16 colors pass 4. (i.e. 2^4 = 16)
                                     [default: 4]
```

## Download

- Download appropriate pre-compiled binary from the [release](https://github.com/urvil38/swatch-rs/releases) page.

```bash
# download tar archive using cURL
curl -L https://github.com/urvil38/swatch-rs/releases/download/v1.0.0/swatch-rs-v1.0.0-x86_64-linux.tar.gz > swatch-rs-v1.0.0-x86_64-linux.tar.gz

# untar archive
tar -xvzf swatch-rs-v1.0.0-x86_64-linux.tar.gz
cd swatch-rs

# move it to bin dir (user need to have root privileges. Run the following command as root using sudo.
sudo mv ./swatch-rs /usr/local/bin
```

## Build

- You can compile from source by [installing Cargo](https://crates.io/install)
([Rust's](https://www.rust-lang.org/) package manager)
and building `swatch-rs` using Cargo:

```bash
git clone https://github.com/urvil38/swatch-rs.git
cd swatch-rs
cargo build --release
```

Compilation will probably take a few minutes depending on your machine. The
binary will end up in `./target/release/swatch-rs`.

## Example

**Image: iceland.jpg - dimention: 1920 × 1080 - size: 1.1 Mib**

![iceland.jpg](./docs/iceland.jpg)


- The following command will produce a quantized image and swatch.html file, which contains top N dominant colors(sorted by luminance).

```bash
swatch-rs -i ./docs/iceland.jpg -d html -c 4 -o
```

![swatch.html](./docs/swatch.png)


**Image: quantize-image.jpg - dimention: 1920 × 1080 - size: 418 Kib**

![quantize-image](./docs/quantize-image.jpg)
