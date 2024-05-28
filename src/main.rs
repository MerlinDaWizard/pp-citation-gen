#![warn(clippy::pedantic)]
#![feature(lazy_cell)]

use std::{
    cell::LazyCell,
    fs::{write, File, OpenOptions},
    time::Duration,
};

use ab_glyph::FontRef;
use image::{
    codecs::gif::{GifEncoder, Repeat},
    imageops::overlay,
    Delay, Frame, GenericImage, GenericImageView, ImageBuffer, Luma, Pixel, Rgba, RgbaImage,
    SubImage,
};
use imageproc::{
    drawing::{draw_text_mut, text_size, Canvas},
    map::map_colors,
};

type ImageType = ImageBuffer<Rgba<u8>, Vec<u8>>;

const STAMP: LazyCell<ImageBuffer<Luma<u8>, Vec<u8>>> = LazyCell::new(|| {
    image::load_from_memory(include_bytes!("../data/stamp.png"))
        .unwrap()
        .to_luma8()
});
const BARCODE: LazyCell<ImageBuffer<Luma<u8>, Vec<u8>>> = LazyCell::new(|| {
    image::load_from_memory(include_bytes!("../data/barcode.png"))
        .unwrap()
        .to_luma8()
});

pub struct CitationData<'a> {
    width: u32,
    height: u32,
    bg_colour: Rgba<u8>,
    fg_colour: Rgba<u8>,
    decoration_colour: Rgba<u8>,
    font: FontRef<'a>,
    font_size: f32,
    header_text: String,
    violation_text: [Option<String>; 4],
    punishment_text: String,
}

impl<'a> Default for CitationData<'a> {
    fn default() -> Self {
        Self {
            width: 366,
            height: 160,
            bg_colour: Rgba([243, 215, 230, 255]),
            fg_colour: Rgba([90, 85, 89, 255]),
            decoration_colour: Rgba([191, 168, 168, 255]),
            font: FontRef::try_from_slice(include_bytes!("../data/BMmini.TTF")).unwrap(),
            font_size: 16.0,
            header_text: "M.O.A. CITATION".to_string(),
            violation_text: [
                Some("Protocol Violated".to_string()),
                Some("Entry Permit: Invalid Name".to_string()),
                None,
                None,
            ],
            punishment_text: "LAST WARNING - NO PENALTY".to_string(),
        }
    }
}

fn main() {
    let config = CitationData {
        ..Default::default()
    };
    // let img = generate(config);
    // img.save("output.png").unwrap();

    let data = generate_gif(&config);
    write("output.gif", data).unwrap();
}

