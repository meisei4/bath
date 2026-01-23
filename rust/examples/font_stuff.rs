use raylib::ffi::{ImageDrawRectangle, ImageClearBackground};
use raylib::init;
use raylib::prelude::*;
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

// Hexagram sizes to generate (matches dot_gothic CJK sizes)
const HEXAGRAM_SIZES: &[i32] = &[16, 32, 48];

// 8 Trigrams - binary encoding (bottom to top: line1, line2, line3)
// 1 = yang (solid), 0 = yin (broken)
const TRIGRAM_BINARY: [u8; 8] = [
    0b111, // 0: ☰ 乾 qián  - Heaven
    0b011, // 1: ☱ 兌 duì   - Lake
    0b101, // 2: ☲ 離 lí    - Fire
    0b001, // 3: ☳ 震 zhèn  - Thunder
    0b110, // 4: ☴ 巽 xùn   - Wind
    0b010, // 5: ☵ 坎 kǎn   - Water
    0b100, // 6: ☶ 艮 gèn   - Mountain
    0b000, // 7: ☷ 坤 kūn   - Earth
];

// Unicode codepoints for trigrams (U+2630-U+2637)
const TRIGRAM_CODEPOINTS: [u32; 8] = [
    0x2630, 0x2631, 0x2632, 0x2633, 0x2634, 0x2635, 0x2636, 0x2637,
];

