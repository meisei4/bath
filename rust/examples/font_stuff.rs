use asset_payload::FONT_PATH;
use raylib::init;
use raylib::prelude::RaylibFont;
use ttf_parser::{name_id, Face};

fn main() {
    let (mut handle, thread) = init()
        .size(100, 100)
        .title("raylib [core] example - fixed function didactic")
        .build();
    let font = handle
        // .load_font(&thread, FONT_PATH)
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
