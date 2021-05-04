use std::fs::File;

use clap::{App, Arg};
use failure::Error;
use geo::algorithm::euclidean_distance::EuclideanDistance;
use geo::{Line, Point};
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

    start: Vector2,
    end: Vector2,

    #[serde(skip)]
    pub line: Option<Line<f32>>,
}

impl Wall {
    fn load_line(&mut self) {
        self.line = Some(Line::new(
            Point::new(self.start.x, self.start.y),
            Point::new(self.end.x, self.end.y),
        ));
    }
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
            wall.load_line();
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

fn find_intersect(wall: &Wall, point: Vector2) -> bool {
    // Define the line
    return wall
        .line
        .unwrap()
        .euclidean_distance(&Point::new(point.x, point.y))
        < 1.0;
}

fn get_color_modifier_of_pixel(pixel: Vector2, world: &World) -> Color {
    // Search all walls
    for wall in world.walls.iter() {
        // Check for collision
        if find_intersect(&wall, pixel) {
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
    let pixel = (normal * magnitude) + (*position * *window_vec);

    // We cannot plot outside the window
    if (pixel.x < 0.0 || pixel.x > window_vec.x) || (pixel.y < 0.0 || pixel.y > window_vec.y) {
        return None;
    }

    // Modify the light ray color
    let modifier = get_color_modifier_of_pixel(pixel, world);
    let ray_color = Color {
        r: (ray_color.r as f32 - modifier.r as f32).clamp(u8::MIN as f32, u8::MAX as f32) as u8,
        g: (ray_color.g as f32 - modifier.g as f32).clamp(u8::MIN as f32, u8::MAX as f32) as u8,
        b: (ray_color.b as f32 - modifier.b as f32).clamp(u8::MIN as f32, u8::MAX as f32) as u8,
        a: 255,
    };

    // Plot the ray
    d.draw_pixel_v(
        Vector2 {
            x: pixel.x,
            y: pixel.y,
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
    ray_color: Color,
    world: &World,
    d: &mut RaylibDrawHandle,
) {
    let mut magnitude = 0.0;
    let mut color = ray_color;
    loop {
        let new_color = plot(position, normal, magnitude, window_vec, &color, world, d);
        magnitude += 2.0;

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
        .size(800, 600)
        .title("GlassCast")
        // .msaa_4x()
        .vsync()
        .build();

    // Load bloom shader
    let bloom_shader = rl.load_shader(&thread, None, Some("./bloom.fs")).unwrap();
    let bloom_surface = rl.load_render_texture(&thread, 800, 600).unwrap();

    // Last light position
    let mut last_light_position = Vector2::new(-1.0, -1.0);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

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

        // Open a shader context
        // Skip rendering if the light didn't move
        if world.light.position != last_light_position {
            unsafe {
                raylib::ffi::BeginTextureMode(*bloom_surface);
            }
            d.clear_background(Color::WHITE);

            // Render every ray extending from the light
            for angle in 0..360 {
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
                    world.light.color,
                    &world,
                    &mut d,
                );
            }

            unsafe {
                raylib::ffi::EndTextureMode();
            }
        }
        last_light_position = world.light.position;

        // Render via the shader
        {
            let mut shader_context = d.begin_shader_mode(&bloom_shader);

            // Blit the texture
            shader_context.draw_texture_rec(
                &bloom_surface,
                Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: bloom_surface.width() as f32,
                    height: (bloom_surface.height() as f32) * -1.0,
                },
                Vector2::zero(),
                Color::WHITE,
            );
        }

        // Render FPS counter
        d.draw_fps(5, 5);
    }
}
