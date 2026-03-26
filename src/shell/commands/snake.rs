use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::keyboard::{self, ControlKey, KeyCode};
use crate::signals::{self, Signal};
use crate::vga::{
    self, Color, ColorCode, ScreenCharacter, VGA_BUFFER_ADDR, VGA_BUFFER_HEIGHT, VGA_BUFFER_WIDTH,
    Buffer,
};
use crate::utils::Cursor;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// Game ticks between each snake step.  Lower = faster.
// At ~18 Hz PIT, 4 ticks ≈ 220 ms per step — a comfortable starting speed.
const INITIAL_SPEED: u32 = 4;
// Speed up by 1 tick every N points scored (minimum 2 ticks per step).
const SPEED_UP_EVERY: u32 = 5;
const MIN_SPEED: u32 = 2;

// Play area boundaries (inclusive).
const AREA_TOP: u8 = 1;
const AREA_BOTTOM: u8 = (VGA_BUFFER_HEIGHT - 2) as u8; // row 23
const AREA_LEFT: u8 = 0;
const AREA_RIGHT: u8 = (VGA_BUFFER_WIDTH - 1) as u8; // col 79

// VGA characters
const SNAKE_HEAD: u8 = b'@';
const SNAKE_BODY: u8 = b'o';
const FOOD_CHAR: u8 = b'*';
const BORDER_H: u8 = b'-';
const WALL_CHAR: u8 = b'#';

// Colors
const HEAD_COLOR: ColorCode = ColorCode::new(Color::LightGreen, Color::Black);
const BODY_COLOR: ColorCode = ColorCode::new(Color::Green, Color::Black);
const FOOD_COLOR: ColorCode = ColorCode::new(Color::LightRed, Color::Black);
const WALL_COLOR: ColorCode = ColorCode::new(Color::DarkGray, Color::Black);
const SCORE_COLOR: ColorCode = ColorCode::new(Color::Yellow, Color::Black);
const HELP_COLOR: ColorCode = ColorCode::new(Color::DarkGray, Color::Black);
const GAMEOVER_COLOR: ColorCode = ColorCode::new(Color::LightRed, Color::Black);
const BG_COLOR: ColorCode = ColorCode::new(Color::White, Color::Black);

// ---------------------------------------------------------------------------
// Direction
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Dir {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Dir {
    /// Prevent 180° turns (instant self-collision).
    fn is_opposite(self, other: Dir) -> bool {
        matches!(
            (self, other),
            (Dir::Up, Dir::Down)
                | (Dir::Down, Dir::Up)
                | (Dir::Left, Dir::Right)
                | (Dir::Right, Dir::Left)
        )
    }
}

// ---------------------------------------------------------------------------
// Simple LCG pseudo-random number generator
// ---------------------------------------------------------------------------

struct Rng {
    state: u32,
}

impl Rng {
    fn new(seed: u32) -> Self {
        // Avoid a zero seed which would make the LCG degenerate.
        Rng {
            state: seed | 1,
        }
    }

    fn next(&mut self) -> u32 {
        // Numerical Recipes LCG parameters
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    /// Random u8 in [lo, hi] inclusive.
    fn range(&mut self, lo: u8, hi: u8) -> u8 {
        let span = (hi - lo) as u32 + 1;
        lo + (self.next() % span) as u8
    }
}

// ---------------------------------------------------------------------------
// Tick counter — driven by TimerTick signal
// ---------------------------------------------------------------------------

static GAME_TICKS: AtomicU32 = AtomicU32::new(0);

fn tick_handler(_signal: u8) {
    GAME_TICKS.fetch_add(1, Ordering::Relaxed);
}

// ---------------------------------------------------------------------------
// VGA direct-write helpers
//
// We write directly to the VGA buffer to avoid disturbing the Writer
// state.  This is the same approach used by timer.rs.
// ---------------------------------------------------------------------------

#[inline]
fn put_char(col: u8, row: u8, ch: u8, color: ColorCode) {
    let buf = unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) };
    buf.chars[row as usize][col as usize] = ScreenCharacter {
        ascii_value: ch,
        color,
    };
}

fn put_str(col: u8, row: u8, s: &[u8], color: ColorCode) {
    for (i, &byte) in s.iter().enumerate() {
        let c = col as usize + i;
        if c >= VGA_BUFFER_WIDTH {
            break;
        }
        put_char(c as u8, row, byte, color);
    }
}

fn clear_screen_game() {
    let blank = ScreenCharacter {
        ascii_value: b' ',
        color: BG_COLOR,
    };
    let buf = unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) };
    for row in 0..VGA_BUFFER_HEIGHT {
        for col in 0..VGA_BUFFER_WIDTH {
            buf.chars[row][col] = blank;
        }
    }
}

// ---------------------------------------------------------------------------
// Drawing helpers
// ---------------------------------------------------------------------------

