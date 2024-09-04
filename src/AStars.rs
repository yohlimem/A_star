use crate::Squares::Square;
use nannou::{lyon::algorithms::length, prelude::*};
use std::borrow::Borrow;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
pub enum Done {
    NotFinished(bool),
    Finished,
}

pub struct AStar {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub position: (usize, usize),
    pub path: HashSet<Square>,
    pub walkers: Vec<Walker>,
    pub max_potential: f32,
}

#[derive(Clone)]
pub struct Walker {
    pub position: (usize, usize), // index
    pub square: Square,
    pub path: HashSet<Square>,
    counter: f32,
    speed: f32,
    pub done: Done,
}

impl Walker {
    fn new(square: Square, path: HashSet<Square>) -> Walker {
        Self {
            position: square.index,
            square,
            path,
            counter: 0.0,
            done: Done::NotFinished(false),
            speed: 100000.0,
        }
    }
}

impl Walker {
    pub fn step(&mut self, next_square: Square, max_potential: f32) {
        self.square = next_square;
        self.position = next_square.index;
        self.counter = 0.0;
        self.path.insert(next_square);
        self.speed = (1.0 - next_square.potential / max_potential)*(1.0 - next_square.potential / max_potential);
    }
}

impl AStar {
    pub fn new(start: (usize, usize), end: (usize, usize)) -> AStar {
        let mut returning = Self {
            start,
            end,
            position: start,
            path: HashSet::new(),
            walkers: Vec::new(),
            max_potential: 0.0,
        };
        returning.walkers.push(Walker::new(
            Square {
                position: vec2(start.0 as f32, start.1 as f32),
                solid: false,
                potential: 0.0,
                index: start,

            },
            HashSet::new(),
        ));
        return returning;
    }
}
impl AStar {
    pub fn step(&mut self, squares: &Vec<Vec<Square>>) {
        let mut new_walkers: Vec<Walker> = Vec::new();
        let mut finished_walker: i32 = -1;
        for (i, walker) in self.walkers.iter_mut().enumerate() {
            if let Done::Finished = walker.done {
                break;
            }
            if walker.counter <= 0.3 {
                walker.counter += walker.speed;
                // println!("Counter: {}, speed: {}", walker.counter, walker.speed);
                continue;
            }
            // println!("Step");
            let current_square = walker.square;
            let sides: [(i32, i32); 4] = [
                (0, 1),
                (1, 0),
                (-1, 0),
                (0, -1),
                // (1, 1),
                // (-1, -1),
                // (-1, 1),
                // (1, -1),
            ];
            let inside_sides: Vec<&(i32, i32)> = sides // find all sides that you can walk to
                .iter()
                .filter(|side| {
                    let new_indices = AStar::add_indecies(current_square.index, **side);
                    !AStar::is_outside(new_indices, squares)
                })
                .collect::<Vec<&(i32, i32)>>();
            let mut squares_around = inside_sides // find new indecies and then get all squares of those indecies
                .iter()
                .map(|side| AStar::add_indecies(current_square.index, **side))
                .map(|side| squares[side.0 as usize][side.1 as usize])
                .filter(|square| !square.solid && !self.path.contains(&square))
                .collect::<Vec<Square>>();

            if squares_around.len() == 0 {
                walker.done = Done::NotFinished(true);
                continue;
            }

            let mut min_square_distance_potential: Square = squares_around[0].clone();
            for square in 0..squares_around.len() {
                let current_square = squares_around[square];
                let distance = Self::potential(self.end, &min_square_distance_potential) + Self::distance(&squares[self.start.0 as usize][self.start.1 as usize], &min_square_distance_potential);
                let distance2 =  Self::potential(self.end, &current_square) + Self::distance(&squares[self.start.0 as usize][self.start.1 as usize], &current_square);

                if distance > distance2 {
                    min_square_distance_potential = current_square.clone();
                }

                new_walkers.push(Walker::new(current_square.clone(), walker.path.clone()));
                self.path.insert(current_square.clone());

                new_walkers.last_mut().unwrap().step(current_square.clone(), self.max_potential);
            }
            walker.step(min_square_distance_potential.clone(), self.max_potential);
            if walker.position == self.end {
                println!("donezo");
                finished_walker = i as i32;
                walker.done = Done::Finished;
            }
            self.path.insert(walker.square.clone());
        }
        if finished_walker != -1 {
            Self::flip_between(&mut self.walkers, 0, finished_walker as usize);
        }
        self.walkers.extend(new_walkers);
        // self.walkers
        //     .retain(|walker| walker.done != Done::NotFinished(true));
    }