// 64 Hexagrams - King Wen sequence
// Binary encoding: bits 0-2 = lower trigram, bits 3-5 = upper trigram
// Each line read bottom to top (line 1 = bit 0, line 6 = bit 5)
const HEXAGRAM_BINARY: [u8; 64] = [
    0b111_111, //  1: 乾 qián      - Creative (Heaven/Heaven)
    0b000_000, //  2: 坤 kūn       - Receptive (Earth/Earth)
    0b010_001, //  3: 屯 zhūn      - Difficulty (Thunder/Water)
    0b100_010, //  4: 蒙 méng      - Folly (Water/Mountain)
    0b010_111, //  5: 需 xū        - Waiting (Heaven/Water)
    0b111_010, //  6: 訟 sòng      - Conflict (Water/Heaven)
    0b000_010, //  7: 師 shī       - Army (Water/Earth)
    0b010_000, //  8: 比 bǐ        - Holding (Earth/Water)
    0b110_111, //  9: 小畜 xiǎo chù - Small Taming (Heaven/Wind)
    0b111_011, // 10: 履 lǚ        - Treading (Lake/Heaven)
    0b000_111, // 11: 泰 tài       - Peace (Heaven/Earth)
    0b111_000, // 12: 否 pǐ        - Standstill (Earth/Heaven)
    0b111_101, // 13: 同人 tóng rén - Fellowship (Fire/Heaven)
    0b101_111, // 14: 大有 dà yǒu  - Great Possession (Heaven/Fire)
    0b000_100, // 15: 謙 qiān      - Modesty (Mountain/Earth)
    0b001_000, // 16: 豫 yù        - Enthusiasm (Earth/Thunder)
    0b011_001, // 17: 隨 suí       - Following (Thunder/Lake)
    0b100_110, // 18: 蠱 gǔ        - Decay (Wind/Mountain)
    0b000_011, // 19: 臨 lín       - Approach (Lake/Earth)
    0b110_000, // 20: 觀 guān      - Contemplation (Earth/Wind)
    0b101_001, // 21: 噬嗑 shì kè  - Biting Through (Thunder/Fire)
    0b100_101, // 22: 賁 bì        - Grace (Fire/Mountain)
    0b100_000, // 23: 剝 bō        - Splitting (Earth/Mountain)
    0b000_001, // 24: 復 fù        - Return (Thunder/Earth)
    0b111_001, // 25: 無妄 wú wàng - Innocence (Thunder/Heaven)
    0b100_111, // 26: 大畜 dà chù  - Great Taming (Heaven/Mountain)
    0b100_001, // 27: 頤 yí        - Nourishment (Thunder/Mountain)
    0b011_110, // 28: 大過 dà guò  - Great Exceeding (Wind/Lake)
    0b010_010, // 29: 坎 kǎn       - Abysmal (Water/Water)
    0b101_101, // 30: 離 lí        - Clinging (Fire/Fire)
    0b011_100, // 31: 咸 xián      - Influence (Mountain/Lake)
    0b001_110, // 32: 恆 héng      - Duration (Wind/Thunder)
    0b111_100, // 33: 遯 dùn       - Retreat (Mountain/Heaven)
    0b001_111, // 34: 大壯 dà zhuàng - Great Power (Heaven/Thunder)
    0b101_000, // 35: 晉 jìn       - Progress (Earth/Fire)
    0b000_101, // 36: 明夷 míng yí - Darkening (Fire/Earth)
    0b110_101, // 37: 家人 jiā rén - Family (Fire/Wind)
    0b101_011, // 38: 睽 kuí       - Opposition (Lake/Fire)
    0b010_100, // 39: 蹇 jiǎn      - Obstruction (Mountain/Water)
    0b001_010, // 40: 解 xiè       - Deliverance (Water/Thunder)
    0b100_011, // 41: 損 sǔn       - Decrease (Lake/Mountain)
    0b110_001, // 42: 益 yì        - Increase (Thunder/Wind)
    0b011_111, // 43: 夬 guài      - Breakthrough (Heaven/Lake)
    0b111_110, // 44: 姤 gòu       - Coming to Meet (Wind/Heaven)
    0b011_000, // 45: 萃 cuì       - Gathering (Earth/Lake)
    0b000_110, // 46: 升 shēng     - Pushing Up (Wind/Earth)
    0b011_010, // 47: 困 kùn       - Oppression (Water/Lake)
    0b010_110, // 48: 井 jǐng      - Well (Wind/Water)
    0b011_101, // 49: 革 gé        - Revolution (Fire/Lake)
    0b101_110, // 50: 鼎 dǐng      - Cauldron (Wind/Fire)
    0b001_001, // 51: 震 zhèn      - Arousing (Thunder/Thunder)
    0b100_100, // 52: 艮 gèn       - Keeping Still (Mountain/Mountain)
    0b110_100, // 53: 漸 jiàn      - Development (Mountain/Wind)
    0b001_011, // 54: 歸妹 guī mèi - Marrying Maiden (Lake/Thunder)
    0b001_101, // 55: 豐 fēng      - Abundance (Fire/Thunder)
    0b101_100, // 56: 旅 lǚ        - Wanderer (Mountain/Fire)
    0b110_110, // 57: 巽 xùn       - Gentle (Wind/Wind)
    0b011_011, // 58: 兌 duì       - Joyous (Lake/Lake)
    0b110_010, // 59: 渙 huàn      - Dispersion (Water/Wind)
    0b010_011, // 60: 節 jié       - Limitation (Lake/Water)
    0b110_011, // 61: 中孚 zhōng fú - Inner Truth (Lake/Wind)
    0b001_100, // 62: 小過 xiǎo guò - Small Exceeding (Mountain/Thunder)
    0b010_101, // 63: 既濟 jì jì   - After Completion (Fire/Water)
    0b101_010, // 64: 未濟 wèi jì  - Before Completion (Water/Fire)
];

// Unicode codepoints for hexagrams (U+4DC0-U+4DFF)
const HEXAGRAM_CODEPOINT_BASE: u32 = 0x4DC0;

struct CharSections {
    ascii: String,
    kana: String,
    cjk: String,
    yijing: String,
}

