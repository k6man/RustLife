//! `la_vie` — Jeu de la Vie de Conway dans une fenêtre graphique (eframe/egui).

mod app;
mod game;
mod patterns;
mod ui;

use std::time::{Duration, Instant};

use eframe::{Frame, NativeOptions};
use eframe::egui::{self, Context};

use crate::app::App;
use crate::patterns::{GLIDER, GOSPER, LWSS, PULSAR};

/// Taille de la grille de jeu (fixe, indépendante de la fenêtre : la grille
/// est affichée entièrement, ajustée au panneau central).
const GRID_W: u16 = 120;
const GRID_H: u16 = 80;

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("la_vie — Jeu de la Vie de Conway")
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    let state = LaVie {
        app: App::new(GRID_W, GRID_H),
        last_step: Instant::now(),
    };
    eframe::run_native("la_vie", options, Box::new(move |_cc| Ok(Box::new(state))))
}

/// Adaptateur entre l'état [`App`] et la boucle eframe : `logic` fait avancer
/// la simulation et traite le clavier, `ui` dessine l'interface.
struct LaVie {
    app: App,
    last_step: Instant,
}

impl eframe::App for LaVie {
    fn logic(&mut self, ctx: &Context, _frame: &mut Frame) {
        handle_keys(&mut self.app, ctx);

        if self.app.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if self.app.running && self.last_step.elapsed() >= step_interval(self.app.speed) {
            self.app.step();
            self.last_step = Instant::now();
        }
        if self.app.running {
            // Prochaine frame à l'échéance du tick : économe en CPU quand la
            // souris ne bouge pas (egui ne redessine que sur événement sinon).
            ctx.request_repaint_after(step_interval(self.app.speed).saturating_sub(self.last_step.elapsed()));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        ui::draw(ui, &mut self.app);
    }
}

/// Intervalle entre deux générations selon la vitesse (générations/seconde).
fn step_interval(speed: u32) -> Duration {
    Duration::from_millis(u64::from(1_000 / speed.max(1)))
}

fn handle_keys(app: &mut App, ctx: &Context) {
    use egui::Key;
    ctx.input(|i| {
        // `Q` ou `Échap` : quitter proprement.
        if i.key_pressed(Key::Q) || i.key_pressed(Key::Escape) {
            app.quit = true;
        }
        if i.key_pressed(Key::Space) {
            app.running = !app.running;
        }
        if i.key_pressed(Key::N) {
            app.step();
        }
        if i.key_pressed(Key::Equals) {
            app.speed_up();
        }
        if i.key_pressed(Key::Minus) {
            app.speed_down();
        }
        if i.key_pressed(Key::R) {
            app.game.randomize();
            app.set_message("Population aléatoire".into());
        }
        if i.key_pressed(Key::C) {
            app.game.clear();
            app.set_message("Grille vidée".into());
        }
        if i.key_pressed(Key::T) {
            app.toggle_toroidal();
        }
        if i.key_pressed(Key::S) {
            app.save();
        }
        if i.key_pressed(Key::O) {
            app.load();
        }
        if i.key_pressed(Key::Enter) {
            app.draw_here();
        }
        for (key, pattern) in [
            (Key::Num1, &GLIDER),
            (Key::Num2, &PULSAR),
            (Key::Num3, &LWSS),
            (Key::Num4, &GOSPER),
        ] {
            if i.key_pressed(key) {
                app.insert_pattern(pattern);
            }
        }
        if i.key_pressed(Key::ArrowUp) || i.key_pressed(Key::K) {
            app.move_cursor(0, -1);
        }
        if i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::J) {
            app.move_cursor(0, 1);
        }
        if i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::H) {
            app.move_cursor(-1, 0);
        }
        if i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::L) {
            app.move_cursor(1, 0);
        }
    });
}
