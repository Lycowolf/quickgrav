#![windows_subsystem = "windows"]
extern crate quicksilver;

use quicksilver::prelude::*;
use quicksilver::graphics::View;
use serde_derive::*;
use serde_json::from_slice;
use quicksilver::saving::{save, load};
use std::iter::once;

mod default_space;

const WIDTH: f32 = 1200.0;
const HEIGHT: f32 = 900.0;
const APP_NAME: &str = "QuickGrav";
const SAVE_PROFILE: &str = "profile1";
// misnomers: it's delay between updates
const DEFAULT_TIME_STEP: f32 = 0.001;
const DEFAULT_UPDATE_RATE: f64 = 0.01;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Planet {
    position: Vector,
    velocity: Vector,
    // per tick
    mass: f32,
    color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Object {
    Barycenter,
    Planet(usize),
}

struct Space {
    planets: Vec<Planet>,
    paused: bool,
    time_step: f32,
    font: Asset<Font>,
    status_text_img: Option<Image>,
    // Text rendering is kind of slow, we cache it here
    centered_at: Object,
    // Index of planet to center view at, or None for centering on barycenter
    rotate_with: Option<Object>,
    // The view will rotate so this planet will be at the right
    clear_screen: bool,
}

impl Space {
    fn load_planets(&mut self, filename: &str) -> () {
        self.planets = match from_slice(load_file(filename).wait().expect(format!("Can't load {}", filename).as_str()).as_slice()) {
            Ok(planets) => planets,
            _ => default_space::get_planets(),
        };
        self.centered_at = Object::Barycenter;
        self.rotate_with = None;
        self.status_text_img = None;
    }

    fn maybe_refresh_status_text(&mut self, window: &Window) -> () {
        if self.status_text_img.is_none() {
            let paused = self.paused;
            let style = FontStyle::new(16.0, Color::WHITE);
            let centering = match self.centered_at {
                Object::Barycenter => "barycenter".to_string(),
                Object::Planet(i) => format!("planet #{}", i)
            };
            let rotation: String = match self.rotate_with {
                None => "no".to_string(),
                Some(object) => match object {
                    Object::Barycenter => "barycenter".to_string(),
                    Object::Planet(i) => format!("fixing planet #{}", i),
                }
            };
            let text = format!(
                "Controls:\n\
                ----------\n\
                <+ -> change update rate, <space> pause\n\
                </ *> change time step\n\
                <S> save, <L> load\n\
                <C> center on planet, <R> rotate with planet\n\
                <Tab> toggle screen clearing (planets leave trails, messes up text rendering)\n\
                \n\
                Sample systems:\n\
                ---------------\n\
                <F1> default, unstable\n\
                <F2> stable, with moon\n\
                <F3> stable in L5 point\n\
                <F4> Binary star\n\
                \n\
                Centered at: {}\n\
                Rotation: {}\n\
                Paused: {}\n\
                Simulation time step: {}\n\
                Update rate: {} updates/sec",
                centering,
                rotation,
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
                    Err(error) => { return Err(error); }
                }
            }).expect("Can't get rendered status text");
            self.status_text_img = img;
        }
    }
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
            centered_at: Object::Barycenter,
            rotate_with: Option::None,
            clear_screen: true,
        })
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        if !self.paused {
            self.planets = integrate(self.time_step, &self.planets);
        }
        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        match event {
            Event::Key(Key::Tab, ButtonState::Pressed) => {
                self.status_text_img = None;
                self.clear_screen = !self.clear_screen;
            }
            Event::Key(Key::Space, ButtonState::Pressed) => {
                self.status_text_img = None;
                self.paused = !self.paused;
            }
            Event::Key(Key::Multiply, ButtonState::Pressed) => {
                self.status_text_img = None;
                self.time_step *= 2.0;
            }
            Event::Key(Key::Divide, ButtonState::Pressed) => {
                self.status_text_img = None;
                self.time_step /= 2.0;
            }
            // Add => faster simulation => smaller update rate (update delay)
            Event::Key(Key::Add, ButtonState::Pressed) => {
                self.status_text_img = None;
                window.set_update_rate(window.update_rate() / 2.0);
            }
            Event::Key(Key::Subtract, ButtonState::Pressed) => {
                self.status_text_img = None;
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
            Event::Key(Key::F1, ButtonState::Pressed) => {
                self.planets = default_space::get_planets()
            }
            Event::Key(Key::F2, ButtonState::Pressed) => {
                self.load_planets("system1.json");
            }
            Event::Key(Key::F3, ButtonState::Pressed) => {
                self.load_planets("system2.json");
            }
            Event::Key(Key::F4, ButtonState::Pressed) => {
                self.load_planets("system3.json");
            }
            Event::Key(Key::C, ButtonState::Pressed) => {
                self.status_text_img = None;
                self.centered_at = match self.centered_at {
                    Object::Barycenter => Object::Planet(0usize),
                    Object::Planet(i) => if i < self.planets.len() - 1 {
                        Object::Planet(i + 1)
                    } else {
                        Object::Barycenter
                    }
                };
            }

            Event::Key(Key::R, ButtonState::Pressed) => {
                self.status_text_img = None;
                let some_planets = (0..self.planets.len()).map(|i| { Some(Object::Planet(i)) });
                let mut rotations = once(None).chain(once(Some(Object::Barycenter))).chain(some_planets).cycle();
                rotations.find(|found| { *found == self.rotate_with });
                let mut next_rot = rotations.next().expect("cycled iter returned None");
                if next_rot.is_some() && next_rot.unwrap() == self.centered_at {
                    next_rot = rotations.next().expect("cycled iter returned None");
                }
                self.rotate_with = next_rot;
            }
            _ => ()
        }

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        let barycenter = self.planets.iter()
            .fold(Vector::new(0, 0), |sum, planet| { sum + planet.position * planet.mass })
            * (1.0 / self.planets.iter().map(|p| p.mass).sum::<f32>());

        let center = match self.centered_at {
            Object::Planet(i) => {
                self.planets[i].position
            }
            Object::Barycenter => barycenter,
        };
        let rotate = match self.rotate_with {
            None => Transform::IDENTITY,
            Some(object) => {
                let target = match object {
                    Object::Barycenter => barycenter,
                    Object::Planet(i) => self.planets[i].position,
                };
                Transform::rotate(-(target - center).angle()) // negative: cancel the rotation
            }
        };
        let size = Vector::new(WIDTH, HEIGHT);

        let view_transform = rotate * Transform::translate(-center);
        let view_rectangle = Rectangle::new(-size / 2.0, size); // centered at (0, 0)

        // NOTE: view takes transform that is applied at every object in the world.
        // It doesn't transform the view's rectangle
        window.set_view(View::new_transformed(view_rectangle, view_transform));

        // background
        if self.clear_screen { window.clear(Color::BLACK)? };

        // planets
        for planet in &self.planets {
            window.draw(
                &Circle::new(
                    planet.position,
                    if planet.mass > 1.0 { planet.mass.powf(1.0 / 3.0) } else { 1.0 },
                ), Background::Col(planet.color),
            );
        }

        self.maybe_refresh_status_text(&window);
        match &self.status_text_img {
            Some(image) => {
                window.draw_ex(
                    &Rectangle::new(-image.area().size / 2.0, image.area().size),
                    Img(&image),
                    view_transform.inverse() // undo the view's transformation
                        * Transform::translate((image.area().size / 2.0) - (size / 2.0)), // move it to the top left
                    1,
                );
            }
            None => panic!("Just refreshed status text and there is none"),
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
        let mut acceleration: Vector = Vector::new(0, 0);
        for (jj, other_planet) in planets.iter().enumerate() {
            if ii != jj {
                let distance = other_planet.position - planet.position;
                let acceleration_size = other_planet.mass / distance.len2();
                acceleration += distance.normalize() * acceleration_size;
            }
        }
        let new_velocity = planet.velocity + acceleration * time_step;
        let new_planet = Planet {
            velocity: new_velocity,
            position: planet.position + new_velocity * time_step,
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