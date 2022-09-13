// Adapted from https://github.com/Rust-SDL2/rust-sdl2/blob/master/examples/ttf-demo.rs&

use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::ttf::FontError;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::Window;
use sdl2::pixels::Color;

const FONT_PATH: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

pub fn create_text_surface<'a>(ttf_context: &'a Sdl2TtfContext, text: &str, point_size: u16) -> Result<Surface<'a>, FontError> {
    let mut font = ttf_context.load_font(FONT_PATH, point_size).expect(&format!("Not found: {}", FONT_PATH));
    font.set_style(sdl2::ttf::FontStyle::NORMAL);
    return font
        .render(text)
        .blended(Color::RGBA(255, 0, 0, 255));
}

pub fn write_text(canvas: &mut Canvas<Window>, ttf_context: &Sdl2TtfContext, text: &str, x: u32, y: u32, point_size: u16) -> Result<(), String> {
    let text_surface = create_text_surface(ttf_context, text, point_size).unwrap();
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.create_texture_from_surface(&text_surface).unwrap();
    let sdl2::render::TextureQuery { width, height, .. } = texture.query();
    let position = rect!(x, y, width, height);
    return canvas.copy(&texture, None, Some(position));
}

pub fn write_multiline_text(canvas: &mut Canvas<Window>, ttf_context: &Sdl2TtfContext, text: &str, x: u32, y: u32, point_size: u16) -> Result<(), String> {
    let splits = text.split("\n").collect::<Vec<&str>>();
    for i in 0..splits.len() {
        let line_result = write_text(canvas, ttf_context, &splits[i], 0, i as u32 * point_size as u32, point_size);
        if line_result.is_err() {
            return line_result;
        }
    }
    Ok(())
}
