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

use rs_math3d::*;

use crate::atlas::Atlas;

mod atlas;

fn list_files_with_extension(dir: ReadDir) -> (Vec<String>, Vec<String>) {
    let entries = dir
        .filter(|x| match x {
            Ok(t) if t.file_type().unwrap().is_file() => true,
            _ => false,
        })
        .map(|x| x.unwrap().path().to_str().unwrap().to_string())
        .collect::<Vec<_>>();

    let icons = entries
        .iter()
        .filter(|x| x.ends_with(".png"))
        .map(|x| x.clone())
        .collect::<Vec<_>>();

    let fonts = entries
        .iter()
        .filter(|x| x.ends_with(".ttf"))
        .map(|x| x.clone())
        .collect::<Vec<_>>();

    (icons, fonts)
}

fn main() {
    let mut args = env::args().peekable();
    let mut arg_val = HashMap::new();

    while args.peek().is_some() {
        let arg = args.next().unwrap();
        if arg.starts_with("--") {
            match args.peek() {
                Some(v) if !v.starts_with("--") => {
                    let key = arg.clone();
                    arg_val.insert(key, args.next().unwrap());
                }
                _ => (),
            }
        }
    }

    if !arg_val.contains_key("--path")
        || !arg_val.contains_key("--font-size")
        || !arg_val.contains_key("--atlas-size")
    {
        println!(
            "usage example: {} --path /path/to/assets --font-size 16 --atlas-size 256",
            env::args().nth(0).unwrap()
        );
        return;
    }

    let folder = arg_val["--path"].clone();
    let dir = match std::fs::read_dir(folder) {
        Ok(r) => r,
        Err(e) => {
            println!("Error reading dir: {}", e);
            return;
        }
    };

    let font_size = match arg_val["--font-size"].parse::<usize>() {
        Ok(s) => s,
        Err(e) => {
            println!("Error: invalide font size: {}", e.to_string());
            return;
        }
    };

    let atlas_size = match arg_val["--atlas-size"].parse::<usize>() {
        Ok(s) => s,
        Err(e) => {
            println!("Error: invalide font size: {}", e.to_string());
            return;
        }
    };

    let (icons, fonts) = list_files_with_extension(dir);
    let mut atlas = Atlas::new(atlas_size, atlas_size);
    for icon in icons {
        match atlas.add_icon(&icon) {
            Err(e) => {
                println!("Error: Unable to add {}: {}", icon, e.to_string());
                return;
            }
            _ => (),
        };
    }
    for font in fonts {
        match atlas.add_font(&font, font_size) {
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

    if arg_val.contains_key("--save-png") {
        match atlas.save_png_image(arg_val["--save-png"].as_str()) {
            Err(e) => {
                println!("Error: Unable to save PNG file: {}", e);
                return;
            }
            _ => (),
        }
    }

    atlas.save_as_rust("atlas_data.rs");
}