#[must_use]
pub fn generate(config: &CitationData) -> ImageType {
    // Empty image with bg colour.
    let mut img = RgbaImage::from_pixel(config.width, config.height, config.bg_colour.to_rgba());

    // Top dotted line
    dotted_row(
        &mut img,
        config.decoration_colour,
        0,
        0,
        config.width - 2, // -2, one to because 0 index, 1 because 2x2 shape.
        2,
        2,
    );
    // Bottom dotted line, starts offset one square
    dotted_row(
        &mut img,
        config.decoration_colour,
        config.height - 2, // -2, one to because 0 index, 1 because 2x2 shape.
        2,
        config.width - 2, // -2, one to because 0 index, 1 because 2x2 shape.
        2,
        2,
    );

    // Right side line solid line
    for y in 0..config.height {
        img.put_pixel(config.width - 1, y, config.decoration_colour);
        img.put_pixel(config.width - 2, y, config.decoration_colour);
    }

    // Header end line
    dotted_row(&mut img, config.fg_colour, 34, 16, 344, 2, 2);

    // Stamp
    let coloured_stamp: ImageType = map_colors(&STAMP.clone(), |p| {
        // Not needed with default stamp. But oh well.
        config
            .decoration_colour
            .map_with_alpha(|c| c, |a| (f32::from(a) * (f32::from(p.0[0]) / 255.)) as u8)
    });
    overlay(&mut img, &coloured_stamp, 150, 88);

    // Side indents
    dotted_column(&mut img, config.decoration_colour, 4, 6, 150, 6, 12);
    dotted_column(&mut img, config.decoration_colour, 352, 6, 150, 6, 12);

    // Barcode
    let coloured_barcode: ImageType = map_colors(&BARCODE.clone(), |p| {
        // Not needed with default barcode. But oh well.
        config
            .fg_colour
            .map_with_alpha(|c| c, |a| (f32::from(a) * (f32::from(p.0[0]) / 255.)) as u8)
    });
    overlay(&mut img, &coloured_barcode, 316, 6);

    // Crime end line
    dotted_row(&mut img, config.fg_colour, 114, 16, 344, 2, 2);

    // Header text
    draw_text_mut(
        &mut img,
        config.fg_colour,
        22,
        8,
        config.font_size,
        &config.font,
        &config.header_text,
    );

    // Violation text
    for (idx, txt) in config.violation_text.iter().enumerate() {
        if let Some(txt) = txt {
            draw_text_mut(
                &mut img,
                config.fg_colour,
                22,
                44 + (18 * idx as i32),
                config.font_size,
                &config.font,
                txt,
            );
        }
    }

    // Punishment text, centered. A bit off.
    let width = text_size(2., &config.font, &config.punishment_text).0 as i32;
    draw_text_mut(
        &mut img,
        config.fg_colour,
        66 - (width >> 1),
        130,
        config.font_size,
        &config.font,
        &config.punishment_text,
    );

    img
}

fn dotted_row<C>(
    canvas: &mut C,
    color: <C as Canvas>::Pixel,
    vertical_pos: u32,
    horizontal_start: u32,
    horizontal_end: u32,
    size: u32,
    // Distance between dots in pixels
    distance: u32,
) where
    C: Canvas,
{
    for x in (horizontal_start..=horizontal_end).step_by((distance + size) as usize) {
        for dx in 0..size {
            for dy in 0..size {
                canvas.draw_pixel(x + dx, vertical_pos + dy, color);
            }
        }
    }
}

fn dotted_column<C>(
    canvas: &mut C,
    color: <C as Canvas>::Pixel,
    horizontal_pos: u32,
    vertical_start: u32,
    vertical_end: u32,
    size: u32,
    // Distance between dots in pixels
    distance: u32,
) where
    C: Canvas,
{
    for y in (vertical_start..=vertical_end).step_by((distance + size) as usize) {
        for dx in 0..size {
            for dy in 0..size {
                canvas.draw_pixel(horizontal_pos + dx, y + dy, color);
            }
        }
    }
}

fn generate_gif(config: &CitationData) -> Vec<u8> {
    let img = generate(config);
    let mut frames = Vec::new();
    for i in 0..153 {
        let ratio = i as f32 / (152.0 - 60.0);
        let height = 30 + (ratio * (img.height() - 30) as f32).round() as u32;
        let height = height.clamp(1, img.height());
        let mut frame = RgbaImage::from_pixel(config.width, config.height, Rgba([0, 0, 0, 0]));
        let view: SubImage<&ImageBuffer<Rgba<u8>, Vec<u8>>> = img.view(0, 0, img.width(), height);
        frame.copy_from(&*view, 0, img.height() - height).unwrap();
        // frame.save(format!("output_frames/frame{}.gif", i)).unwrap();

        frames.push(Frame::from_parts(
            frame,
            0,
            0,
            Delay::from_saturating_duration(Duration::from_millis(30)),
        ))
    }

    let mut gif_data = Vec::new();
    let mut encoder = GifEncoder::new(&mut gif_data);
    encoder.set_repeat(Repeat::Infinite).unwrap();
    encoder.encode_frames(frames).unwrap();
    drop(encoder);
    return gif_data;
}
