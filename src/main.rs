use std::collections::HashSet;

use nannou::prelude::*;
use nannou_egui::egui::style::Widgets;
use nannou_egui::egui::widgets;
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
    restart: bool,
    stop: bool,
    a_star_start: (usize, usize),
    a_star_end: (usize, usize),
    speed: usize,
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
    create_maze(&mut squares, maze_size, 0.3);

    let mut a_star = AStar::new((0, 0), (maze_size - 1, maze_size - 1));
    squares = a_star.generate_potentials(&squares);

    // for _ in maze_size..maze_size * 2 {
    //     a_star.step(&squares);
    // }

    Model {
        egui,
        squares,
        maze_size,
        a_star,
        restart: false,
        stop: false,
        a_star_start: (0, 0),
        speed: 10,
        a_star_end: (maze_size - 1, maze_size - 1),
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    render_egui(&mut model.egui, &mut model.restart, &mut model.stop, &mut model.speed);

    if model.stop {
        return;
    }
    for _ in 0..model.speed {
        model.a_star.step(&model.squares);
    }
    // model.a_star.step(&model.squares);
    if app.mouse.buttons.left().is_down() {
        model.a_star_start = convert_mouse_to_index(&app.mouse.position(), model.maze_size);
        model.restart = true;
    }
    if app.mouse.buttons.right().is_down() {
        model.a_star_end = convert_mouse_to_index(&app.mouse.position(), model.maze_size);
        model.restart = true;
    }
    if model.restart {
        model.restart = false;
        model.a_star = AStar::new(model.a_star_start, model.a_star_end);
        model.squares = model.a_star.generate_potentials(&model.squares);
    }



    // model.a_star.step(&model.squares)
}
fn render_egui(egui: &mut Egui, restart: &mut bool, stop: &mut bool, speed: &mut usize) {
    let egui = egui;
    // egui.set_elapsed_time(update.since_start);

    let ctx = egui.begin_frame();

    egui::Window::new("Rum window").show(&ctx, |ui| {
        ui.label("Controls");
        let button = ui.button("Restart");
        if button.clicked() {
            *restart = true;
        }
        ui.checkbox(stop, "Stop");

        ui.add(widgets::Slider::new(speed, 1..=20).text("Speed"));

    });


}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);
    let mouse_pos = app.mouse.position();
    let size = (1.0 / (model.maze_size as f32 / 3 as f32)) * 250.0;
    let potentials = &model.squares;
    let min_square = potentials
        .iter()
        .flatten()
        .min_by(|a, b| a.potential.partial_cmp(&b.potential).unwrap())
        .unwrap();
    for row in potentials {
        for square in row {
            let color = if square.solid { 0.0 } else { 1.0 };
            let rgb1 = 1.0-(square.potential - min_square.potential) / (model.a_star.max_potential - min_square.potential);
            draw.rect()
                .xy((square.position - model.maze_size as f32 / 2.0) * (size) as f32)
                .wh(vec2(size, size))
                .color(rgb(
                    rgb1 * color,
                    rgb1 * color,
                    rgb1 * color,
                ))
                .stroke(BLACK)
                .stroke_weight(1.0);
        }
    }

    // show astar path at mouse
    for square in &model.a_star.path {
        let color = if square.solid { 0.0 } else { 0.5 };
        draw.rect()
            .xy((square.position - model.maze_size as f32 / 2.0) * size as f32)
            .wh(vec2(size as f32, size as f32))
            .color(hsl(square.potential / (model.maze_size) as f32, 0.5, color))
            .stroke(BLACK)
            .stroke_weight(1.0);
    }

    for walker in model.a_star.walkers.iter()
    {
        draw.rect()
            .xy((AStar::index_to_vec2(walker.position) - model.maze_size as f32 / 2.0) * size as f32)
            .wh(vec2(size as f32, size as f32))
            .color(hsl(1.0, 1.0, 0.5))
            .stroke(BLACK)
            .stroke_weight(1.0);
        let real_pos = (vec2(walker.position.0 as f32, walker.position.1 as f32) - model.maze_size as f32 / 2.0) * size;
        if (real_pos - mouse_pos).length() > size {
            continue;
        }
        let path = walker.path.clone();
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

fn create_maze(squares: &mut Vec<Vec<Square>>, maze_size: usize, random_delete: f32) {
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
    for _ in 0..((maze_size * maze_size) as f32 * random_delete) as usize {
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

fn convert_mouse_to_index(mouse_pos: &Vec2, maze_size: usize) -> (usize, usize) {
    let size = (1.0 / (maze_size as f32 / 3 as f32)) * 250.0;
    let x = (mouse_pos.x / size + 0.5 + maze_size as f32 / 2.0);
    let y = (mouse_pos.y / size + 0.5 + maze_size as f32 / 2.0);
    if x < 0.0 || y < 0.0 || x >= maze_size as f32 || y >= maze_size as f32 {
        return (0, 0);
    }
    // println!("x: {}, y: {}", x as usize, y as usize);
    (x as usize, y as usize)
}
