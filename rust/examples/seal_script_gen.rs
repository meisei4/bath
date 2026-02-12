use image::{DynamicImage, GrayImage, Luma, Rgba, RgbaImage};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

const SEAL_CHARS_PATH: &str = "/Users/adduser/fu4seoi3/refs/seal_script/seal_chars.txt";
const RAW_SCANS_DIR: &str = "/Users/adduser/fu4seoi3/refs/seal_script/raw_jpgs";
const PRECLEANED_DIR: &str = "/Users/adduser/fu4seoi3/refs/seal_script/cleaned";
const PREVIEW_DIR: &str = "/Users/adduser/fu4seoi3/refs/seal_script/cleaned";
const OUTPUT_BASE: &str = "/Users/adduser/fu4seoi3/src/fu4seoi3/romdisk/assets";

const ATLAS_SIZES: &[u32] = &[32, 48];

const OTSU_FALLBACK_THRESHOLD: u8 = 128;
const SPECKLE_MIN_AREA: u32 = 8;
const GLYPH_PADDING: u32 = 1;

const SHINJITAI_TO_KYUUJITAI: &[(char, char)] = &[
    ('体', '體'),
    ('値', '價'),
    ('宝', '寶'),
    ('竜', '龍'),
    ('経', '經'),
    ('霊', '靈'),
    ('験', '驗'),
    ('総', '總'),
    ('緑', '綠'),
    ('黄', '黃'),
    ('黒', '黑'),
    ('気', '氣'),
    ('万', '萬'),
    ('帰', '歸'),
    ('予', '豫'),
    ('壮', '壯'),
    ('済', '濟'),
    ('斉', '齊'),
    ('声', '聲'),
    ('鉄', '鐵'),
    ('広', '廣'),
    ('弐', '貳'),
    ('圧', '壓'),
    ('団', '團'),
    ('仏', '佛'),
];

fn load_seal_chars(path: &Path) -> Vec<char> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ERROR: Cannot open {}: {}", path.display(), e);
            return Vec::new();
        },
    };

    let reader = BufReader::new(file);
    let mut chars = Vec::new();

    for line in reader.lines().flatten() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        for ch in trimmed.chars() {
            if !ch.is_whitespace() && !chars.contains(&ch) {
                chars.push(ch);
            }
        }
    }

    println!("Loaded {} unique seal characters", chars.len());
    chars
}

fn kyuujitai_fallback(ch: char) -> Option<char> {
    SHINJITAI_TO_KYUUJITAI
        .iter()
        .find(|(shin, _)| *shin == ch)
        .map(|(_, kyuu)| *kyuu)
}

fn find_glyph_image(raw_dir: &Path, codepoint: u32) -> Option<PathBuf> {
    let path = raw_dir.join(format!("U+{:04X}.jpg", codepoint));
    if path.exists() {
        return Some(path);
    }
    None
}

fn otsu_threshold(gray: &GrayImage) -> u8 {
    let mut histogram = [0u32; 256];
    for pixel in gray.pixels() {
        histogram[pixel[0] as usize] += 1;
    }

    let total_pixels = gray.width() * gray.height();
    if total_pixels == 0 {
        return OTSU_FALLBACK_THRESHOLD;
    }

    let mut sum_total: f64 = 0.0;
    for (i, &count) in histogram.iter().enumerate() {
        sum_total += i as f64 * count as f64;
    }

    let mut sum_bg: f64 = 0.0;
    let mut weight_bg: f64 = 0.0;
    let mut max_variance: f64 = 0.0;
    let mut best_threshold: u8 = 0;

    for t in 0..256u32 {
        weight_bg += histogram[t as usize] as f64;
        if weight_bg == 0.0 {
            continue;
        }

        let weight_fg = total_pixels as f64 - weight_bg;
        if weight_fg == 0.0 {
            break;
        }

        sum_bg += t as f64 * histogram[t as usize] as f64;
        let mean_bg = sum_bg / weight_bg;
        let mean_fg = (sum_total - sum_bg) / weight_fg;

        let between_variance = weight_bg * weight_fg * (mean_bg - mean_fg) * (mean_bg - mean_fg);
        if between_variance > max_variance {
            max_variance = between_variance;
            best_threshold = t as u8;
        }
    }

    best_threshold
}

