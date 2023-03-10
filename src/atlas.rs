//
// Copyright 2023-Present (c) Raja Lehtihet & Wael El Oraiby
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
use rect_packer::*;
use std::fmt::{Debug, Formatter};
use std::path::*;

use super::*;

#[derive(Debug)]
pub struct CharEntry {
    pub offset: Vec2i,
    pub advance: Vec2i,
    pub rect: Recti, // coordinates in the atlas
}

pub struct Font {
    line_size: usize,        // line size
    font_size: usize,        // font size in pixels
    entries: Vec<CharEntry>, // all printable chars [32-127]
}

impl Debug for Font {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        let mut entries = String::new();
        for e in &self.entries {
            entries.write_fmt(format_args!("{:?}, ", e))?;
        }
        f.write_fmt(format_args!(
            "Font {{ line_size: {}, font_size: {}, entries: [{}] }}",
            self.line_size, self.font_size, entries
        ))
    }
}

#[derive(Debug)]
pub struct Icon {
    rect: Recti,
}

pub struct Atlas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
    pub fonts: Vec<(String, Font)>,
    pub icons: Vec<(String, Icon)>,

    packer: Packer,
}

impl Atlas {
    pub fn new(width: usize, height: usize) -> Self {
        let config = rect_packer::Config {
            width: width as _,
            height: height as _,

            border_padding: 1,
            rectangle_padding: 1,
        };

        Self {
            width,
            height,
            pixels: vec![0; width * height],
            fonts: Vec::new(),
            icons: Vec::new(),

            packer: Packer::new(config),
        }
    }

    pub fn add_icon(&mut self, path: &str) -> Result<()> {
        let (width, height, pixels) = Self::load_icon(path)?;
        let rect = self.add_tile(width, height, pixels.as_slice())?;
        self.icons.push((Self::format_path(&path), Icon { rect }));
        Ok(())
    }

    pub fn add_font(&mut self, path: &str, size: usize) -> Result<()> {
        let font = Self::load_font(path)?;
        let mut entries = Vec::new();
        let mut min_y = i32::MAX;
        let mut max_y = -i32::MAX;
        for i in 32..127 {
            // Rasterize and get the layout metrics for the letter at font size.
            let ch = i as u8 as char;
            let (metrics, bitmap) = font.rasterize(ch, size as f32);
            let rect = self.add_tile(metrics.width as _, metrics.height as _, bitmap.as_slice())?;
            let ce = CharEntry {
                offset: Vec2i::new(metrics.xmin, metrics.ymin),
                advance: Vec2i::new(metrics.advance_width as _, metrics.advance_height as _),
                rect,
            };
            entries.push(ce);
            min_y = min_y.min(size as i32 - metrics.ymin - metrics.height as i32);
            max_y = max_y.max(size as i32 - metrics.ymin - metrics.height as i32);
        }

        self.fonts.push((
            Self::format_path(path),
            Font {
                line_size: (max_y - min_y) as usize,
                font_size: size,
                entries,
            },
        ));
        Ok(())
    }

    pub fn save_png_image(&self, path: &str) -> Result<()> {
        // png writer
        let file = File::create(path)?;
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width as _, self.height as _); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;

        writer.write_image_data(self.pixels.as_slice())?;
        Ok(())
    }

    pub fn save_as_rust(&self, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let raw_file = File::create(path.to_string() + ".raw")?;
        let ref mut w = BufWriter::new(file);
        let ref mut rw = BufWriter::new(raw_file);

        let header = "
//
// This is an autogenerated FILE, DO NOT CHANGE MANUALLY
//

use rs_math3d::*;

pub struct CharEntry {
    pub offset: Vec2i,
    pub advance: Vec2i,
    pub rect: Recti, // coordinates in the atlas
}

pub struct Font {
    pub line_size: usize,         // line size in pixels
    pub font_size: usize,         // font size in pixels
    pub entries: [CharEntry; 95], // all printable chars [32-127]
}

pub struct Icon {
    pub rect: Recti,
}

pub struct Atlas<'a> {
    pub width: usize,
    pub height: usize,
    pub pixels: &'a [u8],
    pub fonts: &'a [(&'a str, Font)],
    pub icons: &'a [(&'a str, Icon)],
}

