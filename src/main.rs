use std::fs::File;

use clap::{value_t, App, Arg};
use failure::Error;
use raylib::prelude::*;
use raylib::{color::Color, math::Vector2};
use serde::{Deserialize, Serialize};

trait ColorLoad {
    fn load_colors(&mut self);
}

#[derive(Debug, Serialize, Deserialize)]
struct Wall {
    #[serde(rename = "color")]
    raw_color: (u8, u8, u8, u8),

    #[serde(skip)]
    pub color: Color,

    pub start: Vector2,
    pub end: Vector2,
}

impl ColorLoad for Wall {
    fn load_colors(&mut self) {
        self.color = self.raw_color.into();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Light {
    pub fixed: bool,

    #[serde(rename = "color")]
    raw_color: (u8, u8, u8, u8),

    #[serde(skip)]
    pub color: Color,

    pub position: Vector2,
}

impl ColorLoad for Light {
    fn load_colors(&mut self) {
        self.color = self.raw_color.into();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct World {
    pub walls: Vec<Wall>,
    pub light: Light,
}

impl ColorLoad for World {
    fn load_colors(&mut self) {
        for wall in self.walls.iter_mut() {
            wall.load_colors();
        }
        self.light.load_colors();
    }
}

impl World {
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let mut world: World = serde_json::from_reader(File::open(path)?)?;
        world.load_colors();
        Ok(world)
    }
}

fn find_intersect(
    wall_start: Vector2,
    wall_end: Vector2,
    ray_start: Vector2,
    ray_end: Vector2,
) -> Option<Vector2> {
    let a1 = wall_end.y - wall_start.y;
    let b1 = wall_start.x - wall_end.x;
    let c1 = a1 * wall_start.x + b1 * wall_start.y;

    let a2 = ray_end.y - ray_start.y;
    let b2 = ray_start.x - ray_end.x;
    let c2 = a2 * ray_start.x + b2 * ray_start.y;

    let delta = a1 * b2 - a2 * b1;

    if delta == 0.0 {
        return None;
    }

    Some(Vector2 {
        x: (b2 * c1 - b1 * c2) / delta,
        y: (a1 * c2 - a2 * c1) / delta,
    })
}

fn get_color_modifier_of_pixel(pixel: Vector2, world: &World) -> Color {
    // Search all walls
    for wall in world.walls.iter() {
        // Check for collision
        if find_intersect(
            wall.start,
            wall.end,
            pixel - Vector2::new(0.5, 0.5),
            pixel + Vector2::new(0.5, 0.5),
        )
        .is_some()
        {
            return wall.color;
        }
    }

    // No modifier
    return Color::BLACK;
}

fn plot(
    position: &Vector2,
    normal: Vector2,
    magnitude: f32,
    window_vec: &Vector2,
    ray_color: &Color,
    world: &World,
    d: &mut RaylibDrawHandle,
) -> Option<Color> {
    // Calculate the current pixel coord
    let pixel = (normal * magnitude) + *position;

    // We cannot plot outside the window
    if (pixel.x < 0.0 || pixel.x > window_vec.x) || (pixel.y < 0.0 || pixel.y > window_vec.y) {
        return None;
    }

    // Modify the light ray color
    let modifier = get_color_modifier_of_pixel(pixel, world);
    let ray_color = Color {
        r: ray_color.r - modifier.r,
        g: ray_color.g - modifier.g,
        b: ray_color.b - modifier.b,
        a: 255,
    };

    // Plot the ray
    d.draw_pixel_v(
        Vector2 {
            x: pixel.x * window_vec.x,
            y: pixel.y * window_vec.y,
        },
        ray_color,
    );

    // Iterate a step down the ray
    return Some(ray_color);
}

fn trace_and_plot(
    position: &Vector2,
    normal: Vector2,
    window_vec: &Vector2,
    ray_color: &Color,
    world: &World,
    d: &mut RaylibDrawHandle,
) {
    let mut magnitude = 0.0;
    let mut color = *ray_color;
    loop {
        let new_color = plot(position, normal, magnitude, window_vec, &color, world, d);
        magnitude += 1.0;

        // Handle edge of the screen
        if new_color.is_none() {
            return;
        }

        color = new_color.unwrap();
    }
}

fn main() {
    let matches = App::new("glasscast")
        .author("Evan Pratten <ewpratten@gmail.com>")
        .arg(
            Arg::with_name("world")
                .takes_value(true)
                .help("Path to the world JSON file")
                .required(true),
        )
        .get_matches();

    // Get data
    let world = matches.value_of("world").unwrap();

    // Parse the world
    let mut world = World::from_file(world).expect("Failed to read JSON file");

    // Configure a window
    let (mut rl, thread) = raylib::init()
        .size(1080, 720)
        .title("GlassCast")
        .msaa_4x()
        .vsync()
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        // Get the window size as a vector
        let window_vec = Vector2 {
            x: d.get_screen_width() as f32,
            y: d.get_screen_height() as f32,
        };

        // Handle light controls
        if !world.light.fixed {
            // Get the mouse vector
            let mouse_pos = d.get_mouse_position();

            // Normalize and set
            world.light.position = mouse_pos / window_vec;
        }

        // Render every ray extending from the light
        for angle in 0..180 {
            let angle = angle as f32;

            // Calculate the ray normal
            let normal = Vector2 {
                x: angle.to_radians().cos(),
                y: angle.to_radians().sin(),
            };

            // Recursive render
            trace_and_plot(
                &world.light.position,
                normal,
                &window_vec,
                &world.light.color,
                &world,
                &mut d,
            );
        }

        // Render FPS counter
        d.draw_fps(5, 5);
    }
}