fn clean_glyph(raw_image: &DynamicImage) -> GrayImage {
    let gray = raw_image.to_luma8();
    let threshold = otsu_threshold(&gray);

    let (w, h) = gray.dimensions();
    let mut binary = GrayImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let pixel = gray.get_pixel(x, y)[0];
            binary.put_pixel(x, y, Luma([if pixel < threshold { 255 } else { 0 }]));
        }
    }

    remove_speckles(&mut binary, SPECKLE_MIN_AREA);

    binary
}

fn remove_speckles(binary: &mut GrayImage, min_area: u32) {
    let (w, h) = binary.dimensions();
    let mut labels: Vec<i32> = vec![0; (w * h) as usize];
    let mut label_id = 0i32;
    let mut label_areas: Vec<u32> = vec![0];

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if binary.get_pixel(x, y)[0] == 255 && labels[idx] == 0 {
                label_id += 1;
                let mut area = 0u32;
                let mut stack = vec![(x, y)];

                while let Some((cx, cy)) = stack.pop() {
                    let cidx = (cy * w + cx) as usize;
                    if labels[cidx] != 0 {
                        continue;
                    }
                    if binary.get_pixel(cx, cy)[0] != 255 {
                        continue;
                    }
                    labels[cidx] = label_id;
                    area += 1;

                    if cx > 0 {
                        stack.push((cx - 1, cy));
                    }
                    if cx + 1 < w {
                        stack.push((cx + 1, cy));
                    }
                    if cy > 0 {
                        stack.push((cx, cy - 1));
                    }
                    if cy + 1 < h {
                        stack.push((cx, cy + 1));
                    }
                }

                label_areas.push(area);
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let label = labels[idx];
            if label > 0 && label_areas[label as usize] < min_area {
                binary.put_pixel(x, y, Luma([0]));
            }
        }
    }
}

