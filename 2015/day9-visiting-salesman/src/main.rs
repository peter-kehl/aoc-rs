#![feature(unboxed_closures)]
#![allow(unused_variables)]
use std::collections::{HashMap, HashSet};
use std::fs;
//use std::str::Lines;

type Distances = HashMap<(u8, u8), u16>;

/// Returns the shortest possible visiting salesman trip given the already visited subpath & current node. The total trip covers all cities once (i.e. NO return to the city of origin).
/// All arguments (or their parts) representing a city are 0-based indices, matching `distances`.
/// # Arguments
/// * `previously_visited` previously visited cities, in this order, excluding `from`; not necessary for functionality, but useful for assertions & debugging
/// * `from` city to visit next (if Some; if None, then we are at the top level and try all cities)
/// * `previously_visited_distance` distance of this trip (as per `previously_visited` but excluding `from`) so far
/// ...
/// * `compare_before_leaf_level` Whether we can compare the subpath length to `best_total_distance_known`, and if the subpath is already longer, stop exploring this subpath any deeper. Pass true when searching for the shortest route. Pass false when searching for the longest route.
///
fn best_total_distance(
    previously_visited: &Vec<u8>,
    from: Option<u8>,
    previously_visited_distance: u16,
    given_cities_to_visit: &Vec<u8>,
    mut best_total_distance_known: u16, // initially u16::MAX for shortest distance, or 0 for longest.
    distances: &Distances,
    depth: usize, // for debugging
    compare_to_best: impl Copy + Fn(&u16, &u16) -> bool,
    compare_before_leaf_level: bool,
) -> u16 {
    assert!(from.is_some() || depth == 0);

    assert!((depth <= 1) == previously_visited.is_empty()); // previously_visited is empty if depth==0 or depth==1.
    assert!(depth == 0 || previously_visited.len() == depth - 1);
    assert!(from.is_none() || !previously_visited.iter().any(|&city| city == from.unwrap()));

    let already_visited = if let Some(from) = from {
        let mut already_visited = previously_visited.clone();
        already_visited.insert(already_visited.len(), from);
        already_visited
    } else {
        assert!(previously_visited.is_empty());
        vec![]
    };

    assert!(
        from.is_none()
            || !given_cities_to_visit
                .iter()
                .any(|&city| city == from.unwrap())
    );
    assert!(
        !given_cities_to_visit.is_empty(),
        "Call only if there is at least one city left to visit."
    );
    let at_leaf_level = given_cities_to_visit.len() == 1;

    // TODO - if I remove & after the recurse I undo
    // TODO consider SmallVec - because the deeper were are, the fewer items in the vector
    // - so less cloning to do, and it may fit on stack
    //let mut cities_yet_to_visit = given_cities_to_visit.clone();

    for &to in given_cities_to_visit {
        let leg_distance = if let Some(from) = from {
            *distances.get(&(from, to)).unwrap()
        } else {
            0
        };
        let new_sub_distance = previously_visited_distance + leg_distance;

        if !compare_before_leaf_level
            || compare_to_best(&new_sub_distance, &best_total_distance_known)
        {
            if at_leaf_level {
                // reached the bottom - the subroute is complete
                if compare_before_leaf_level
                    || compare_to_best(&new_sub_distance, &best_total_distance_known)
                {
                    best_total_distance_known = new_sub_distance;
                }
                continue;
            }

            // TODO move `cities_to_visit_after_the_current` outside the loop:
            let mut cities_to_visit_after_the_current = given_cities_to_visit.clone();
            cities_to_visit_after_the_current.retain(|&city| city != to);
            assert!(cities_to_visit_after_the_current.len() == given_cities_to_visit.len() - 1);

            let deep_result = best_total_distance(
                &already_visited,
                Some(to),
                new_sub_distance,
                &cities_to_visit_after_the_current,
                best_total_distance_known,
                distances,
                depth + 1,
                compare_to_best,
                compare_before_leaf_level,
            );
            // any call to rest_of_the_trip_starting_from(...) must return result no longer than
            // given shortest_total_distance_so_far passed to it. So here we don't need any
            // extra check like
            //    if deep_result <= shortest_total_distance_so_far { ... }
            // before we assign to shortest_total_distance_so_far - just assign:
            assert!(
                deep_result == best_total_distance_known
                    || compare_to_best(&deep_result, &best_total_distance_known)
            );
            if deep_result != best_total_distance_known {
                dbg!("breakpoint modifying: best_total_distance_known");
            }
            best_total_distance_known = deep_result;
        } // else: shortcut/optimization: don't recurse
    }
    best_total_distance_known
}

fn main() {
    let input = fs::read_to_string("input.txt").unwrap();
    let lines = input.lines();

    let line_entries: Vec<(&str, &str, u16)> = lines
        .map(|line| {
            let line: Vec<&str> = line.split_whitespace().collect();
            (line[0], line[2], line[4].parse().unwrap())
        })
        .collect();

    let city_names = line_entries.iter().fold(
        HashSet::<&str>::new(),
        |mut result, (from_name, to_name, _)| {
            result.insert(from_name);
            result.insert(to_name); // needed, because the last target city (in the input) doesn't have an exit leg
            result
        },
    );
    println!("{:#?}", city_names);

    // map each city to a unique usize, starting with 0
    let city_to_idx = city_names
        .iter()
        .fold(HashMap::<&str, u8>::new(), |mut result, city| {
            result.insert(city, result.len() as u8);
            result
        });
    println!("{:#?}", city_to_idx);

    // full matrix of (from_idx, to_idx) => u16 distance
    let distances = line_entries.iter().fold(
        Distances::new(),
        |mut result, (from_name, to_name, distance)| {
            let from_idx = *city_to_idx.get(from_name).unwrap();
            let to_idx = *city_to_idx.get(to_name).unwrap();
            assert_ne!(from_idx, to_idx);
            result.insert((from_idx, to_idx), *distance);
            result.insert((to_idx, from_idx), *distance);
            result
        },
    );
    println!("{:#?}", distances);
    // Assert that the reverse distances are the same:
    assert!((0..city_names.len() as u8).all(|x| {
        (0..city_names.len() as u8)
            .all(|y| x == y || distances.get(&(x, y)).unwrap() == distances.get(&(y, x)).unwrap())
    }));

    let all_cities = (0..city_names.len() as u8).into_iter().collect::<Vec<_>>();

    println!("{:#?}", all_cities);
    let already_visited = Vec::<u8>::new();
    println!(
        "MIN: {}",
        best_total_distance(
            &already_visited,
            None,
            0,
            &all_cities,
            u16::MAX,
            &distances,
            0,
            u16::lt,
            true
        )
    );
    println!(
        "MAX: {}",
        best_total_distance(
            &already_visited,
            None,
            0,
            &all_cities,
            0,
            &distances,
            0,
            u16::gt,
            false
        )
    )
}
