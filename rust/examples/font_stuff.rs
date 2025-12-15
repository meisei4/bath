use asset_payload::{FONT_IMAGE_PATH, FONT_PATH};
use raylib::init;
use raylib::prelude::*;
use std::fs::File;
use std::io::Write;
use ttf_parser::{name_id, Face};

fn main() {
    let (mut handle, thread) = init().size(100, 100).title("raylib font exporter").build();

    let sizes = [8, 10, 12, 16, 20, 24, 32];
    let chars: String = (32u8..127u8).map(|c| c as char).collect();

    for &size in &sizes {
        let font = handle
            .load_font_ex(&thread, FONT_PATH, size, Some(&chars))
            .expect("Failed to load font");

        let base_path = format!("/home/adduser/fu4seoi3/src/fu4seoi3/romdisk/assets/font_{}px", size);
        let mut image = font.texture().load_image().unwrap();

        image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);

        let tex_width = image.width;
        let tex_height = image.height;

        image.export_image(&format!("{}.png", base_path));

        let mut fnt_file = File::create(format!("{}.fnt", base_path)).unwrap();
        writeln!(fnt_file, "info face=\"font\" size={} bold=0 italic=0", size).unwrap();
        writeln!(
            fnt_file,
            "common lineHeight={} base={} scaleW={} scaleH={} pages=1",
            size, size, tex_width, tex_height
        )
        .unwrap();
        writeln!(fnt_file, "page id=0 file=\"font_{}px.png\"", size).unwrap();
        writeln!(fnt_file, "chars count={}", font.glyphCount).unwrap();

        for (i, glyph) in font.chars().iter().enumerate() {
            let rec = unsafe { *font.recs.add(i) };
            writeln!(
                fnt_file,
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
            "Exported: {}.png ({}x{}) and {}.fnt",
            base_path, tex_width, tex_height, base_path
        );
    }

    // let sizes = [8, 10, 12];
    // let chars: String = (32u8..127u8).map(|c| c as char).collect();
    // for &size in &sizes {
    //     let font = handle
    //         .load_font_ex(&thread, FONT_PATH, size, Some(&chars))
    //         .expect("Failed to load font");
    //     let base_path = format!("/home/adduser/fu4seoi3/src/fu4seoi3/romdisk/assets/font_{}px", size);
    //     let image = font.texture().load_image().unwrap();
    //     image.export_image(&format!("{}.png", base_path));
    //     println!("Exported: {}.png", base_path);
    // }

    println!("\nFont metadata for size 32:");
    let font = handle
        .load_font_ex(&thread, FONT_PATH, 32, None)
        .expect("Failed to load font");
    println!("File: {}", FONT_PATH);
    println!("Valid: {}", font.is_font_valid());
    println!("Base size: {} px", font.base_size());
    println!("Glyph count: {}", font.glyphCount);
    println!("Glyph padding: {}", font.glyphPadding);
    println!(
        "Texture: id={} size={}x{} mipmaps={} format={:?}",
        font.texture().id,
        font.texture().width,
        font.texture().height,
        font.texture().mipmaps,
        font.texture().format
    );
    println!("Texture pointer: {:p}", font.texture());
    println!("Recs pointer: {:p}", font.recs);
    println!("Glyphs pointer: {:p}", font.glyphs);
    println!();
    for (i, glyph) in font.chars().iter().take(5).enumerate() {
        println!(
            "[{:3}] codepoint '{}' ({}), adv={}, offs=({},{}), img={}x{}",
            i,
            std::char::from_u32(glyph.value as u32).unwrap_or('?'),
            glyph.value,
            glyph.advanceX,
            glyph.offsetX,
            glyph.offsetY,
            glyph.image.width,
            glyph.image.height
        );
    }

    let data = std::fs::read(FONT_PATH).expect("read font file");
    let face = Face::parse(&data, 0).expect("parse TTF");
    let family_name = face
        .names()
        .into_iter()
        .find(|n| n.name_id == name_id::FULL_NAME)
        .and_then(|n| n.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    println!("Full name:     {}", family_name);
    println!("Units per EM:  {}", face.units_per_em());
    println!("Glyphs:        {}", face.number_of_glyphs());
    println!("Asc/Desc:      {}/{}", face.ascender(), face.descender());
    println!("Line gap:      {}", face.line_gap());
    println!("Height:        {}", face.height());
    println!("Monospaced:    {}", face.is_monospaced());
    println!("Variable font: {}", face.is_variable());
    println!("Weight:        {:?}", face.weight());
    println!("Width:         {:?}", face.width());
    println!("Italic angle:  {}", face.italic_angle());
    println!("Bounding box: {:?}", face.global_bounding_box());
    println!("Has kerning table: {}", face.tables().kern.is_some());
    println!("Has cmap (character map): {}", face.tables().cmap.is_some());
    println!("Has glyf (outline): {}", face.tables().glyf.is_some());
    if let Some(ul) = face.underline_metrics() {
        println!("Underline:     pos={} thickness={}", ul.position, ul.thickness);
    }
    if let Some(st) = face.strikeout_metrics() {
        println!("Strikeout:     pos={} thickness={}", st.position, st.thickness);
    }
}
