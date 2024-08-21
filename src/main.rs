use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::Vec2;
use once_cell::sync::Lazy;
use rusttype::Scale;
use std::sync::Arc;
use std::time::{Duration, Instant};

mod framebuffer;
use framebuffer::Framebuffer;

mod maze;
use maze::load_maze;

mod player;
use player::{process_events, Player};

mod audio;
use audio::AudioPlayer;

mod cast_function;
use cast_function::cast_ray;

mod texture;
use texture::Texture;

const WINDOW_WIDTH: usize = 1200;
const WINDOW_HEIGHT: usize = 720;
const FRAMEBUFFER_WIDTH: usize = 1200;
const FRAMEBUFFER_HEIGHT: usize = 720;
const BLOCK_SIZE: usize = 100;
const FRAME_DURATION: Duration = Duration::from_millis(15);

static WALL: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/wall.jpg")));
static WALLALT: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/wall2.jpg")));
static SPRITE: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/alien.png")));
static STATION: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/station.jpg")));

fn walls_minimap(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
) {
    if cell != ' ' {
        framebuffer.set_current_color(0x14544b);
        for x in xo..xo + block_size {
            for y in yo..yo + block_size {
                framebuffer.point(x, y);
            }
        }
    }
}

fn game_map(framebuffer: &mut Framebuffer, player: &Player, z_buffer: &mut [f32]) {
    let maze = match load_maze("./maze.txt") {
        Ok(maze) => maze,
        Err(e) => {
            eprintln!("Failed to load maze: {}", e);
            return;
        }
    };
    let block_size = 100;

    for i in 0..framebuffer.width {
        for j in 0..(framebuffer.height / 2) {
            framebuffer.set_current_color(0x2c9473);
            framebuffer.point(i, j)
        }

        for j in (framebuffer.height / 2)..framebuffer.height {
            framebuffer.set_current_color(0x56615d);
            framebuffer.point(i, j)
        }
    }

    let hh = framebuffer.height as f32 / 2.0;

    let num_rays = framebuffer.width;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let Intersect = cast_ray(framebuffer, &maze, player, a, block_size, false);

        let distance = Intersect.distance * (a - player.a).cos();
        let mut stake_height = (framebuffer.height as f32 / distance) * 70.0;
        if stake_height > framebuffer.height as f32 {
            stake_height = framebuffer.height as f32;
        }
        let stake_top = (hh - (stake_height / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height / 2.0)) as usize;

        z_buffer[i] = distance;

        for y in stake_top..stake_bottom {
            let ty =
                (y as f32 - stake_top as f32) / (stake_bottom as f32 - stake_top as f32) * 128.0;
            let tx = Intersect.tx;
            let color = texture_walls(Intersect.impact, tx as u32, ty as u32);
            framebuffer.set_current_color(color);
            framebuffer.point(i, y)
        }
    }
}

fn maze_to_minimap(framebuffer: &mut Framebuffer, player: &Player) {
    let maze = match load_maze("./maze.txt") {
        Ok(maze) => maze,
        Err(e) => {
            eprintln!("Failed to load maze: {}", e);
            return;
        }
    };
    let block_size = 100;

    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            walls_minimap(
                framebuffer,
                col * block_size,
                row * block_size,
                block_size,
                maze[row][col],
            );
        }
    }
    framebuffer.set_current_color(0xFFFFFF);
    framebuffer.point(player.pos.x as usize, player.pos.y as usize);

    let num_rays = 150;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        cast_ray(framebuffer, &maze, player, a, block_size, true);
    }
}

fn minimap_function(
    framebuffer: &mut Framebuffer,
    maze: &[Vec<char>],
    block_size: usize,
    player: &Player,
) {
    let minimap_size = 150;
    let minimap_x = framebuffer.width - minimap_size - 80;
    let minimap_y = framebuffer.height - minimap_size - 10;

    for x in minimap_x..minimap_x + minimap_size + 70 {
        for y in minimap_y..minimap_y + minimap_size {
            framebuffer.set_current_color(0x56615d);
            framebuffer.point(x, y);
        }
    }

    let scale = minimap_size as f32 / (maze.len() as f32 * block_size as f32);
    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            let cell_x = (col as f32 * block_size as f32 * scale) as usize;
            let cell_y = (row as f32 * block_size as f32 * scale) as usize;
            let mini_block_size = (block_size as f32 * scale) as usize;
            walls_minimap(
                framebuffer,
                minimap_x + cell_x,
                minimap_y + cell_y,
                mini_block_size,
                maze[row][col],
            );
        }
    }

    framebuffer.set_current_color(0xFFFFFF);
    let player_x = (player.pos.x as f32 * scale) as usize;
    let player_y = (player.pos.y as f32 * scale) as usize;
    framebuffer.point(minimap_x + player_x, minimap_y + player_y);
}