fn crop_and_center(glyph: &GrayImage, cell_size: u32) -> RgbaImage {
    let (w, h) = glyph.dimensions();
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;

    for y in 0..h {
        for x in 0..w {
            if glyph.get_pixel(x, y)[0] > 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    if min_x > max_x || min_y > max_y {
        return RgbaImage::new(cell_size, cell_size);
    }

    let content_w = max_x - min_x + 1;
    let content_h = max_y - min_y + 1;

    let usable = cell_size - GLYPH_PADDING * 2;
    let scale = (usable as f64 / content_w.max(content_h) as f64).min(1.0);

    let scaled_w = (content_w as f64 * scale) as u32;
    let scaled_h = (content_h as f64 * scale) as u32;

    let cropped = image::imageops::crop_imm(glyph, min_x, min_y, content_w, content_h).to_image();

    let resized = image::imageops::resize(
        &cropped,
        scaled_w.max(1),
        scaled_h.max(1),
        image::imageops::FilterType::Lanczos3,
    );

    let mut output = RgbaImage::new(cell_size, cell_size);
    let off_x = (cell_size - scaled_w) / 2;
    let off_y = (cell_size - scaled_h) / 2;

    for y in 0..scaled_h.min(cell_size) {
        for x in 0..scaled_w.min(cell_size) {
            let val = resized.get_pixel(x, y)[0];
            let dx = off_x + x;
            let dy = off_y + y;
            if dx < cell_size && dy < cell_size {
                output.put_pixel(dx, dy, Rgba([255, 255, 254, val]));
            }
        }
    }

    output
}

fn next_power_of_two(n: u32) -> u32 {
    let mut p = 1u32;
    while p < n {
        p *= 2;
    }
    p
}

struct PackedGlyph {
    codepoint: u32,
    atlas_x: u32,
    atlas_y: u32,
    width: u32,
    height: u32,
}

fn pack_atlas(glyphs: &[(char, RgbaImage)], cell_size: u32) -> (RgbaImage, Vec<PackedGlyph>) {
    let glyph_count = glyphs.len() as u32;

    let mut best_cols = ((glyph_count as f64).sqrt().ceil() as u32).max(1);
    let mut best_area = u64::MAX;
    for try_cols in 1..=glyph_count {
        let try_rows = (glyph_count + try_cols - 1) / try_cols;
        let w = next_power_of_two(try_cols * cell_size);
        let h = next_power_of_two(try_rows * cell_size);
        if w > 1024 || h > 1024 {
            continue;
        }
        let area = w as u64 * h as u64;
        if area < best_area
            || (area == best_area
                && (w as i64 - h as i64).unsigned_abs()
                    < (next_power_of_two(best_cols * cell_size) as i64
                        - next_power_of_two(((glyph_count + best_cols - 1) / best_cols) * cell_size) as i64)
                        .unsigned_abs())
        {
            best_area = area;
            best_cols = try_cols;
        }
    }

    let cols = best_cols;
    let rows = (glyph_count + cols - 1) / cols;

    let content_w = cols * cell_size;
    let content_h = rows * cell_size;
    let atlas_w = next_power_of_two(content_w);
    let atlas_h = next_power_of_two(content_h);

    let mut atlas = RgbaImage::new(atlas_w, atlas_h);
    let mut packed = Vec::new();

    for (i, (ch, glyph_img)) in glyphs.iter().enumerate() {
        let col = i as u32 % cols;
        let row = i as u32 / cols;
        let x = col * cell_size;
        let y = row * cell_size;

        image::imageops::overlay(&mut atlas, glyph_img, x as i64, y as i64);

        packed.push(PackedGlyph {
            codepoint: *ch as u32,
            atlas_x: x,
            atlas_y: y,
            width: cell_size,
            height: cell_size,
        });
    }

    let waste = if atlas_w * atlas_h > 0 {
        100 - (content_w * content_h * 100) / (atlas_w * atlas_h)
    } else {
        0
    };

    println!(
        "  Atlas: {}x{} content, {}x{} POT, {} glyphs, {}% waste",
        content_w, content_h, atlas_w, atlas_h, glyph_count, waste
    );

    (atlas, packed)
}

fn write_bmfont(path: &Path, png_filename: &str, cell_size: u32, atlas_w: u32, atlas_h: u32, glyphs: &[PackedGlyph]) {
    let mut fnt = File::create(path).unwrap();
    writeln!(fnt, "info face=\"seal_script\" size={} bold=0 italic=0", cell_size).unwrap();
    writeln!(
        fnt,
        "common lineHeight={} base={} scaleW={} scaleH={} pages=1",
        cell_size, cell_size, atlas_w, atlas_h
    )
    .unwrap();
    writeln!(fnt, "page id=0 file=\"{}\"", png_filename).unwrap();
    writeln!(fnt, "chars count={}", glyphs.len()).unwrap();

    for g in glyphs {
        writeln!(
            fnt,
            "char id={} x={} y={} width={} height={} xoffset=0 yoffset=0 xadvance={} page=0",
            g.codepoint, g.atlas_x, g.atlas_y, g.width, g.height, g.width
        )
        .unwrap();
    }
}

fn scan_dataset(raw_dir: &Path) -> HashMap<u32, PathBuf> {
    let mut mapping: HashMap<u32, PathBuf> = HashMap::new();

    if !raw_dir.is_dir() {
        eprintln!("ERROR: raw_jpgs directory not found at {}", raw_dir.display());
        return mapping;
    }

    if let Ok(entries) = fs::read_dir(raw_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("U+") || !name.ends_with(".jpg") {
                continue;
            }
            let hex_str = &name[2..name.len() - 4];
            if let Ok(codepoint) = u32::from_str_radix(hex_str, 16) {
                mapping.insert(codepoint, entry.path());
            }
        }
    }

    println!("Dataset index: {} characters with images", mapping.len());
    mapping
}

fn main() {
    println!("\n=== Seal Script Font Generator ===");
    println!("Pipeline: CODH TE00039 scans -> clean -> pack atlas -> BMFont\n");

    let chars = load_seal_chars(Path::new(SEAL_CHARS_PATH));
    if chars.is_empty() {
        eprintln!("ERROR: No characters loaded from {}", SEAL_CHARS_PATH);
        return;
    }

    let raw_dir = PathBuf::from(RAW_SCANS_DIR);
    let preview_dir = PathBuf::from(PREVIEW_DIR);

    if !raw_dir.exists() {
        eprintln!("ERROR: TE00039 dataset not found at {}", raw_dir.display());
        eprintln!("Download: http://codh.rois.ac.jp/tensho/dataset/v2/TE00039.zip");
        eprintln!("Extract:  unzip TE00039.zip -d {}", raw_dir.parent().unwrap().display());
        return;
    }

    let precleaned_dir = PathBuf::from(PRECLEANED_DIR);
    let have_precleaned = precleaned_dir.is_dir();

    if have_precleaned {
        println!("Using pre-cleaned PNGs from {} ...", precleaned_dir.display());
    } else {
        println!("No pre-cleaned PNGs found, using raw scans with built-in cleaning ...");
    }

    println!("Scanning TE00039 dataset at {} ...", raw_dir.display());
    let image_map = scan_dataset(&raw_dir);

    let mut processed_glyphs: Vec<(char, GrayImage)> = Vec::new();
    let mut missing: Vec<char> = Vec::new();
    let mut fallback_used: Vec<(char, char)> = Vec::new();
    let mut precleaned_count = 0u32;

    for &ch in &chars {
        let codepoint = ch as u32;
        let unicode_str = format!("U+{:04X}", codepoint);

        if have_precleaned {
            let pc_path = precleaned_dir.join(format!("{}.png", unicode_str));
            if pc_path.exists() {
                match image::open(&pc_path) {
                    Ok(img) => {
                        let gray = img.to_luma8();
                        let ink_pixels: u32 = gray.pixels().map(|p| if p[0] > 0 { 1u32 } else { 0u32 }).sum();
                        let total_pixels = gray.width() * gray.height();
                        let ink_ratio = if total_pixels > 0 {
                            ink_pixels * 100 / total_pixels
                        } else {
                            0
                        };
                        println!(
                            "  {} '{}' -> {}x{}, {}% ink [pre-cleaned]",
                            unicode_str,
                            ch,
                            gray.width(),
                            gray.height(),
                            ink_ratio
                        );
                        processed_glyphs.push((ch, gray));
                        precleaned_count += 1;
                        continue;
                    },
                    Err(e) => {
                        eprintln!("  {} '{}' pre-cleaned load error: {}", unicode_str, ch, e);
                    },
                }
            }
        }

        let mut image_path = image_map
            .get(&codepoint)
            .cloned()
            .or_else(|| find_glyph_image(&raw_dir, codepoint));

        if image_path.is_none() {
            if let Some(kyuu) = kyuujitai_fallback(ch) {
                let kyuu_cp = kyuu as u32;
                image_path = image_map
                    .get(&kyuu_cp)
                    .cloned()
                    .or_else(|| find_glyph_image(&raw_dir, kyuu_cp));
                if image_path.is_some() {
                    fallback_used.push((ch, kyuu));
                }
            }
        }

        match image_path {
            Some(path) => match image::open(&path) {
                Ok(img) => {
                    let cleaned = clean_glyph(&img);

                    let out_path = preview_dir.join(format!("{}.png", unicode_str));
                    cleaned.save(&out_path).ok();

                    let ink_pixels: u32 = cleaned.pixels().map(|p| if p[0] > 0 { 1u32 } else { 0u32 }).sum();
                    let total_pixels = cleaned.width() * cleaned.height();
                    let ink_ratio = if total_pixels > 0 {
                        ink_pixels * 100 / total_pixels
                    } else {
                        0
                    };

                    println!(
                        "  {} '{}' -> {}x{}, {}% ink [raw+clean]",
                        unicode_str,
                        ch,
                        cleaned.width(),
                        cleaned.height(),
                        ink_ratio
                    );
                    processed_glyphs.push((ch, cleaned));
                },
                Err(e) => {
                    eprintln!("  {} '{}' ERROR loading: {}", unicode_str, ch, e);
                    missing.push(ch);
                },
            },
            None => {
                missing.push(ch);
            },
        }
    }

    if precleaned_count > 0 {
        println!("\n  {} glyphs loaded from pre-cleaned PNGs", precleaned_count);
    }

    println!(
        "\n=== Results: {} found, {} missing ===",
        processed_glyphs.len(),
        missing.len()
    );

    if !fallback_used.is_empty() {
        println!("\nShinjitai -> Kyuujitai fallbacks used:");
        for (shin, kyuu) in &fallback_used {
            println!(
                "  {} (U+{:04X}) -> {} (U+{:04X})",
                shin, *shin as u32, kyuu, *kyuu as u32
            );
        }
    }

    if !missing.is_empty() {
        println!("\nMissing characters (not in Shuowen Jiezi):");
        for ch in &missing {
            println!("  U+{:04X} '{}'", *ch as u32, ch);
        }
    }

    if processed_glyphs.is_empty() {
        eprintln!("\nNo glyphs to pack!");
        return;
    }

    println!();
    for &cell_size in ATLAS_SIZES {
        println!("Generating seal_script_{}px ...", cell_size);

        let sized_glyphs: Vec<(char, RgbaImage)> = processed_glyphs
            .iter()
            .map(|(ch, gray)| (*ch, crop_and_center(gray, cell_size)))
            .collect();

        let (atlas, packed) = pack_atlas(&sized_glyphs, cell_size);
        let (atlas_w, atlas_h) = atlas.dimensions();

        let png_filename = format!("seal_script_{}px.png", cell_size);
        let fnt_filename = format!("seal_script_{}px.fnt", cell_size);

        let png_path = Path::new(OUTPUT_BASE).join(&png_filename);
        let fnt_path = Path::new(OUTPUT_BASE).join(&fnt_filename);

        atlas.save(&png_path).unwrap();
        write_bmfont(&fnt_path, &png_filename, cell_size, atlas_w, atlas_h, &packed);

        println!(
            "    -> {} ({}x{}, {} glyphs)",
            png_path.display(),
            atlas_w,
            atlas_h,
            packed.len()
        );
    }

    let preview_path = preview_dir.join("_preview_grid.png");
    let preview_cell = 64u32;
    let preview_cols = 16u32;
    let preview_rows = ((processed_glyphs.len() as u32) + preview_cols - 1) / preview_cols;
    let mut preview = RgbaImage::new(preview_cols * preview_cell, preview_rows * preview_cell);

    for (i, (_, gray)) in processed_glyphs.iter().enumerate() {
        let col = i as u32 % preview_cols;
        let row = i as u32 / preview_cols;
        let cell_img = crop_and_center(gray, preview_cell);
        image::imageops::overlay(
            &mut preview,
            &cell_img,
            (col * preview_cell) as i64,
            (row * preview_cell) as i64,
        );
    }
    preview.save(&preview_path).ok();
    println!("\nPreview grid: {}", preview_path.display());

    println!("\n=== Done ===");
}
