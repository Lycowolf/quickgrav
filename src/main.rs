#[windows_subsystem = "windows"]
extern crate quicksilver;

use quicksilver::prelude::*;
use std::ops::Index;
use quicksilver::graphics::View;

const WIDTH: f32 = 1200.0;
const HEIGHT: f32 = 900.0;


#[derive(Clone, Copy, Debug)]
struct Planet {
    position: Vector,
    velocity: Vector,
    // per tick
    mass: f32,
    color: Color,
}

struct Space {
    planets: Vec<Planet>,
    paused: bool,
    time_step: f32,
    font: Asset<Font>,
    status_text_img: Option<Image>, // Text rendering is kind of slow, we cache it here
}

impl State for Space {
    fn new() -> Result<Space> {
        let mut planets: Vec<Planet> = Vec::new();

        let planet = Planet {
            position: Vector::new(0, 0),
            velocity: Vector::new(0, 0),
            mass: 200.0,
            color: Color::RED,
        };
        planets.push(planet);

        let planet = Planet {
            position: Vector::new(100, 0),
            velocity: Vector::new(0, 1.3),
            mass: 5.0,
            color: Color::GREEN,
        };
        planets.push(planet);

        let planet = Planet {
            position: Vector::new(200, 0),
            velocity: Vector::new(0, 1.1),
            mass: 2.0,
            color: Color::BLUE,
        };
        planets.push(planet);

        let planet = Planet {
            position: Vector::new(300, 0),
            velocity: Vector::new(0, 0.9),
            mass: 2.0,
            color: Color::CYAN,
        };
        planets.push(planet);

        let font = Asset::new(Font::load("FiraCode-Medium.ttf"));

        Ok(Space {
            planets,
            paused: true,
            time_step: 0.02,
            font,
            status_text_img: Option::None,
        }
        )
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        let mut status_text_changed = false;

        self.paused = match window.keyboard()[Key::Space] {
            ButtonState::Pressed => {
                status_text_changed = true;
                !self.paused
            }
            _ => self.paused,
        };

        self.time_step = match window.keyboard()[Key::Add] {
            ButtonState::Pressed => {
                status_text_changed = true;
                self.time_step * 2.0
            }
            _ => self.time_step,
        };

        self.time_step = match window.keyboard()[Key::Subtract] {
            ButtonState::Pressed => {
                status_text_changed = true;
                self.time_step / 2.0
            }
            _ => self.time_step,
        };

        if !self.paused {
            self.planets = integrate(self.time_step, &self.planets);
        }

        if status_text_changed || self.status_text_img.is_none() {
            let paused = self.paused;
            let style = FontStyle::new(16.0, Color::WHITE);
            let text = format!("Controls: <+/-> change timestep, <space> pause\nPaused: {}\nTime step: {}", paused, self.time_step);
            let mut img: Option<Image> = None;
            self.font.execute(|font| {
                match font.render(&text, &style) {
                    Ok(image) => {
                        img = Some(image);
                        Ok(())
                    }
                    Err(error) => panic!(format!("Can't render status text: {}", text)),
                }
            }).expect("Can't get rendered status text");
            self.status_text_img = img;
        }

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        let upper_left = self.planets[0].position - Vector::new(WIDTH / 2.0, HEIGHT / 2.0);
        window.set_view(View::new(Rectangle::new(upper_left, Vector::new(WIDTH, HEIGHT))));

        // background
        window.clear(Color::BLACK)?;

        // planets
        for planet in &self.planets {
            window.draw(
                &Circle::new(
                    planet.position,
                    planet.mass.powf(1.0 / 3.0),
                ), Background::Col(planet.color),
            );
        }

        // user interface
        let paused = self.paused;

        match &self.status_text_img {
            Some(image) => {
                window.draw(&Rectangle::new(upper_left, image.area().size), Img(&image));
            }
            None => (),
        }

        Ok(())
    }
}

fn integrate(time_step: f32, planets: &Vec<Planet>) -> Vec<Planet> {
    /*
    Integrate with semi-implicit Euler:
        velocity += acceleration * dt;
        position += velocity * dt;
    i. e. use next-step's velocity when computing position.
    Semi-implicit Euler is first-order (not very precise), symplectic (energy-preserving), fast integrator.
    */
    let mut new_planets: Vec<Planet> = Vec::new();
    for (ii, planet) in planets.iter().enumerate() {
        let mut force: Vector = Vector::new(0, 0);
        for (jj, curr_planet) in planets.iter().enumerate() {
            if ii != jj {
                let distance = curr_planet.position - planet.position;
                let force_size = planet.mass * curr_planet.mass / distance.len2();
                force += distance.normalize() * force_size;
            }
        }
        let new_planet = Planet {
            velocity: planet.velocity + (force / planet.mass) * time_step,
            position: planet.position + planet.velocity * time_step,
            mass: planet.mass,
            color: planet.color,
        };
        new_planets.push(new_planet);
    }
    new_planets
}

fn main() {
    let mut settings: Settings = Default::default();
    settings.update_rate = 0.1;
    run::<Space>("Hello World", Vector::new(WIDTH, HEIGHT), settings);
}