fn render_object(framebuffer: &mut Framebuffer, player: &Player, pos: &Vec2, z_buffer: &mut [f32]) {
    let mut sprite_a = (pos.y - player.pos.y).atan2(pos.x - player.pos.x) - player.a;

    while sprite_a < -std::f32::consts::PI {
        sprite_a += 2.0 * std::f32::consts::PI;
    }
    while sprite_a > std::f32::consts::PI {
        sprite_a -= 2.0 * std::f32::consts::PI;
    }

    if sprite_a.abs() > (player.fov / 2.0) {
        return;
    }

    let sprite_d = ((player.pos.x - pos.x).powi(2) + (player.pos.y - pos.y).powi(2)).sqrt();

    if sprite_d < 10.0 {
        return;
    }

    let screen_height = framebuffer.height as f32;
    let screen_width = framebuffer.width as f32;

    let sprite_size = (screen_height / sprite_d) * 100.0 * 0.5;

    // Ajustar la posición Y para que el sprite esté más cerca del "suelo"
    let start_x =
        (screen_width / 2.0) + (sprite_a * screen_width / player.fov) - (sprite_size / 2.0);

    // Aquí se ajusta `start_y` para evitar que el sprite "flote"
    let start_y = (screen_height / 2.0) - (sprite_size / 2.0) + sprite_size * 0.25;

    let end_x = ((start_x + sprite_size) as usize).min(framebuffer.width);
    let end_y = ((start_y + sprite_size) as usize).min(framebuffer.height);
    let start_x = start_x.max(0.0) as usize;
    let start_y = start_y.max(0.0) as usize;

    for x in start_x..end_x {
        if sprite_d < z_buffer[x] {
            for y in start_y..end_y {
                let tx = ((x as f32 - start_x as f32) / sprite_size * 128.0) as u32;
                let ty = ((y as f32 - start_y as f32) / sprite_size * 128.0) as u32;
                let color = SPRITE.get_pixel_color(tx, ty);
                if color != 0xffffff {
                    framebuffer.set_current_color(color);
                    framebuffer.point(x, y);
                    z_buffer[x] = sprite_d;
                }
            }
        }
    }
}

fn position_sprites(framebuffer: &mut Framebuffer, player: &Player, z_buffer: &mut [f32]) {
    let sprites = vec![Vec2::new(250.0, 250.0), Vec2::new(1050.0, 710.0)];

    for sprite_obj in &sprites {
        render_object(framebuffer, &player, sprite_obj, z_buffer);
    }
}

fn texture_walls(cell: char, tx: u32, ty: u32) -> u32 {
    match cell {
        '+' | '|' => WALLALT.get_pixel_color(tx, ty),
        '-' => WALL.get_pixel_color(tx, ty),
        'g' => STATION.get_pixel_color(tx, ty),
        _ => 0x0000000,
    }
}

fn initialize_window(title: &str) -> Window {
    Window::new(title, WINDOW_WIDTH, WINDOW_HEIGHT, WindowOptions::default()).unwrap()
}

