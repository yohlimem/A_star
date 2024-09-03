use std::collections::HashSet;

use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};

mod Squares;
use Squares::Square;
mod AStars;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use AStars::{AStar, Done};

struct Model {
    // window: Window,
    egui: Egui,
    maze_size: usize,
    squares: Vec<Vec<Square>>,
    a_star: AStar,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    let maze_size = 100;

    // generate maze
    let mut squares = vec![];
    for i in 0..maze_size {
        let mut row = vec![];
        for j in 0..maze_size {
            row.push(Square {
                position: vec2(i as f32, j as f32),
                solid: true,
                potential: 0.0,
                index: (i, j),
            });
        }
        squares.push(row);
    }

    // Create the maze using DFS
    create_maze(&mut squares, maze_size, 500);

    let a_star = AStar::new((0, 0), (maze_size - 1, maze_size - 1));
    squares = a_star.generate_potentials(&squares);

    Model {
        egui,
        squares,
        maze_size,
        a_star,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    render_egui(&mut model.egui);

    // if app.elapsed_frames() % 10 == 0 {
    for _ in 0..10 {
        model.a_star.step(&model.squares);
    }
    // }

    // model.a_star.step(&model.squares)
}
fn render_egui(egui: &mut Egui) {
    let egui = egui;
    // egui.set_elapsed_time(update.since_start);

    let ctx = egui.begin_frame();

    egui::Window::new("Rum window").show(&ctx, |ui| {
        // ui.label("res"); // template
        // ui.add(egui::Slider::new(&mut model.num, 1.0..=40.0));
    });
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);
    let size = (1.0 / (model.maze_size as f32 / 3 as f32)) * 400.0;
    let potentials = &model.squares;
    for row in potentials {
        for square in row {
            let color = if square.solid { 0.0 } else { 0.5 };
            // println!("{}", square.potential);
            draw.rect()
                .xy((square.position - model.maze_size as f32 / 2.0) * (size) as f32)
                .wh(vec2(size, size))
                .color(rgb(
                    (square.potential / (model.maze_size) as f32 + 0.3) * color,
                    (square.potential / (model.maze_size) as f32 + 0.3) * color,
                    (square.potential / (model.maze_size) as f32 + 0.3) * color,
                ))
                .stroke(BLACK)
                .stroke_weight(1.0);
        }
    }

    // show astar path
    for square in &model.a_star.path {
        // println!("{:?}", square.index);
        let color = if square.solid { 0.0 } else { 0.5 };
        draw.rect()
            .xy((square.position - model.maze_size as f32 / 2.0) * size as f32)
            .wh(vec2(size as f32, size as f32))
            .color(hsl(square.potential / (model.maze_size) as f32, 0.5, color))
            .stroke(BLACK)
            .stroke_weight(1.0);
    }

    if model
        .a_star
        .walkers
        .iter()
        .filter(|walker| walker.done == Done::Finished)
        .count()
        > 0
    {
        let path = model
            .a_star
            .walkers
            .iter()
            .filter(|walker| walker.done == Done::Finished)
            .next()
            .unwrap()
            .path
            .clone();
        for square in path {
            let color = if square.solid { 0.0 } else { 0.5 };
            draw.rect()
                .xy((square.position - model.maze_size as f32 / 2.0) * size as f32)
                .wh(vec2(size as f32, size as f32))
                .color(hsl(0.5, 0.8, color))
                .stroke(BLACK)
                .stroke_weight(1.0);
        }
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn create_maze(squares: &mut Vec<Vec<Square>>, maze_size: usize, random_delete: usize) {
    let mut rng = thread_rng();
    let mut stack = vec![];
    let start_x = rng.gen_range(0..maze_size);
    let start_y = rng.gen_range(0..maze_size);
    squares[start_x][start_y].solid = false;
    stack.push((start_x, start_y));

    while let Some((x, y)) = stack.pop() {
        let mut neighbors = vec![];
        if x > 1 {
            neighbors.push((x - 2, y));
        }
        if x < maze_size - 2 {
            neighbors.push((x + 2, y));
        }
        if y > 1 {
            neighbors.push((x, y - 2));
        }
        if y < maze_size - 2 {
            neighbors.push((x, y + 2));
        }

        neighbors.shuffle(&mut rng);

        for &(nx, ny) in &neighbors {
            if squares[nx][ny].solid {
                squares[nx][ny].solid = false;
                squares[(x + nx) / 2][(y + ny) / 2].solid = false;
                stack.push((nx, ny));
            }
        }
    }
    // choose some random squares to be non solid
    for _ in 0..(random_delete) {
        choose_random_square(&mut rng, squares, maze_size, 0)
    }

    squares[0][0].solid = false;
    squares[1][0].solid = false;
    squares[0][1].solid = false;
    squares[1][1].solid = false;
    squares[maze_size - 1][maze_size - 1].solid = false;
    squares[maze_size - 2][maze_size - 1].solid = false;
    squares[maze_size - 1][maze_size - 2].solid = false;
    squares[maze_size - 2][maze_size - 2].solid = false;
}
fn choose_random_square(
    rng: &mut rand::rngs::ThreadRng,
    squares: &mut Vec<Vec<Square>>,
    maze_size: usize,
    times: usize,
) {
    if times > 20 {
        return;
    }
    let x = rng.gen_range(0..maze_size);
    let y = rng.gen_range(0..maze_size);

    if !squares[x][y].solid {
        choose_random_square(rng, squares, maze_size, times + 1);
        // println!("trying again");
        return;
    }
    squares[x][y].solid = false;
}