";
        w.write(header.as_bytes())?;

        for (idx, (icon, _)) in self.icons.iter().enumerate() {
            w.write(format!("pub const {} : usize = {};\n", icon, idx).as_bytes())?;
        }

        for (idx, (font, _)) in self.fonts.iter().enumerate() {
            w.write(format!("pub const {} : usize = {};\n", font, idx).as_bytes())?;
        }

        w.write_fmt(format_args!(
            "\nconst ICONS : [(&str, Icon); {}] = [\n",
            self.icons.len()
        ))?;
        for (icon, data) in self.icons.iter() {
            w.write_fmt(format_args!("\t(\"{}\", {:?}),\n", icon, data))?;
        }
        w.write("];\n".as_bytes())?;

        w.write_fmt(format_args!(
            "\nconst FONTS : [(&str, Font); {}] = [\n",
            self.fonts.len()
        ))?;
        for (font, data) in self.fonts.iter() {
            w.write_fmt(format_args!("\t(\"{}\", {:?}),\n", font, data))?;
        }
        w.write("];\n".as_bytes())?;

        let s = "

pub const ATLAS : Atlas = Atlas {
";
        w.write(s.as_bytes())?;
        w.write_fmt(format_args!(
            "\twidth: {}, height: {}, ",
            self.width, self.height
        ))?;

        rw.write_all(&self.pixels)?;
        w.write_fmt(format_args!("\tpixels: include_bytes!(\"{}.raw\"),\n", path.to_string()))?;
        w.write_fmt(format_args!("\tfonts: &FONTS,\n"))?;
        w.write("\ticons: &ICONS,\n".as_bytes())?;
        w.write("};\n".as_bytes())?;

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Private methods
////////////////////////////////////////////////////////////////////////////////
impl Atlas {
    fn add_tile(&mut self, width: usize, height: usize, pixels: &[u8]) -> Result<Recti> {
        let rect = self.packer.pack(width as _, height as _, false);
        match rect {
            Some(r) => {
                for y in 0..height {
                    for x in 0..width {
                        self.pixels
                            [(r.x + x as i32 + (r.y + y as i32) * self.width as i32) as usize] =
                            pixels[x + y * width];
                    }
                }
                Ok(Recti::new(r.x, r.y, r.width, r.height))
            }
            None if width != 0 && height != 0 => {
                let error = format!(
                    "Bitmap size of {}x{} is not enough to hold the atlas, please resize",
                    self.width, self.height
                );
                Err(Error::new(ErrorKind::Other, error))
            }
            _ => Ok(Recti::new(0, 0, 0, 0)),
        }
    }

    fn load_icon(path: &str) -> Result<(usize, usize, Vec<u8>)> {
        let mut decoder = png::Decoder::new(File::open(path)?);
        decoder.set_transformations(png::Transformations::normalize_to_color8());
        let mut reader = decoder.read_info().unwrap();
        let mut img_data = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut img_data)?;

        assert_eq!(info.bit_depth, BitDepth::Eight);

        let pixel_size = match info.color_type {
            ColorType::Grayscale => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::Indexed => 1,
            ColorType::Rgb => 3,
            ColorType::Rgba => 4,
        };

        let mut pixels = vec![0u8; (info.width * info.height) as usize];
        let line_size = info.line_size;
        for y in 0..info.height {
            let line = &img_data[(y as usize * line_size)..((y as usize + 1) * line_size)];

            for x in 0..info.width {
                let xx = (x * pixel_size) as usize;
                let color = match info.color_type {
                    ColorType::Grayscale => line[xx],
                    ColorType::GrayscaleAlpha => line[xx + 1],
                    ColorType::Indexed => todo!(),
                    ColorType::Rgb => {
                        ((line[xx] as u32 + line[xx + 1] as u32 + line[xx + 2] as u32) / 3) as u8
                    }
                    ColorType::Rgba => line[xx + 3],
                };
                pixels[(x + y * info.width) as usize] = color;
            }
        }

        Ok((info.width as _, info.height as _, pixels))
    }

    fn load_font(path: &str) -> Result<fontdue::Font> {
        let mut data = Vec::new();
        File::open(path).unwrap().read_to_end(&mut data).unwrap();

        let font = fontdue::Font::from_bytes(data, FontSettings::default())
            .map_err(|error| Error::new(ErrorKind::Other, format!("{}", error)))?;
        Ok(font)
    }

    fn strip_path_to_file(path: &str) -> String {
        let p = Path::new(path);
        p.file_name().unwrap().to_str().unwrap().to_string()
    }

    fn strip_extension(path: &str) -> String {
        let p = Path::new(path);
        p.with_extension("").to_str().unwrap().to_string()
    }

    fn format_path(path: &str) -> String {
        Self::strip_extension(&Self::strip_path_to_file(path))
    }
}
