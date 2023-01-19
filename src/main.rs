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

use std::collections::HashMap;
use std::env;
use std::fs::*;
use std::io::*;
use std::path::PathBuf;

use rs_math3d::*;

use crate::atlas::Atlas;

mod atlas;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// path to assets (icons, fonts)
    #[arg(short, long="icon-path", value_name = "icon-path")]
    icon_path: PathBuf,

    /// atlas size
    #[arg(short, long, value_name = "atlas-size")]
    atlas_size: usize,


    /// save to png
    #[arg(long, value_name = "save-png")]
    png_file: Option<String>,

    /// font name
    #[arg(short='f', long="font", value_name = "font")]
    fonts: Vec<String>,

    /// font size
    #[arg(short='s', long="size", value_name = "size")]
    sizes: Vec<usize>,
}

fn list_icons_with_extension(dir: ReadDir) -> Vec<String> {
    let entries = dir
        .filter(|x| match x {
            Ok(t) if t.file_type().unwrap().is_file() => true,
            _ => false,
        })
        .map(|x| x.unwrap().path().to_str().unwrap().to_string())
        .collect::<Vec<_>>();

    entries
        .iter()
        .filter(|x| x.ends_with(".png"))
        .map(|x| x.clone())
        .collect::<Vec<_>>()
}

fn main() {
    let args = Cli::parse();

    let icon_folder = args.icon_path.to_str().unwrap().to_string();
    let icon_dir = match std::fs::read_dir(icon_folder) {
        Ok(r) => r,
        Err(e) => {
            println!("Error reading dir: {}", e);
            return;
        }
    };

    let icons = list_icons_with_extension(icon_dir);
    let mut atlas = Atlas::new(args.atlas_size, args.atlas_size);
    for icon in icons {
        match atlas.add_icon(&icon) {
            Err(e) => {
                println!("Error: Unable to add {}: {}", icon, e.to_string());
                return;
            }
            _ => (),
        };
    }
    if args.fonts.len() != args.sizes.len() {
        println!("Both fonts and size count should be the same");
        return;
    }

    let font_size = args.fonts.iter().zip(args.sizes.iter());
    for (font, size) in font_size {
        match atlas.add_font(font, *size) {
            Err(e) => {
                println!("Error: Unable to add {}: {}", font, e.to_string());
                return;
            }
            _ => (),
        }
    }

    for (i, _) in &atlas.icons {
        println!("icon {}", i);
    }

    for (f, _) in &atlas.fonts {
        println!("font {}", f);
    }

    if args.png_file.is_some() {
        match atlas.save_png_image(args.png_file.unwrap().as_str()) {
            Err(e) => {
                println!("Error: Unable to save PNG file: {}", e);
                return;
            }
            _ => (),
        }
    }

    atlas.save_as_rust("atlas_data.rs");
}