fn load_char_sections() -> CharSections {
    let mut sections = CharSections {
        ascii: String::new(),
        kana: String::new(),
        cjk: String::new(),
        yijing: String::new(),
    };

    let file = match File::open(CHARS_PATH) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ERROR: Cannot open {}: {}", CHARS_PATH, e);
            return sections;
        }
    };

    let reader = BufReader::new(file);
    let mut current_section = "";

    for line in reader.lines().flatten() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_lowercase();
        if lower.starts_with("#ascii") {
            current_section = "ascii";
            continue;
        } else if lower.starts_with("#kana") {
            current_section = "kana";
            continue;
        } else if lower.starts_with("#cjk") {
            current_section = "cjk";
            continue;
        } else if lower.starts_with("#yijing") {
            current_section = "yijing";
            continue;
        } else if trimmed.starts_with('#') {
            current_section = "";
            continue;
        }

        let target = match current_section {
            "ascii" => &mut sections.ascii,
            "kana" => &mut sections.kana,
            "cjk" => &mut sections.cjk,
            "yijing" => &mut sections.yijing,
            _ => continue,
        };

        for ch in trimmed.chars() {
            if !ch.is_whitespace() && !target.contains(ch) {
                target.push(ch);
            }
        }
    }

    println!(
        "Loaded sections: ascii={}, kana={}, cjk={}, yijing={}",
        sections.ascii.chars().count(),
        sections.kana.chars().count(),
        sections.cjk.chars().count(),
        sections.yijing.chars().count()
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
        }
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
                // CJK sizes get CJK + YIJING characters
                let mut chars = String::from(" ");
                for ch in sections.cjk.chars() {
                    if !chars.contains(ch) {
                        chars.push(ch);
                    }
                }
                for ch in sections.yijing.chars() {
                    if !chars.contains(ch) {
                        chars.push(ch);
                    }
                }
                println!(
                    "  {}@{}px: CJK + Yijing ({} chars)",
                    family_name,
                    size,
                    chars.chars().count()
                );
                chars
            }
        }
        _ => {
            println!("  {}@{}px: ASCII only (default)", family_name, size);
            sections.ascii.clone()
        }
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
        }
    };

    let base_path = format!("{}/{}_{size}px{suffix}", OUTPUT_BASE, family.name);
    let mut image = match font.texture().load_image() {
        Ok(img) => img,
        Err(e) => {
            eprintln!("  ERROR texture {}@{}px: {:?}", family.name, size, e);
            return;
        }
    };

    image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    let (w, h) = (image.width, image.height);
    image.export_image(&format!("{}.png", base_path));

    let mut fnt = File::create(format!("{}.fnt", base_path)).unwrap();
    writeln!(
        fnt,
        "info face=\"{}\" size={} bold=0 italic=0",
        family.name, size
    )
    .unwrap();
    writeln!(
        fnt,
        "common lineHeight={} base={} scaleW={} scaleH={} pages=1",
        size, size, w, h
    )
    .unwrap();
    writeln!(
        fnt,
        "page id=0 file=\"{}_{size}px{suffix}.png\"",
        family.name
    )
    .unwrap();
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

    println!(
        "    -> {} ({}x{}, {} glyphs)",
        base_path, w, h, font.glyphCount
    );
}

/// Draw a single line (yang=solid, yin=broken) into the image
fn draw_line(image: &mut Image, x: i32, y: i32, width: i32, thickness: i32, is_yang: bool, gap: i32) {
    let color = Color::WHITE;
    if is_yang {
        // Solid line
        unsafe {
            ImageDrawRectangle(image as *mut _ as *mut raylib::ffi::Image, x, y, width, thickness, color.into());
        }
    } else {
        // Broken line (two segments with gap in center)
        let seg_width = (width - gap) / 2;
        unsafe {
            ImageDrawRectangle(image as *mut _ as *mut raylib::ffi::Image, x, y, seg_width, thickness, color.into());
            ImageDrawRectangle(image as *mut _ as *mut raylib::ffi::Image, x + seg_width + gap, y, seg_width, thickness, color.into());
        }
    }
}

/// Draw a trigram (3 lines) into the image at the specified position
fn draw_trigram(image: &mut Image, binary: u8, cell_x: i32, cell_y: i32, cell_size: i32) {
    let padding = cell_size / 8;
    let line_width = cell_size - padding * 2;
    let line_thickness = (cell_size - padding * 2) / 5;
    let line_spacing = line_thickness + line_thickness / 2;
    let yin_gap = cell_size / 5;

    // Center vertically for 3 lines
    let total_height = line_thickness * 3 + line_spacing * 2;
    let start_y = cell_y + (cell_size - total_height) / 2;

    for line in 0..3 {
        let is_yang = (binary >> line) & 1 == 1;
        let y = start_y + (2 - line as i32) * (line_thickness + line_spacing);
        draw_line(image, cell_x + padding, y, line_width, line_thickness, is_yang, yin_gap);
    }
}

