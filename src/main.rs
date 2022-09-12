extern crate sdl2;
extern crate num;
extern crate lyon_geom;

use std::collections::HashMap;

use lyon_geom::Scalar;
use lyon_geom::euclid::Point2D;
use lyon_geom::{QuadraticBezierSegment, Point};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels;

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const DRAW_COLOR: Color = Color::WHITE;
const CONSTRUCT_COLOR: Color = Color::RGBA(0x77u8, 0x77u8, 0x77u8, 0x22u8);

fn sdl_point(input: lyon_geom::Point<f64>) -> sdl2::rect::Point {
    return sdl2::rect::Point::new(input.x as i32, input.y as i32);
}

fn sample_point(segment: &QuadraticBezierSegment<f64>, t: f64) -> sdl2::rect::Point {
    let sample = segment.sample(t);
    return sdl_point(sample);
}

fn draw_segment(canvas: &mut Canvas<Window>, segment: &QuadraticBezierSegment<f64>) {
    let color = canvas.draw_color();
    canvas.set_draw_color(DRAW_COLOR);
    let subdivisions = (segment.length() / 20f64) as usize;
    let step = 1f64 / (subdivisions as f64);
    for i in 1..subdivisions + 1 {
        let t0 = step * ((i - 1) as f64);
        let t1 = step * (i as f64);
        let p0 = sample_point(segment, t0);
        let p1 = sample_point(segment, t1);
        canvas.draw_line(p0, p1);
    }
    canvas.set_draw_color(color);
}

fn draw_snap<S: Scalar>(canvas: &mut Canvas<Window>, snap: Snap<S>, size: i32) -> Result<(), String> {
    let color = canvas.draw_color();
    canvas.set_draw_color(Color::GREEN);
    let result = canvas.draw_rect(rect_from_point(&snap.point, size));
    canvas.set_draw_color(color);
    return result;
}

fn rect_from_point<S: Scalar>(p: &Point<S>, size: i32) -> sdl2::rect::Rect {
    let x = p.x.to_i32().unwrap() - size/2;
    let y = p.y.to_i32().unwrap() - size/2;
    return sdl2::rect::Rect::new(x, y, size as u32, size as u32);
}

#[derive(Copy, Clone)]
struct Snap<S> {
    pub geometry_index: usize,
    pub t: S,
    pub point: Point<S>
}

struct GeometryContainer<S> {
    snap_tolerance: S,

    geometries: HashMap<usize, Geometry<S>>,
    index: usize,
    active: Option<usize>
}

impl<S> GeometryContainer<S> {
    pub fn focus(&mut self, p: Snap<S>) -> Option<usize> {
        self.active = Some(p.geometry_index);
        return self.active;
    }

    pub fn unfocus(&mut self) {
        self.active = None;
    }
}

impl<S: Scalar> GeometryContainer<S> {

    pub fn new(snap_tolerance: S) -> Self {
        return Self {
            snap_tolerance: snap_tolerance,
            geometries: HashMap::new(),
            index: 0,
            active: None
        };
    }

    pub fn snap(&self, x: S, y: S) -> Option<Snap<S>> {
        for (index, geometry) in &self.geometries {
            match self.active {
                Some(active_idx) => if *index == active_idx {
                    continue;
                }
                _ => ()
            }
            let p = Point2D::new(x, y);
            match geometry.snap_t(p, self.snap_tolerance) {
                Some(t) => {
                    return Some(Snap { 
                        geometry_index: index.clone(),
                        t: t,
                        point: geometry.sample(t)
                    });
                },
                _ => ()
            }
        }
        return None;
    }

    pub fn snap_split(&mut self, p: Snap<S>) -> Result<(), String> {
        self.map_replace(p.geometry_index, |g| { g.split(p.t) })
    }

    pub fn map_replace(&mut self, index: usize, replace_fn: impl Fn(&Geometry<S>) -> [Geometry<S>; 2]) -> Result<(), String> {
        match self.geometries.get(&index) {
            Some(geometry) => {
                for new_geometry in replace_fn(geometry) {
                    self.insert(new_geometry);
                }
                self.geometries.remove(&index);
                return Ok(());
            }
            _ => Err(format!("Cannot replace geometry. No such index: {}", index))
        }
    }

    pub fn insert(&mut self, geometry: Geometry<S>) -> usize {
        self.geometries.insert(self.index, geometry);
        self.index += 1;
        return self.index - 1;
    }

    pub fn emplace(&mut self, x: S, y: S) {
        let geometry = Geometry::new(x, y);
        self.active = Some(self.insert(geometry));
    }

    pub fn apply(&mut self, x: S, y: S) -> Result<(), String> {
        match self.active {
            None => Ok(match self.snap(x, y) {
                Some(snap) => self.emplace(snap.point.x, snap.point.y),
                _ => self.emplace(x, y)
            }),
            Some(idx) => match self.geometries.get_mut(&idx) {
                Some(geometry) => {
                    geometry.shift();
                    if geometry.finalized() {
                        self.active = None;
                    }
                    Ok(())
                }
                _ => Err(format!("Cannot update geometry. No such index: {}", idx))
            }
        }
    }

