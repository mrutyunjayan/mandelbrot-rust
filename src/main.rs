use std::fs::File;
use std::str::FromStr;

use image::png::PNGEncoder;
use image::ColorType;
use num::Complex;
use rayon::prelude::*;

//parses a string, looks for a specified seperator, and returns two strings, splitting the two strings at the seperator
fn parse_pair<T: FromStr>(s: &str, seperator: char) -> Option<(T, T)> {
    match s.find(seperator) {
        Some(index) => match (T::from_str(&s[..index]), (T::from_str(&s[index + 1..]))) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
        None => None,
    }
}

//parses a string input
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

// `bounds` is a pair giving the width and height of the image in pixels.
// `pixel` is a (column, row) pair indicating a particular pixel in that image.
// The `upper_left` and `lower_right` parameters are points on the complex
// plane designating the area our image covers.
fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }
    None
}

fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(
        &pixels,
        bounds.0 as u32,
        bounds.1 as u32,
        ColorType::Gray(8),
    )?;

    Ok(())
}

fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert_eq!(pixels.len(), (bounds.0 * bounds.1));

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);

            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                Some(count) => 255 - count as u8, //colour
                None => 0,                        //black
            };
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() != 5 {
        eprintln!("Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT");
        eprintln!(
            "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
            args[0]
        );
        std::process::exit(1);
    }

    let bounds = parse_pair::<usize>(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");

    let mut pixels = vec![0; bounds.0 * bounds.1];

    let bands = pixels.chunks_mut(bounds.0).enumerate().collect::<Vec<_>>();

    bands.into_par_iter().for_each(|(i, band)| {
        let top = i;
        let band_bounds = (bounds.0, 1);
        let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
        let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1), upper_left, lower_right);
        render(band, band_bounds, band_upper_left, band_lower_right);
    });
    write_image(&args[1], &pixels, bounds).expect("error writing PNG file");
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 100),
            (25, 75),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 }
        ),
        Complex { re: -0.5, im: -0.5 }
    );
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10", ','), None);
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

#[test]
fn test_parse_complex() {
    assert_eq!(
        parse_complex("1.25,-0.0625"),
        Some(Complex {
            re: 1.25,
            im: -0.0625
        })
    );
    assert_eq!(parse_complex(",-0.0625"), None);
}
