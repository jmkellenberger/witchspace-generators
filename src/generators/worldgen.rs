use crate::prelude::*;

const STARPORT_DISTRIBUTION: [&str; 11] = ["A", "A", "A", "B", "B", "C", "C", "D", "E", "E", "X"];

pub fn generate_mainworld<R: Rollable>(
    rng: &mut R,
    hz_variance: i32,
    habitable_zone: i32,
) -> World {
    let orbit_roll = rng.flux(0);
    let mainworld_type_roll = rng.flux(0);
    let mainworld_type = mainworld_type(mainworld_type_roll, orbit_roll);

    let port = roll_starport(rng.roll(2, 6, -2));

    let naval_roll = rng.roll(2, 6, 0);
    let naval_base = match port.as_str() {
        "A" => naval_roll <= 6,
        "B" => naval_roll <= 5,
        _ => false,
    };
    let scout_roll = rng.roll(2, 6, 0);
    let scout_base = match port.as_str() {
        "A" => scout_roll <= 4,
        "B" => scout_roll <= 5,
        "C" => scout_roll <= 6,
        "D" => scout_roll <= 7,
        _ => false,
    };

    let bases = match (naval_base, scout_base) {
        (true, true) => vec![Base::Naval, Base::Scout],
        (true, false) => vec![Base::Naval],
        (false, true) => vec![Base::Scout],
        (false, false) => Vec::new(),
    };
    let size = match rng.roll(2, 6, -2) {
        10 => rng.roll(1, 6, 9),
        roll => roll,
    };

    // Asteroid worlds are placed as planetoid belts, ignoring HZ_var
    let orbit = if size == 0 {
        rng.roll(2, 6, -1 + habitable_zone).max(0)
    } else {
        (habitable_zone + hz_variance).max(0)
    };

    let atmosphere = match size {
        0 => 0,
        _ => rng.flux(size).clamp(0, 15),
    };

    let hydrographics = match (size, atmosphere) {
        (0 | 1, _) => 0,
        (_, 0..=2 | 10..=15) => rng.flux(atmosphere - 4).clamp(0, 10),
        (_, _) => rng.flux(atmosphere).clamp(0, 10),
    };

    let population = match rng.roll(2, 6, -2) {
        10 => rng.roll(2, 6, 3),
        roll => roll,
    };

    let population_digit = match population {
        0 => 0,
        _ => rng.roll(1, 9, 0),
    };

    let government = match population {
        0 => 0,
        _ => rng.flux(population).clamp(0, 15),
    };
    let law = match population {
        0 => 0,
        _ => rng.flux(government).clamp(0, 18),
    };

    let tech = rng
        .roll(
            1,
            6,
            tech_mod(
                &port,
                size,
                atmosphere,
                hydrographics,
                population,
                government,
            ),
        )
        .clamp(0, 33);

    let travel_zone = match (port.as_str(), government + law) {
        ("X", _) | (_, 22..=32) => TravelZone::Red,
        (_, 20 | 21) => TravelZone::Amber,
        _ => TravelZone::Green,
    };

    World {
        mainworld_type,
        hz_variance,
        orbit,
        port,
        bases,
        size,
        atmosphere,
        hydrographics,
        population,
        population_digit,
        government,
        law,
        tech,
        travel_zone,
    }
}

fn roll_starport(roll: i32) -> String {
    String::from(STARPORT_DISTRIBUTION[roll as usize])
}

fn mainworld_type(flux: i32, orbit_roll: i32) -> MainWorldType {
    let close_orbit = (orbit_roll + 6).clamp(1, 11) as u8;
    let far_orbit = close_orbit + 13;
    match flux {
        -5 | -4 => MainWorldType::FarSatellite(far_orbit),
        -3 => MainWorldType::CloseSatellite(close_orbit),
        _ => MainWorldType::Planet,
    }
}

fn tech_mod(
    port: &str,
    size: i32,
    atmosphere: i32,
    hydrographics: i32,
    population: i32,
    government: i32,
) -> i32 {
    port_tech(port)
        + size_tech(size)
        + atmmosphere_tech(atmosphere)
        + hydrographics_tech(hydrographics)
        + population_tech(population)
        + government_tech(government)
}

fn port_tech(port: &str) -> i32 {
    match port {
        "A" => 6,
        "B" => 4,
        "C" => 2,
        "F" => 1,
        "X" => -4,
        _ => 0,
    }
}

fn size_tech(size: i32) -> i32 {
    match size {
        0..=1 => 2,
        2..=4 => 1,
        _ => 0,
    }
}

fn atmmosphere_tech(atmosphere: i32) -> i32 {
    match atmosphere {
        0..=3 | 10..=15 => 1,
        _ => 0,
    }
}

fn hydrographics_tech(hydrographics: i32) -> i32 {
    match hydrographics {
        9 => 1,
        10 => 2,
        _ => 0,
    }
}

fn population_tech(population: i32) -> i32 {
    match population {
        1..=5 => 1,
        9 => 2,
        10..=15 => 4,
        _ => 0,
    }
}

fn government_tech(government: i32) -> i32 {
    match government {
        0 | 5 => 1,
    13 => -2,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::population_tech;

    #[test]
    fn test_tech_low_pop_mod() {
        for i in 1..=5 {
            assert_eq!(population_tech(i), 1)
        }
    }

    #[test]
    fn test_tech_9_pop_mod() {
        assert_eq!(population_tech(9), 2)
    }

    #[test]
    fn test_tech_huge_pop_mod() {
        for i in 10..=15 {
            assert_eq!(population_tech(i), 4)
        }
    }
}
