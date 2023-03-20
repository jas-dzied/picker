#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use colored::Colorize;
use log::display_result;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use serde::Deserialize;
use std::{
    cmp,
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

mod log;

#[derive(Deserialize, Debug, Clone, Copy)]
struct Settings {
    solutions: i64,
}

#[derive(Deserialize, Debug)]
struct Config {
    settings: Settings,
    preferred: HashMap<String, Vec<String>>,
    unpreferred: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct Solution {
    pub rooms: Vec<(String, String)>,
    pub preferred: u64,
    pub accepted: u64,
    pub unpreferred: u64,
}

fn parse_config<T: AsRef<Path>>(path: T) -> Result<Config> {
    let text = fs::read_to_string(path)?;
    let result = toml::from_str(&text)?;
    Ok(result)
}

fn get_preferred_people(a: &str, people: &[String], config: &Config) -> Option<Vec<String>> {
    let mut result = vec![];
    for b in people {
        let a_prefers_b = config.preferred.get(a)?.contains(b);
        let b_prefers_a = config.preferred.get(b)?.contains(&a.to_string());
        if a_prefers_b && b_prefers_a {
            result.push(b.clone());
        }
    }
    Some(result)
}

fn get_accepted_people(a: &str, people: &[String], config: &Config) -> Option<Vec<String>> {
    let mut result = vec![];
    for b in people {
        let a_unprefers_b = config.unpreferred.get(a)?.contains(b);
        let b_unprefers_a = config.unpreferred.get(b)?.contains(&a.to_string());
        if !a_unprefers_b && !b_unprefers_a {
            result.push(b.clone());
        }
    }
    Some(result)
}

fn find_index<T: cmp::PartialEq>(item: &T, array: &[T]) -> Result<usize> {
    array
        .iter()
        .position(|x| x == item)
        .ok_or_else(|| anyhow!("Error choosing random item from list. Array empty."))
}

fn choose_person(
    list: &Vec<String>,
    index_list: &mut Vec<String>,
    rng: &mut ThreadRng,
) -> Result<String> {
    let person = list
        .choose(rng)
        .ok_or_else(|| anyhow!("Error choosing random person"))?;
    let index = find_index(person, index_list)?;
    Ok(index_list.remove(index))
}

fn solve(config: &Config, rng: &mut ThreadRng) -> Result<Solution> {
    let mut rooms = vec![];
    let mut preferred = 0;
    let mut accepted = 0;
    let mut unpreferred = 0;

    let mut people = config.unpreferred.keys().cloned().collect::<Vec<_>>();
    people.shuffle(rng);

    while let Some(person) = people.pop() {
        let preferred_people = get_preferred_people(&person, &people, config)
            .ok_or_else(|| anyhow!("Error generating preferred people"))?;
        let accepted_people = get_accepted_people(&person, &people, config)
            .ok_or_else(|| anyhow!("Error generating accepted people"))?;

        if !preferred_people.is_empty() {
            let second_person = choose_person(&preferred_people, &mut people, rng)?;
            rooms.push((person, second_person));
            preferred += 1;
        } else if !accepted_people.is_empty() {
            let second_person = choose_person(&accepted_people, &mut people, rng)?;
            rooms.push((person, second_person));
            accepted += 1;
        } else {
            let second_person = choose_person(&people.clone(), &mut people, rng)?;
            rooms.push((person, second_person));
            unpreferred += 1;
        }
    }

    Ok(Solution {
        rooms,
        preferred,
        accepted,
        unpreferred,
    })
}

fn generate_solutions(config: &Config, rng: &mut ThreadRng) -> Result<Vec<Solution>> {
    let mut result = vec![];
    for _ in 0..config.settings.solutions {
        result.push(solve(config, rng)?);
    }
    Ok(result)
}

fn main() -> Result<()> {
    // Uses first env arg as path to config file. If not provided, uses the
    // config.toml file in the current working directory
    let logger = log::info("Finding config file path")?;
    let default_path = String::from("config.toml");
    let path = env::args().nth(1).unwrap_or(default_path);
    let full_path = PathBuf::from(path.clone()).canonicalize().unwrap();
    let display_path = full_path.to_str().unwrap();
    logger.end();

    // Parses the provided config file into a Config struct
    let logger = log::info(format!("Parsing config file at {}", display_path.blue()))?;
    let config = parse_config(path)?;
    logger.end();

    let logger = log::info("Generating rng")?;
    let mut rng = rand::thread_rng();
    logger.end();

    // Generates n amount of solutions, randomly changing the order of the list
    // of people randomly each time, to ensure a range of solutions are generated
    let logger = log::info(format!(
        "Generating {} solutions",
        config.settings.solutions.to_string().blue()
    ))?;
    let solutions = generate_solutions(&config, &mut rng)?;
    logger.end();

    // Filters out all solutions that do not have the minimum number of unpreferred matchups
    // Then filters out all solutions that do not have the maximum number of preferred matchups
    let logger = log::info("Ranking solutions")?;
    let min_unpreferred = solutions.iter().map(|x| x.unpreferred).min().unwrap();
    let solutions = solutions
        .iter()
        .filter(|x| x.unpreferred == min_unpreferred)
        .collect::<Vec<_>>();

    let max_preferred = solutions.iter().map(|x| x.preferred).max().unwrap();
    let solutions = solutions
        .iter()
        .filter(|x| x.preferred == max_preferred)
        .collect::<Vec<_>>();
    logger.end();

    log::info(format!(
        "{} optimal solutions found",
        solutions.len().to_string().blue()
    ))?
    .end();

    display_result(solutions.choose(&mut rng).unwrap());

    Ok(())
}
