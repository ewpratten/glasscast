use std::fs::File;

use clap::{value_t, App, Arg};
use failure::Error;
use line_intersection::LineInterval;
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

fn get_color_modifier_of_pixel(pixel: Vector2, world: &World) -> Color {

    // Search all walls
    for wall in world.walls.iter() {

        // Define the line segment of the wall
        let segment = LineInterval::line_segment(Line {
            start: (wall.start.x, wall.start.y).into(),
            end: (wall.end.x, wall.end.y).into(),
        });

    }

    // No modifier
    return Color::BLACK;

}

fn trace_and_plot(
    normal: Vector2,
    magnitude: f32,
    window_vec: &Vector2,
    ray_color: &mut Color,
    world: &World,
    d: &mut RaylibDrawHandle,
) {
    // Calculate the current pixel coord
    let pixel = normal * magnitude;
    let pixel = Vector2 {
        x: pixel.x.floor(),
        y: pixel.y.floor()
    };

    // We cannot plot outside the window
    if (pixel.x < 0.0 || pixel.x > window_vec.x) || (pixel.y < 0.0 || pixel.y > window_vec.y) {
        return;
    }

    // Modify the light ray color

    // Plot the ray
    d.draw_pixel_v(pixel, *ray_color);

    // Iterate a step down the ray
    trace_and_plot(normal, magnitude + 1.0, window_vec, ray_color, world, d);

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

        // Render FPS counter
        d.draw_fps(5, 5);
    }
}
