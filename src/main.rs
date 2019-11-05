#![windows_subsystem = "windows"]
extern crate quicksilver;

use quicksilver::prelude::*;
use std::ops::Index;
use quicksilver::graphics::View;
use serde_derive::*;
use quicksilver::saving::{save, load};

mod default_space;

const WIDTH: f32 = 1200.0;
const HEIGHT: f32 = 900.0;
const APP_NAME: &str = "QuickGrav";
const SAVE_PROFILE: &str = "profile1";
const DEFAULT_TIME_STEP: f32 = 0.001;
const DEFAULT_UPDATE_RATE: f64 = 0.01; // misnomer: it's delay between updates


#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Planet {
    position: Vector,
    velocity: Vector, // per tick
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
        let planets = match load::<Vec<Planet>>(APP_NAME, SAVE_PROFILE) {
            Ok(planets) => planets,
            _ => default_space::get_planets(),
        };

        let font = Asset::new(Font::load("FiraCode-Medium.ttf"));

        Ok(Space {
            planets,
            paused: true,
            time_step: DEFAULT_TIME_STEP,
            font,
            status_text_img: Option::None,
        }
        )
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        if !self.paused {
            self.planets = integrate(self.time_step, &self.planets);
        }
        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        let mut status_text_changed = false;

        match event {
            Event::Key(Key::Space, ButtonState::Pressed) => {
                    status_text_changed = true;
                    self.paused = !self.paused;
            }
            Event::Key(Key::Multiply, ButtonState::Pressed) => {
                status_text_changed = true;
                self.time_step *= 2.0;
            }
            Event::Key(Key::Divide, ButtonState::Pressed) => {
                status_text_changed = true;
                self.time_step /= 2.0;
            }
            // Add => faster simulation => smaller update rate (update delay)
            Event::Key(Key::Add, ButtonState::Pressed) => {
                status_text_changed = true;
                window.set_update_rate(window.update_rate() / 2.0);
            }
            Event::Key(Key::Subtract, ButtonState::Pressed) => {
                status_text_changed = true;
                window.set_update_rate(window.update_rate() * 2.0);
            }
            Event::Key(Key::S, ButtonState::Pressed) => {
                save(APP_NAME, SAVE_PROFILE, &self.planets).expect("Can't save planet data");
            }
            Event::Key(Key::L, ButtonState::Pressed) => {
                self.planets = match load::<Vec<Planet>>(APP_NAME, SAVE_PROFILE) {
                    Ok(planets) => planets,
                    _ => default_space::get_planets(),
        };
            }
            _ => ()
        }

        if status_text_changed || self.status_text_img.is_none() {
            let paused = self.paused;
            let style = FontStyle::new(16.0, Color::WHITE);
            let text = format!(
                "Controls: <+ -> change update rate, <space> pause, </ *> change time step, <S> save, <L> load\n\
                Paused: {}\n\
                Simulation time step: {}\n\
                Update rate: {} updates/sec",
                paused,
                self.time_step,
                1000.0 / window.update_rate()
            );
            let mut img: Option<Image> = None;
            self.font.execute(|font| {
                match font.render(&text, &style) {
                    Ok(image) => {
                        img = Some(image);
                        Ok(())
                    }
                    Err(error) => { return Err(error) }
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
    settings.update_rate = DEFAULT_UPDATE_RATE;
    run::<Space>(APP_NAME, Vector::new(WIDTH, HEIGHT), settings);
}