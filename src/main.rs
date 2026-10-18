//! `la_vie` — Jeu de la Vie de Conway dans le terminal, avec ratatui.

mod app;
mod game;
mod patterns;
mod ui;

use std::io;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
        KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::DefaultTerminal;

use crate::app::App;
use crate::patterns::{GLIDER, GOSPER, LWSS, PULSAR};

const TICK: Duration = Duration::from_millis(16);

fn main() -> ExitCode {
    // La capture souris doit être active avant `ratatui::init()`, qui met
    // lui-même le terminal en mode brut et installe un hook de panique qui
    // restaure l'écran (mode brut + écran alternatif) en cas de crash.
    if let Err(e) = enable_raw_mode() {
        eprintln!("Impossible de passer le terminal en mode brut : {e}");
        return ExitCode::FAILURE;
    }
    if let Err(e) = execute!(io::stdout(), EnableMouseCapture) {
        let _ = disable_raw_mode();
        eprintln!("Impossible d'activer la capture souris : {e}");
        return ExitCode::FAILURE;
    }

    let mut terminal = ratatui::init();

    let size = terminal.size().unwrap_or_default();
    let mut app = App::new(
        size.width.saturating_sub(2).max(4),  // - 2 bordures
        size.height.saturating_sub(5).max(4), // - 2 bordures + 2 lignes de statut
    );

    let result = run(&mut terminal, &mut app);

    // Restauration propre du terminal dans tous les cas.
    ratatui::restore();
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), DisableMouseCapture);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Erreur : {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    let mut last_step = Instant::now();
    while !app.quit {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(TICK)? {
            handle_event(app)?;
        }

        if app.running && last_step.elapsed() >= step_interval(app.speed) {
            app.game.step();
            last_step = Instant::now();
        }
    }
    Ok(())
}

/// Intervalle entre deux générations selon la vitesse (générations/seconde).
fn step_interval(speed: u32) -> Duration {
    Duration::from_millis(u64::from(1_000 / speed.max(1)))
}

fn handle_event(app: &mut App) -> io::Result<()> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(app, key),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        Event::Resize(_, _) => {} // ratatui redessine tout à la prochaine frame
        _ => {}
    }
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) | (_, KeyCode::Char('q')) => app.quit = true,
        (_, KeyCode::Char(' ')) => app.running = !app.running,
        (_, KeyCode::Char('n')) => app.step(),
        (_, KeyCode::Char('+')) | (_, KeyCode::Char('=')) => app.speed_up(),
        (_, KeyCode::Char('-')) | (_, KeyCode::Char('_')) => app.speed_down(),
        (_, KeyCode::Char('r') | KeyCode::Char('R')) => app.game.randomize(),
        (_, KeyCode::Char('c' | 'C')) => app.game.clear(),
        (_, KeyCode::Char('t')) => app.toggle_toroidal(),
        (_, KeyCode::Char('z' | 'Z')) => app.cycle_density(),
        (_, KeyCode::Char('s')) => app.save(),
        (_, KeyCode::Char('o')) => app.load(),
        (_, KeyCode::Enter) => app.draw_here(),
        (_, KeyCode::Char('1')) => app.insert_pattern(&GLIDER),
        (_, KeyCode::Char('2')) => app.insert_pattern(&PULSAR),
        (_, KeyCode::Char('3')) => app.insert_pattern(&LWSS),
        (_, KeyCode::Char('4')) => app.insert_pattern(&GOSPER),
        (_, KeyCode::Up | KeyCode::Char('k')) => app.move_cursor(0, -1),
        (_, KeyCode::Down | KeyCode::Char('j')) => app.move_cursor(0, 1),
        (_, KeyCode::Left | KeyCode::Char('h')) => app.move_cursor(-1, 0),
        (_, KeyCode::Right | KeyCode::Char('l')) => app.move_cursor(1, 0),
        _ => {}
    }
}

fn handle_mouse(app: &mut App, mouse: crossterm::event::MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some((gx, gy)) = ui::screen_to_game(app, mouse.column, mouse.row) {
                app.drag_value = !app.game.is_alive(gx, gy);
                app.game.set_alive(gx, gy, app.drag_value);
                app.drawing = true;
            }
        }
        MouseEventKind::Drag(MouseButton::Left) if app.drawing => {
            if let Some((gx, gy)) = ui::screen_to_game(app, mouse.column, mouse.row) {
                app.game.set_alive(gx, gy, app.drag_value);
            }
        }
        MouseEventKind::Up(_) => app.drawing = false,
        MouseEventKind::ScrollUp => app.move_cursor(0, -1),
        MouseEventKind::ScrollDown => app.move_cursor(0, 1),
        _ => {}
    }
}