/// Draw a hexagram (6 lines) into the image at the specified position
fn draw_hexagram(image: &mut Image, binary: u8, cell_x: i32, cell_y: i32, cell_size: i32) {
    let padding = cell_size / 8;
    let line_width = cell_size - padding * 2;
    let line_thickness = (cell_size - padding * 2) / 9;
    let line_spacing = line_thickness / 2;
    let yin_gap = cell_size / 5;

    // Center vertically for 6 lines
    let total_height = line_thickness * 6 + line_spacing * 5;
    let start_y = cell_y + (cell_size - total_height) / 2;

    for line in 0..6 {
        let is_yang = (binary >> line) & 1 == 1;
        // Draw from bottom to top (line 0 at bottom, line 5 at top)
        let y = start_y + (5 - line as i32) * (line_thickness + line_spacing);
        draw_line(image, cell_x + padding, y, line_width, line_thickness, is_yang, yin_gap);
    }
}

/// Generate hexagram font atlas and BMFont metadata
fn generate_hexagram_font(size: i32) {
    println!("  yijing_hex@{}px: 8 trigrams + 64 hexagrams (72 glyphs)", size);

    // Atlas layout: 8 columns, 9 rows (row 0 = trigrams, rows 1-8 = hexagrams)
    let cols = 8;
    let rows = 9;
    let atlas_w = cols * size;
    let atlas_h = rows * size;

    // Create blank RGBA image
    let mut image = Image::gen_image_color(atlas_w, atlas_h, Color::BLANK);

    // Draw 8 trigrams in row 0
    for i in 0..8 {
        let cell_x = (i as i32) * size;
        let cell_y = 0;
        draw_trigram(&mut image, TRIGRAM_BINARY[i], cell_x, cell_y, size);
    }

    // Draw 64 hexagrams in rows 1-8
    for i in 0..64 {
        let col = i % 8;
        let row = (i / 8) + 1; // +1 to skip trigram row
        let cell_x = (col as i32) * size;
        let cell_y = (row as i32) * size;
        draw_hexagram(&mut image, HEXAGRAM_BINARY[i], cell_x, cell_y, size);
    }

    // Ensure RGBA8888 format
    image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);

    // Export PNG
    let base_path = format!("{}/yijing_hex_{}px", OUTPUT_BASE, size);
    image.export_image(&format!("{}.png", base_path));

    // Generate BMFont .fnt file
    let mut fnt = File::create(format!("{}.fnt", base_path)).unwrap();
    writeln!(fnt, "info face=\"yijing_hex\" size={} bold=0 italic=0", size).unwrap();
    writeln!(
        fnt,
        "common lineHeight={} base={} scaleW={} scaleH={} pages=1",
        size, size, atlas_w, atlas_h
    ).unwrap();
    writeln!(fnt, "page id=0 file=\"yijing_hex_{}px.png\"", size).unwrap();
    writeln!(fnt, "chars count=72").unwrap();

    // Write trigram char entries (Unicode U+2630-U+2637)
    for i in 0..8 {
        let x = (i as i32) * size;
        let y = 0;
        writeln!(
            fnt,
            "char id={} x={} y={} width={} height={} xoffset=0 yoffset=0 xadvance={} page=0",
            TRIGRAM_CODEPOINTS[i],
            x, y, size, size, size
        ).unwrap();
    }

    // Write hexagram char entries (Unicode U+4DC0-U+4DFF)
    for i in 0..64 {
        let col = i % 8;
        let row = (i / 8) + 1;
        let x = (col as i32) * size;
        let y = (row as i32) * size;
        let codepoint = HEXAGRAM_CODEPOINT_BASE + (i as u32);
        writeln!(
            fnt,
            "char id={} x={} y={} width={} height={} xoffset=0 yoffset=0 xadvance={} page=0",
            codepoint,
            x, y, size, size, size
        ).unwrap();
    }

    println!("    -> {} ({}x{}, 72 glyphs)", base_path, atlas_w, atlas_h);
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

    println!("\nyijing_hex (procedural)");
    for &size in HEXAGRAM_SIZES {
        generate_hexagram_font(size);
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

    println!("yijing_hex: 72 glyphs (8 trigrams + 64 hexagrams), mono=true");
}