    pub fn generate_potentials(&mut self, squares: &Vec<Vec<Square>>) -> Vec<Vec<Square>> {
        // square, potential
        let mut max_potential = 0.0;
        let new_squres = squares
            .iter()
            .map(|row| {
                row.iter()
                    .map(|square| {

                        let mut new_square = square.clone();
                        let potential = Self::potential(self.end, square) + Self::distance(&squares[self.start.0 as usize][self.start.1 as usize], &new_square);
                        new_square.potential = potential;

                        if potential > max_potential {
                            max_potential = potential;
                        }
                        // println!("{:?}", Self::convert_usize_i32(self.end),);
                        // println!("{:?}", square.index);
                        // println!(
                        //     "{:?} is {:?} - {:?}",
                        //     Self::sub_indecies(square.index, Self::convert_usize_i32(self.end)),
                        //     square.index,
                        //     Self::convert_usize_i32(self.end),
                        // );
                        // println!("potential {}", potential);
                        // println!("potential {}, distance: {}, start: {}, current: {}", Self::potential(self.end, square), Self::distance(&squares[self.start.0 as usize][self.start.1 as usize], &new_square), squares[self.start.0 as usize][self.start.1 as usize].position, &new_square.position);
                        return new_square.to_owned();
                    })
                    .collect()
            })
            .collect();
            self.max_potential = max_potential;
            return new_squres;
    }

    fn find_position_square(&self, squares: &Vec<Vec<Square>>) -> Option<Square> {
        for row in squares.iter() {
            for square in row.iter() {
                if square.index == self.position {
                    return Some(square.clone());
                }
            }
        }
        None
    }

    fn is_outside(index: (usize, usize), grid: &Vec<Vec<Square>>) -> bool {
        // if the index is outside the grid
        if index.0 >= grid.len() || index.1 >= grid[0].len() || index.0 < 0 || index.1 < 0 {
            return true;
        }
        false
    }

    fn add_indecies(index: (usize, usize), index2: (i32, i32)) -> (usize, usize) {
        (
            (index.0 as i32 + index2.0).abs() as usize,
            (index.1 as i32 + index2.1).abs() as usize,
        )
    }

    fn sub_indecies(index: (usize, usize), index2: (i32, i32)) -> (usize, usize) {
        (
            (index.0 as i32 - index2.0).abs() as usize,
            (index.1 as i32 - index2.1).abs() as usize,
        )
    }

    fn index_to_vec2(index: (usize, usize)) -> Vec2 {
        vec2(index.0 as f32, index.1 as f32)
    }

    fn convert_usize_i32(index: (usize, usize)) -> (i32, i32) {
        (index.0 as i32, index.1 as i32)
    }

    fn flip_between<T>(items: &mut [T], index1: usize, index2: usize) {
        if index1 != index2 && index1 < items.len() && index2 < items.len() {
            items.swap(index1, index2);
        }
    }

    // fn potential(&self, square: &Square) -> f32{
    //     Self::index_to_vec2(Self::sub_indecies(
    //         square.index,
    //         Self::convert_usize_i32(self.end),
    //     )).length()
    // }

    fn potential(end: (usize, usize), square: &Square) -> f32{
        Self::index_to_vec2(Self::sub_indecies(
            square.index,
            Self::convert_usize_i32(end),
        )).length()
    }

    fn distance(square: &Square, square2: &Square) -> f32{
       square.position.distance(square2.position)
    }
}
