use clap::{App, Arg, value_t};

#[derive(Debug, Serialize, Deserialize)]
struct Wall {

}

#[derive(Debug, Serialize, Deserialize)]
struct Light {

}

#[derive(Debug, Serialize, Deserialize)]
struct World {
    walls: [Wall],
    light: Light
}

fn main() {
    let matches = App::new("glasscast")
    .author("Evan Pratten <ewpratten@gmail.com>")
    .arg(
     Arg::with_name("world")
        .takes_value(true)
        .help("Path to the world JSON file")
        .required(true)
    )
    .get_matches();

    // Get data
    let world = matches.value_of("world").unwrap();

    // Parse the world

    
}