    pub fn cancel(&mut self, x: S, y: S) -> Result<(), String> {
        match self.active {
            Some(idx) => match self.geometries.get_mut(&idx) {
                Some(geometry) => {
                    geometry.finalize();
                    self.active = None;
                    Ok(())
                },
                _ => Err(format!("Cannot seal geometry. No such index: {}", idx))
            }
            _ => Ok(())
        }
    }

    fn update_with_snap(snap: Option<Snap<S>>, geometry: &mut Geometry<S>, x: S, y: S) {
        match snap {
            Some(snap) => geometry.update(snap.point.x, snap.point.y),
            _ => geometry.update(x, y)
        }
    }

    pub fn update(&mut self, x: S, y: S) -> Result<(), String> {
        let snap = self.snap(x, y);
        match self.active {
            Some(idx) => match self.geometries.get_mut(&idx) {
                Some(geometry) => {
                    Self::update_with_snap(snap, geometry, x, y);
                    Ok(())
                },
                _ => Err(format!("Cannot update geometry. No such index: {}", idx))
            }
            _ => Ok(())
        }
    }

    pub fn foreach(&self, mut f: impl FnMut (&Geometry<S>) -> ()) {
        for (_, geometry) in &self.geometries {
            f(&geometry);
        }
    }

}

struct Way<S> {
    pub segment: QuadraticBezierSegment<S>
}

impl<S: Scalar> Way<S> {

    pub fn new(segment: QuadraticBezierSegment<S>) -> Self {
        return Self {
            segment: segment
        }
    }
}

struct Geometry<S> {
    way: Way<S>,
    construction_point: usize
}

impl<S: Scalar> Geometry<S> {

    const CONSTRUCTION_POINTS: usize = 3;

    fn new(x: S, y: S) -> Self {
        let point = lyon_geom::point(x, y);
        let segment = QuadraticBezierSegment {
            from: point.clone(),
            to: point.clone(),
            ctrl: point.clone()
        };
        let way = Way::new(segment);
        return Self {
            way: way,
            construction_point: 1
        }
    }

    fn new_split_segment(split_segment: QuadraticBezierSegment<S>) -> Self {
        return Self { way: Way::new(split_segment), construction_point: Self::CONSTRUCTION_POINTS };
    }

    pub fn split(&self, t: S) -> [Self; 2] {
        let splits = self.way.segment.split(t);
        return [Self::new_split_segment(splits.0), Self::new_split_segment(splits.1)];
    }

    fn update_at(&mut self, at: usize, x: S, y: S) {
        let mut segment = &mut self.way.segment;
        let mut point = match at {
            0 => &mut segment.from,
            1 => &mut segment.to,
            _ => &mut segment.ctrl
        };
        point.x = x;
        point.y = y;
    }

    pub fn snap_t(&self, p: Point<S>, tolerance: S) -> Option<S>{
        if self.way.segment.distance_to_point(p) <= tolerance {
            let closest_point = self.way.segment.closest_point(p);
            return Some(closest_point);
        }
        return None;
    }

    fn update(&mut self, x: S, y: S) {
        for i in self.construction_point..Self::CONSTRUCTION_POINTS {
            self.update_at(i, x, y);
        }
    }

    fn shift(&mut self) -> usize {
        if self.construction_point < Self::CONSTRUCTION_POINTS {
            self.construction_point += 1;
        }
        return self.construction_point;
    }

    pub fn finalized(&self) -> bool {
        return self.construction_point == Self::CONSTRUCTION_POINTS;
    }

    fn finalize(&mut self) {
        self.construction_point = Self::CONSTRUCTION_POINTS;
    }

    fn sample(&self, t: S) -> Point<S> {
        return self.way.segment.sample(t);
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let window = video_subsys
        .window(
            "rust-sdl2_gfx",
            SCREEN_WIDTH, 
            SCREEN_HEIGHT,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    const SNAP_TOLERANCE: f64 = 15.0f64;

    let mut events = sdl_context.event_pump()?;
    let mut seg_ctr: usize = 0;
    let mut container = GeometryContainer::new(15f64);
    let mut active: Option<usize> = None;
    let mut fx = 0f64;
    let mut fy = 0f64;
    let mut snap: Option<Snap<f64>> = None;

    'main: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Escape => {
                        break 'main;
                    }
                    _ => ()
                }

                Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                    match mouse_btn {
                        MouseButton::Left => {
                            container.apply(fx, fy);
                        }
                        MouseButton::Right => {
                            container.cancel(fx, fy);
                        }
                        _ => {}
                    }
                }

                Event::MouseMotion { x, y, .. } => {
                    fx = x as f64;
                    fy = y as f64;
                    container.update(fx, fy);
                }

                _ => {}
            }
        }
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        container.foreach(|g| { 
            draw_segment(&mut canvas, &g.way.segment) 
        });
        match container.snap(fx, fy) {
            Some(snap) => { 
            draw_snap(&mut canvas, snap, container.snap_tolerance as i32);
            },
            _ => ()
        };
        canvas.present();
    }

    Ok(())
}