use raylib::init;
use raylib::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use ttf_parser::{name_id, Face};

const CHARS_PATH: &str = "/home/adduser/fu4seoi3/src/fu4seoi3/romdisk/assets/chars.txt";
const OUTPUT_BASE: &str = "/home/adduser/fu4seoi3/src/fu4seoi3/romdisk/assets";

struct FontFamily {
    name: &'static str,
    path: &'static str,
    sizes: &'static [i32],
}

const FONT_FAMILIES: &[FontFamily] = &[
    FontFamily {
        name: "ds_bios_8",
        path: "../assets/fonts/font.ttf",
        sizes: &[8, 16],
    },
    FontFamily {
        name: "dot_gothic_16",
        path: "../assets/fonts/DotGothic16-Regular.ttf",
        sizes: &[16, 32, 48],
    },
];

struct CharSections {
    ascii: String,
    kana: String,
    cjk: String,
}

fn load_char_sections() -> CharSections {
    let mut sections = CharSections {
        ascii: String::new(),
        kana: String::new(),
        cjk: String::new(),
    };

    let file = match File::open(CHARS_PATH) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ERROR: Cannot open {}: {}", CHARS_PATH, e);
            return sections;
        },
    };

    let reader = BufReader::new(file);
    let mut current_section = "";

    for line in reader.lines().flatten() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("#ascii") {
            current_section = "ascii";
            continue;
        } else if trimmed.starts_with("#kana") {
            current_section = "kana";
            continue;
        } else if trimmed.starts_with("#cjk") {
            current_section = "cjk";
            continue;
        } else if trimmed.starts_with('#') {
            continue;
        }

        let target = match current_section {
            "ascii" => &mut sections.ascii,
            "kana" => &mut sections.kana,
            "cjk" => &mut sections.cjk,
            _ => continue,
        };

        for ch in trimmed.chars() {
            if !ch.is_whitespace() && !target.contains(ch) {
                target.push(ch);
            }
        }
    }

    println!(
        "Loaded sections: ascii={}, kana={}, cjk={}",
        sections.ascii.chars().count(),
        sections.kana.chars().count(),
        sections.cjk.chars().count()
    );

    sections
}

fn get_chars_for_font(family_name: &str, size: i32, sections: &CharSections) -> String {
    match family_name {
        "ds_bios_8" => {
            println!(
                "  {}@{}px: ASCII only ({} chars)",
                family_name,
                size,
                sections.ascii.chars().count()
            );
            sections.ascii.clone()
        },
        "dot_gothic_16" => {
            if size <= 16 {
                let mut chars = sections.ascii.clone();
                for ch in sections.kana.chars() {
                    if !chars.contains(ch) {
                        chars.push(ch);
                    }
                }
                println!(
                    "  {}@{}px: ASCII + Kana ({} chars)",
                    family_name,
                    size,
                    chars.chars().count()
                );
                chars
            } else {
                let mut chars = String::from(" ");
                for ch in sections.cjk.chars() {
                    if !chars.contains(ch) {
                        chars.push(ch);
                    }
                }
                println!(
                    "  {}@{}px: CJK only ({} chars)",
                    family_name,
                    size,
                    chars.chars().count()
                );
                chars
            }
        },
        _ => {
            println!("  {}@{}px: ASCII only (default)", family_name, size);
            sections.ascii.clone()
        },
    }
}

fn generate_font(
    handle: &mut RaylibHandle,
    thread: &RaylibThread,
    family: &FontFamily,
    size: i32,
    chars: &str,
    suffix: &str,
) {
    let font = match handle.load_font_ex(thread, family.path, size, Some(chars)) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("  ERROR loading {}@{}px: {}", family.name, size, e);
            return;
        },
    };

    let base_path = format!("{}/{}_{size}px{suffix}", OUTPUT_BASE, family.name);
    let mut image = match font.texture().load_image() {
        Ok(img) => img,
        Err(e) => {
            eprintln!("  ERROR texture {}@{}px: {:?}", family.name, size, e);
            return;
        },
    };

    image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    let (w, h) = (image.width, image.height);
    image.export_image(&format!("{}.png", base_path));

    let mut fnt = File::create(format!("{}.fnt", base_path)).unwrap();
    writeln!(fnt, "info face=\"{}\" size={} bold=0 italic=0", family.name, size).unwrap();
    writeln!(
        fnt,
        "common lineHeight={} base={} scaleW={} scaleH={} pages=1",
        size, size, w, h
    )
    .unwrap();
    writeln!(fnt, "page id=0 file=\"{}_{size}px{suffix}.png\"", family.name).unwrap();
    writeln!(fnt, "chars count={}", font.glyphCount).unwrap();

    for (i, glyph) in font.chars().iter().enumerate() {
        let rec = unsafe { *font.recs.add(i) };
        writeln!(
            fnt,
            "char id={} x={} y={} width={} height={} xoffset={} yoffset={} xadvance={} page=0",
            glyph.value,
            rec.x as i32,
            rec.y as i32,
            rec.width as i32,
            rec.height as i32,
            glyph.offsetX,
            glyph.offsetY,
            glyph.advanceX
        )
        .unwrap();
    }

    println!("    -> {} ({}x{}, {} glyphs)", base_path, w, h, font.glyphCount);
}

fn main() {
    let (mut handle, thread) = init().size(100, 100).title("font exporter").build();

    println!("\n=== Loading Character Sections ===\n");
    let sections = load_char_sections();

    println!("\n=== Font Generation ===\n");
    for family in FONT_FAMILIES {
        if !std::path::Path::new(family.path).exists() {
            eprintln!("SKIP {}: not found at {}", family.name, family.path);
            continue;
        }
        println!("{}", family.name);
        for &size in family.sizes {
            let chars = get_chars_for_font(family.name, size, &sections);
            generate_font(&mut handle, &thread, family, size, &chars, "");
        }
    }

    println!("\n=== Font Info ===\n");
    for family in FONT_FAMILIES {
        if !std::path::Path::new(family.path).exists() {
            continue;
        }
        let data = match std::fs::read(family.path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let face = match Face::parse(&data, 0) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let name = face
            .names()
            .into_iter()
            .find(|n| n.name_id == name_id::FULL_NAME)
            .and_then(|n| n.to_string())
            .unwrap_or_else(|| "?".to_string());
        println!(
            "{}: {} glyphs, mono={}",
            name,
            face.number_of_glyphs(),
            face.is_monospaced()
        );
    }
}