fn draw_borders() {
    // Top border (row 0 doubles as score bar — draw walls at edges)
    for col in AREA_LEFT..=AREA_RIGHT {
        put_char(col, 0, BORDER_H, WALL_COLOR);
        put_char(col, AREA_BOTTOM + 1, BORDER_H, WALL_COLOR);
    }
}

fn draw_score(score: u32) {
    // "Score: NNNN" right-aligned on row 0
    let mut buf = [b' '; 16];
    buf[0] = b'S';
    buf[1] = b'c';
    buf[2] = b'o';
    buf[3] = b'r';
    buf[4] = b'e';
    buf[5] = b':';
    buf[6] = b' ';

    // Format the number into buf[7..]
    let mut n = score;
    let mut digits = [b'0'; 6];
    let mut i = 5;
    loop {
        digits[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 || i == 0 {
            break;
        }
        i -= 1;
    }
    // Copy digits after "Score: "
    let digit_len = 6 - i;
    for d in 0..digit_len {
        buf[7 + d] = digits[i + d];
    }

    put_str(2, 0, &buf[..7 + digit_len], SCORE_COLOR);
}

fn draw_help() {
    put_str(2, AREA_BOTTOM + 1, b"Arrows:move  ESC:quit", HELP_COLOR);
}

fn draw_food(col: u8, row: u8) {
    put_char(col, row, FOOD_CHAR, FOOD_COLOR);
}

fn draw_snake_head(col: u8, row: u8) {
    put_char(col, row, SNAKE_HEAD, HEAD_COLOR);
}

fn draw_snake_body(col: u8, row: u8) {
    put_char(col, row, SNAKE_BODY, BODY_COLOR);
}

fn erase_cell(col: u8, row: u8) {
    put_char(col, row, b' ', BG_COLOR);
}

// ---------------------------------------------------------------------------
// Game state
// ---------------------------------------------------------------------------

struct SnakeGame {
    body: VecDeque<(u8, u8)>,  // (col, row), front = head
    dir: Dir,
    food: (u8, u8),
    score: u32,
    speed: u32,                 // ticks per step
    rng: Rng,
    game_over: bool,
}

impl SnakeGame {
    fn new(seed: u32) -> Self {
        let mut rng = Rng::new(seed);
        let mut body = VecDeque::new();

        // Start in the middle of the play area, going right
        let start_col = (AREA_RIGHT / 2) as u8;
        let start_row = (AREA_TOP + AREA_BOTTOM) / 2;
        // Initial length of 4
        for i in 0..4u8 {
            body.push_back((start_col - i, start_row));
        }

        let food = Self::random_food_pos(&body, &mut rng);

        SnakeGame {
            body,
            dir: Dir::Right,
            food,
            score: 0,
            speed: INITIAL_SPEED,
            rng,
            game_over: false,
        }
    }

    /// Pick a random food position that doesn't overlap the snake.
    fn random_food_pos(body: &VecDeque<(u8, u8)>, rng: &mut Rng) -> (u8, u8) {
        loop {
            let col = rng.range(AREA_LEFT, AREA_RIGHT);
            let row = rng.range(AREA_TOP, AREA_BOTTOM);
            // Make sure food doesn't land on the snake
            let on_snake = body.iter().any(|&(c, r)| c == col && r == row);
            if !on_snake {
                return (col, row);
            }
        }
    }

    /// Advance the snake one step.  Returns true if still alive.
    fn step(&mut self) -> bool {
        let (hx, hy) = *self.body.front().unwrap();

        // Compute new head position
        let (nx, ny) = match self.dir {
            Dir::Up => (hx, hy.wrapping_sub(1)),
            Dir::Down => (hx, hy + 1),
            Dir::Left => (hx.wrapping_sub(1), hy),
            Dir::Right => (hx + 1, hy),
        };

        // Wall collision
        if nx < AREA_LEFT || nx > AREA_RIGHT || ny < AREA_TOP || ny > AREA_BOTTOM {
            self.game_over = true;
            return false;
        }

        // Self collision (skip the tail which is about to move)
        for &(bx, by) in self.body.iter() {
            if bx == nx && by == ny {
                self.game_over = true;
                return false;
            }
        }

        // Move: push new head
        self.body.push_front((nx, ny));

        // Check food
        if nx == self.food.0 && ny == self.food.1 {
            // Ate food — don't remove tail (snake grows)
            self.score += 1;

            // Speed up
            if self.score % SPEED_UP_EVERY == 0 && self.speed > MIN_SPEED {
                self.speed -= 1;
            }

            // Place new food
            self.food = Self::random_food_pos(&self.body, &mut self.rng);
        } else {
            // Normal move — erase and remove tail
            let (tx, ty) = self.body.pop_back().unwrap();
            erase_cell(tx, ty);
        }

        true
    }

    /// Full redraw of the snake and food.
    fn draw(&self) {
        // Draw body segments first, then head on top
        for (i, &(col, row)) in self.body.iter().enumerate() {
            if i == 0 {
                draw_snake_head(col, row);
            } else {
                draw_snake_body(col, row);
            }
        }
        draw_food(self.food.0, self.food.1);
        draw_score(self.score);
    }

    /// Incremental draw after a step (faster than full redraw):
    /// only the new head, the old head (now body), and the score.
    fn draw_step(&self) {
        if self.body.len() >= 2 {
            // Old head becomes body
            let &(bx, by) = &self.body[1];
            draw_snake_body(bx, by);
        }
        // New head
        let &(hx, hy) = self.body.front().unwrap();
        draw_snake_head(hx, hy);

        // Food (always visible — might have been placed fresh)
        draw_food(self.food.0, self.food.1);
        draw_score(self.score);
    }
}

// ---------------------------------------------------------------------------
// Game over screen
// ---------------------------------------------------------------------------

fn draw_game_over(score: u32) {
    let row = (VGA_BUFFER_HEIGHT / 2) as u8;
    put_str(30, row - 1, b"====================", GAMEOVER_COLOR);
    put_str(30, row,     b"    GAME  OVER !    ", GAMEOVER_COLOR);
    put_str(30, row + 1, b"====================", GAMEOVER_COLOR);

    // "Score: NNN"
    let mut score_buf = [b' '; 20];
    score_buf[0] = b' ';
    score_buf[1] = b' ';
    score_buf[2] = b' ';
    score_buf[3] = b'S';
    score_buf[4] = b'c';
    score_buf[5] = b'o';
    score_buf[6] = b'r';
    score_buf[7] = b'e';
    score_buf[8] = b':';
    score_buf[9] = b' ';
    let mut n = score;
    let mut digits = [b'0'; 6];
    let mut i = 5;
    loop {
        digits[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 || i == 0 {
            break;
        }
        i -= 1;
    }
    let digit_len = 6 - i;
    for d in 0..digit_len {
        score_buf[10 + d] = digits[i + d];
    }
    put_str(30, row + 2, &score_buf[..10 + digit_len], SCORE_COLOR);
    put_str(28, row + 4, b"Press ESC to return...", HELP_COLOR);
}

// ---------------------------------------------------------------------------
// Public entry point — called as a shell command
//
// Usage:  snake
// ---------------------------------------------------------------------------

pub fn run(_args: &[&str]) {
    // Drain any stale key events from previous input
    while keyboard::get_next_key_event().is_some() {}

    // Hide the hardware cursor during the game
    Cursor::disable_cursor();

    // Register our tick handler so GAME_TICKS advances
    GAME_TICKS.store(0, Ordering::Relaxed);
    signals::register_signal(Signal::TimerTick.as_u8(), tick_handler);

    // --- Set up the screen ---
    clear_screen_game();
    draw_borders();
    draw_help();

    // Seed the PRNG from the current tick count (varies every boot / launch)
    let seed = crate::timer::TICK_COUNT.load(Ordering::Relaxed);
    let mut game = SnakeGame::new(seed);
    game.draw();

    let mut last_step_tick: u32 = GAME_TICKS.load(Ordering::Relaxed);

    // -----------------------------------------------------------------------
    // Main game loop
    // -----------------------------------------------------------------------
    loop {
        // 1. Let signals (especially TimerTick) fire
        signals::dispatch_pending_signals();

        // 2. Process keyboard input
        let mut quit = false;
        while let Some(event) = keyboard::get_next_key_event() {
            if !event.pressed {
                continue;
            }
            match event.code {
                KeyCode::Control(ControlKey::Escape) => {
                    quit = true;
                    break;
                }
                KeyCode::Control(ControlKey::UpArrow) => {
                    if !Dir::Up.is_opposite(game.dir) {
                        game.dir = Dir::Up;
                    }
                }
                KeyCode::Control(ControlKey::DownArrow) => {
                    if !Dir::Down.is_opposite(game.dir) {
                        game.dir = Dir::Down;
                    }
                }
                KeyCode::Control(ControlKey::LeftArrow) => {
                    if !Dir::Left.is_opposite(game.dir) {
                        game.dir = Dir::Left;
                    }
                }
                KeyCode::Control(ControlKey::RightArrow) => {
                    if !Dir::Right.is_opposite(game.dir) {
                        game.dir = Dir::Right;
                    }
                }
                _ => {}
            }
        }

        if quit {
            break;
        }

        // 3. Step the game at the configured speed
        if !game.game_over {
            let now = GAME_TICKS.load(Ordering::Relaxed);
            if now.wrapping_sub(last_step_tick) >= game.speed {
                last_step_tick = now;
                if game.step() {
                    game.draw_step();
                } else {
                    draw_game_over(game.score);
                }
            }
        }

        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }

    // -----------------------------------------------------------------------
    // Cleanup — restore shell state
    // -----------------------------------------------------------------------
    signals::unregister_signal(Signal::TimerTick.as_u8());


    vga::clear_screen();
    vga::WRITER.lock().cursor.enable_cursor(0, 15);
    while keyboard::get_next_key_event().is_some() {}

    println!("Snake finished! Final score: {}", game.score);
}