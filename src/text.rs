use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::Window;
use sdl2::pixels::Color;

const FONT_PATH: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

fn get_centered_rect(screen_width: u32, screen_height: u32, rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (screen_width as i32 - w) / 2;
    let cy = (screen_height as i32 - h) / 2;
    rect!(cx, cy, w, h)
}

fn get_text_surface(canvas: &mut Canvas<Window>, ttf_context: &mut Sdl2TtfContext, screen_width: u32, screen_height: u32) -> Option<Rect> {
    let mut font = ttf_context.load_font(FONT_PATH, 128).ok()?;
    font.set_style(sdl2::ttf::FontStyle::NORMAL);
    
    let surface = font
        .render("Hello Rust!")
        .blended(Color::RGBA(255, 0, 0, 255))
        .map_err(|e| e.to_string()).ok()?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string()).ok()?;

    let sdl2::render::TextureQuery { width, height, .. } = texture.query();

    let padding = 64;
    let target = get_centered_rect(
        screen_width,
        screen_height,
        width,
        height,
        screen_width - padding,
        screen_height - padding,
    );

    return Some(target);
}
