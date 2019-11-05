use quicksilver::geom::Vector;
use quicksilver::graphics::Color;
use crate::Planet;

pub fn get_planets() -> Vec<Planet> {
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

    planets
}