fn main() {
    let audio_player =
        AudioPlayer::new("assets/music.mp3").expect("Failed to initialize AudioPlayer");
    audio_player.play();
    let steps_player =
        AudioPlayer::new("assets/steps.mp3").expect("Failed to initialize AudioPlayer");

    let mut framebuffer = Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
    let mut window = initialize_window("SPACE MAZE - RUST GAME RAYCASTING");
    window.set_position(0, 0);
    window.update();

    framebuffer.set_background_color(0x213b31);

    while window.is_open() && !window.is_key_down(Key::Enter) {
        let frame_start_time = Instant::now();

        framebuffer.clear();

        let scale = Scale::uniform(32.0);
        let text1 = "BIENVENIDO A SPACE MAZE";
        let text2 = "Enter x Jugar ! Esc x Salir";

        let text1_width = framebuffer.text_width(&text1, scale);
        let text2_width = framebuffer.text_width(&text2, scale);

        let start_x1 = (FRAMEBUFFER_WIDTH as f32 - text1_width) / 2.0;
        let start_y1 = (FRAMEBUFFER_HEIGHT as f32 - scale.y) / 2.0;

        let start_x2 = (FRAMEBUFFER_WIDTH as f32 - text2_width) / 2.0;
        let start_y2 = start_y1 + scale.y + 10.0;

        framebuffer.drawtext(
            &text1,
            start_x1 as usize,
            start_y1 as usize,
            scale,
            0xFFFFFF,
        );

        framebuffer.drawtext(
            &text2,
            start_x2 as usize,
            start_y2 as usize,
            scale,
            0xFFFFFF,
        );

        window
            .update_with_buffer(&framebuffer.buffer, FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT)
            .unwrap();

        let frame_end_time = Instant::now();
        let frame_duration_actual = frame_end_time.duration_since(frame_start_time);
        if frame_duration_actual < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - frame_duration_actual);
        }
    }

    let mut player = Player {
        pos: Vec2::new(150.0, 150.0),
        a: std::f32::consts::PI / 3.0,
        fov: std::f32::consts::PI / 3.0,
        velocity: Vec2::new(0.0, 0.0),
        previous_mouse_pos: Vec2::new(0.0, 0.0),
    };

    let mut mode = "3D";

    let maze = match load_maze("./maze.txt") {
        Ok(maze) => maze,
        Err(e) => {
            eprintln!("Failed to load maze: {}", e);
            return;
        }
    };

    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut fps_text = String::new();

    let mut goal_position = Vec2::new(0.0, 0.0);
    for (row_idx, row) in maze.iter().enumerate() {
        for (col_idx, &cell) in row.iter().enumerate() {
            if cell == 'g' {
                goal_position = Vec2::new(
                    col_idx as f32 * BLOCK_SIZE as f32,
                    row_idx as f32 * BLOCK_SIZE as f32,
                );
                break;
            }
        }
    }

    while window.is_open() {
        let frame_start_time = Instant::now();

        if window.is_key_down(Key::Escape) {
            break;
        }
        if window.is_key_down(Key::M) {
            mode = if mode == "2D" { "3D" } else { "2D" };
        }

        if (player.pos - goal_position).norm() < (BLOCK_SIZE as f32) / 2.0 {
            framebuffer.clear();

            let scale = Scale::uniform(32.0);
            let text1 = "FELICIDADES LLEGASTE A LA ESTACION";
            let text2 = "CREADO POR GERCO";

            let text1_width = framebuffer.text_width(&text1, scale);
            let text2_width = framebuffer.text_width(&text2, scale);

            let start_x1 = (FRAMEBUFFER_WIDTH as f32 - text1_width) / 2.0;
            let start_y1 = (FRAMEBUFFER_HEIGHT as f32 - scale.y) / 2.0;

            let start_x2 = (FRAMEBUFFER_WIDTH as f32 - text2_width) / 2.0;
            let start_y2 = start_y1 + scale.y + 10.0;

            framebuffer.drawtext(
                &text1,
                start_x1 as usize,
                start_y1 as usize,
                scale,
                0xFFFFFF,
            );

            framebuffer.drawtext(
                &text2,
                start_x2 as usize,
                start_y2 as usize,
                scale,
                0xFFFFFF,
            );

            window
                .update_with_buffer(&framebuffer.buffer, FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT)
                .unwrap();

            let end_time = Instant::now() + Duration::from_secs(5);
            while Instant::now() < end_time {
                if !window.is_open() || window.is_key_down(Key::Escape) {
                    break;
                }

                window
                    .update_with_buffer(&framebuffer.buffer, FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT)
                    .unwrap();

                std::thread::sleep(Duration::from_millis(16));
            }

            break;
        }

        process_events(&window, &mut player, &maze, BLOCK_SIZE, &steps_player);

        framebuffer.clear();
        if mode == "2D" {
            maze_to_minimap(&mut framebuffer, &player);
        } else {
            let mut z_buffer = vec![f32::INFINITY; framebuffer.width];
            game_map(&mut framebuffer, &player, &mut z_buffer);
            position_sprites(&mut framebuffer, &player, &mut z_buffer);
        }
        minimap_function(&mut framebuffer, &maze, BLOCK_SIZE, &player);

        frame_count += 1;
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(last_time);

        if elapsed >= Duration::from_secs(1) {
            let fps = frame_count as f64 / elapsed.as_secs_f64();
            fps_text = format!("FPS: {:.0}", fps);
            last_time = current_time;
            frame_count = 0;
        }

        framebuffer.drawtext(&fps_text, 10, 10, Scale::uniform(32.0), 0xFFFFFF);

        window
            .update_with_buffer(&framebuffer.buffer, FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT)
            .unwrap();

        let frame_end_time = Instant::now();
        let frame_duration_actual = frame_end_time.duration_since(frame_start_time);
        if frame_duration_actual < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - frame_duration_actual);
        }
    }
}
