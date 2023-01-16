//
// Copyright 2021-Present (c) Raja Lehtihet & Wael El Oraiby
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software without
// specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//

use fontdue::*;
use png::BitDepth;
use png::ColorType;
use std::fs::*;
use std::io::*;
use rect_packer::*;

fn write_image(path: &str, width: usize, height: usize, pixels: &[u8]) {
    // png writer
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width as _, height as _); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    
    writer.write_image_data(pixels).unwrap();
}

fn load_image(path: &str) -> (usize, usize, Vec<u8>) {
    let mut decoder = png::Decoder::new(File::open(path).unwrap());
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().unwrap();
    let mut img_data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut img_data).unwrap();

    println!("format   : {:?}", info.color_type);
    println!("data size: {}", img_data.len());
    println!("line size: {}", info.line_size);
    println!("practical: {}x{}x4 = {}", info.width, info.height, info.width * info.height * 4);
    println!("bit depth: {:?}bits", info.bit_depth);

    assert!(info.bit_depth == BitDepth::Eight);

    let pixel_size = match info.color_type {
        ColorType::Grayscale => 1,
        ColorType::GrayscaleAlpha => 2,
        ColorType::Indexed => 1,
        ColorType::Rgb => 3,
        ColorType::Rgba => 4,
    };

    let mut pixels = vec! { 0u8; (info.width * info.height) as usize };
    let line_size = info.line_size;
    for y in 0..info.height {
        let line = &img_data[(y as usize * line_size)..((y as usize + 1) * line_size)];

        for x in 0..info.width {
            let xx = (x * pixel_size) as usize;
            let color = match info.color_type {
                ColorType::Grayscale => line[xx],
                ColorType::GrayscaleAlpha => line[xx + 1],
                ColorType::Indexed => todo!(),
                ColorType::Rgb => ((line[xx] as u32 + line[xx + 1] as u32 + line[xx * 2] as u32) / 3) as u8,
                ColorType::Rgba => line[xx + 3]
            };
            pixels[(x + y * info.width) as usize] = color;

        }
    }

    (info.width as _,
        info.height as _,
        pixels)
}

fn main() {

    let (width, height, icon_data) = load_image("/home/aifu/Downloads/folder-open-outline.png");
    println!("width: {}, height: {}", width, height);

    let config = rect_packer::Config {
        width: 128,
        height: 128,
    
        border_padding: 1,
        rectangle_padding: 1,
    };
    
    let mut pixels = vec! { 0u8; (config.width * config.height) as _ };
    
    let mut data = Vec::new();
    File::open("/usr/local/share/fonts/f/fixedsys_excelsior_301.ttf").unwrap().read_to_end(&mut data).unwrap();
    
    let font = Font::from_bytes(data, FontSettings::default()).unwrap();

    let mut packer = Packer::new(config);

    match packer.pack(width as _, height as _, false) {
        Some(r) => {
            for y in 0..height {
                for x in 0..width {
                    pixels[(r.x + x as i32 + (r.y + y as i32) * config.width) as usize] = icon_data[x + y * width];
                }
            }

        },
        _ => ()
    }

    for i in 32..127 {
        // Rasterize and get the layout metrics for the letter at font size.
        let ch = i as u8 as char;
        let (metrics, bitmap) = font.rasterize(ch, 16.0);
        let rect = packer.pack(metrics.width as _, metrics.height as _, false);
        match rect {
            Some(r) => {
                for y in 0..metrics.height {
                    for x in 0..metrics.width {
                        pixels[(r.x + x as i32 + (r.y + y as i32) * config.width) as usize] = bitmap[x + y * metrics.width];
                    }
                }

            },
            _ => ()
        }

        //println!("metrics: {:?}", metrics);
        //println!("{:?}", bitmap);
        println!("{} - rect: {:?}", ch, rect);
    }

    write_image("atlas.png", config.width as _, config.height as _, pixels.as_slice());

}
