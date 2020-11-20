#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg};
use walkdir::WalkDir;
use std::path::Path;
use std::collections::HashSet;
use std::fs;
use image::GenericImageView;
use indicatif::ProgressBar;

use rayon::prelude::*;

fn main() {

    let app = App::new("imgresize")
        .setting(AppSettings::NeedsLongHelp)
        .version("1.0")
        .about("Resizes images")
        .arg(
            Arg::with_name("input")
            .help("Sets the input directory")
            .index(1)
            .required(true)
        )
        .arg(
            Arg::with_name("filter")
            .help("Image extensions to resize, such as png or jpg")
            .multiple(true)
            .short("f")
            .takes_value(true)
            .required(true)
        )
        .arg(
            Arg::with_name("size")
            .short("s")
            .takes_value(true)
            .help("Any file larger than this will be resized")
            .default_value("307200")
        )
        .arg(
            Arg::with_name("width")
            .short("w")
            .takes_value(true)
            .help("Width in pixels to resize images to")
            .default_value("1920")
        )
        .arg(
            Arg::with_name("height")
            .short("h")
            .help("Height in pixels to resize images to")
            .takes_value(true)
            .default_value("1080")
        )
        .arg(
            Arg::with_name("verbose")
            .short("v")
            .help("Show verbose information")
        )
        .arg(
            Arg::with_name("quality")
            .short("q")
            .help("Image quality sampling used for scaling down images (between 1-5). Higher values are better quality scaling but take longer.\n1 - Nearest Neighbor\n2 - Linear:Triange\n3 - Cubic: Catmull-Rom\n4 - Gaussian\n5 - Lanczos with window 3\n")
            .takes_value(true)
            .default_value("3")
        );

    let matches = app.get_matches();

    let filter: HashSet<&str> = matches.values_of("filter").unwrap()
        .collect();
    //let size: u64 = matches.value_of("size").and_then(|x| Some(x.parse())).unwrap().unwrap();
    let size = value_t!(matches, "size", u64).unwrap_or_else(|e| e.exit());
    let w: u32 = matches.value_of("width").and_then(|x| Some(x.parse())).unwrap().unwrap();
    let h: u32 = matches.value_of("height").and_then(|x| Some(x.parse())).unwrap().unwrap();

    let verbose = matches.is_present("verbose");
    let quality = value_t!(matches, "quality", u8).unwrap_or_else(|e| e.exit());
    let ops = match quality {
        1 => Some(image::imageops::Nearest),
        2 => Some(image::imageops::Triangle),
        3 => Some(image::imageops::CatmullRom),
        4 => Some(image::imageops::Gaussian),
        5 => Some(image::imageops::Lanczos3),
        _ => None,
    };
    let ops = ops.expect("Error - quality must be between 1-5. Run with --help to see usage / information");

    let mut resize_imgs = Vec::new();

    println!("Gathering files...");

    let input = matches.value_of("input").unwrap();
    for entry in WalkDir::new(input) {
        if let Ok(e) = entry {
            let ext = Path::new(e.file_name()).extension().and_then(|x| x.to_str());
            if let Some(ext) = ext {
                if !filter.contains(ext) {
                    if verbose { println!("- Skip {} (extension)", e.path().display()); }
                    continue;
                }

                if let Ok(metadata) = e.metadata() {
                    if metadata.len() <= size {
                        if verbose { println!("- Skip {} (file size)", e.path().display()); }
                        continue;
                    }

                    let path = String::from(e.path().to_str().unwrap());
                    resize_imgs.push(path);
                }
            }
        }
    }

    println!("Resizing images...");
    let bar = ProgressBar::new(resize_imgs.len() as u64);

    resize_imgs.into_par_iter().for_each(|path|{
        match image::open(&path) {
            Ok(img) => {
                let (width, height) = img.dimensions();
                if width <= w && height <= h {
                    if verbose { println!("- Skip {} (image dimensions)", path); }
                    return;
                }

                let mut perms = fs::metadata(&path).unwrap().permissions();
                if perms.readonly() {
                    perms.set_readonly(false);
                    if let Err(fserr) = fs::set_permissions(&path, perms) {
                        eprintln!("Error setting permissions {}", fserr);
                    }

                    if verbose { println!("- Set readonly false {}", path); }
                }


                if verbose { println!("- Resize: {}", path); }

                let resized = img.resize(w, h, ops);
                resized.save(&path).unwrap();
            },
            Err(err) => {
                eprintln!("Error with file: {}: {}", &path, err);
            }
        }
        bar.inc(1);
    });
    bar.finish();

    println!("Done!");


    /*
    let path = "C:/Users/moran/Pictures/tmp/trees.jpg";

    let img = image::open(path).unwrap();
    let sml = img.resize(1920, 1080, image::imageops::Gaussian);
    
    sml.save("C:/Users/moran/Pictures/tmp/trees.jpg").unwrap();
    */
}
