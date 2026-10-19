//! État de l'application : jeu, curseur, vitesse, état de lecture.

use std::time::Instant;

use crate::game::Game;
use crate::patterns::Pattern;

const MIN_SPEED: u32 = 1;
const MAX_SPEED: u32 = 60;
const DEFAULT_SPEED: u32 = 10;
/// Durée d'affichage des messages transitoires (sauvegarde, bascule…).
const MESSAGE_DURATION: std::time::Duration = std::time::Duration::from_secs(2);
/// Fichier de sauvegarde par défaut (dans le répertoire courant).
pub const SAVE_FILE: &str = "la_vie_save.txt";

pub struct App {
    pub game: Game,
    pub cursor: (u16, u16),
    pub running: bool,
    pub speed: u32,
    /// Souris en train de dessiner (clic gauche enfoncé sur la grille).
    pub drawing: bool,
    /// Valeur peinte pendant le drag (naissante ou mourante).
    pub drag_value: bool,
    /// Cellule sous le pointeur, si elle est au-dessus de la grille.
    pub hover: Option<(u16, u16)>,
    /// Dernière cellule peinte pendant l'appui courant (Reset du souris).
    pub last_painted: Option<(u16, u16)>,
    pub quit: bool,
    /// Message transitoire affiché dans la barre de statut.
    pub message: Option<(String, Instant)>,
}

impl App {
    pub fn new(width: u16, height: u16) -> Self {
        let mut game = Game::new(width, height);
        game.randomize();
        Self {
            cursor: (width / 2, height / 2),
            running: true,
            speed: DEFAULT_SPEED,
            drawing: false,
            drag_value: false,
            hover: None,
            last_painted: None,
            quit: false,
            message: None,
            game,
        }
    }

    /// Bascule entre bords toriques et bords morts.
    pub fn toggle_toroidal(&mut self) {
        self.game.toroidal = !self.game.toroidal;
        let mode = if self.game.toroidal { "bords toriques" } else { "bords morts" };
        self.set_message(format!("Mode : {mode}"));
    }

    /// Sauvegarde l'état dans `SAVE_FILE`.
    pub fn save(&mut self) {
        let path = std::path::Path::new(SAVE_FILE);
        let msg = match self.game.save(path) {
            Ok(()) => format!("Sauvegardé dans {SAVE_FILE}"),
            Err(e) => format!("Échec de sauvegarde : {e}"),
        };
        self.set_message(msg);
    }

    /// Charge `SAVE_FILE` (l'état courant reste intact en cas d'erreur).
    pub fn load(&mut self) {
        let path = std::path::Path::new(SAVE_FILE);
        match Game::load(path) {
            Ok(mut game) => {
                // Ramener le curseur dans les dimensions de la grille chargée.
                self.cursor = (
                    self.cursor.0.min(game.width.saturating_sub(1)),
                    self.cursor.1.min(game.height.saturating_sub(1)),
                );
                std::mem::swap(&mut self.game, &mut game);
                self.set_message(format!("Chargé depuis {SAVE_FILE}"));
            }
            Err(e) => self.set_message(format!("Échec de chargement : {e}")),
        }
    }

    /// Affiche un message transitoire dans la barre de statut.
    pub fn set_message(&mut self, msg: String) {
        self.message = Some((msg, Instant::now()));
    }

    /// Message encore visible, ou None si absent ou expiré.
    pub fn current_message(&mut self) -> Option<&str> {
        let expired = self
            .message
            .as_ref()
            .is_some_and(|(_, t)| t.elapsed() > MESSAGE_DURATION);
        if expired {
            self.message = None;
        }
        self.message.as_ref().map(|(msg, _)| msg.as_str())
    }

    /// Avance d'une génération manuellement.
    pub fn step(&mut self) {
        self.game.step();
    }

    pub fn speed_up(&mut self) {
        self.speed = (self.speed + 2).min(MAX_SPEED);
    }

    /// Baisse la vitesse de 2 gen/s (clampée à [MIN_SPEED, MAX_SPEED]), avec
    /// un dernier pas de 1 pour que la vitesse minimale soit accessible.
    pub fn speed_down(&mut self) {
        self.speed = (self.speed - 1).saturating_sub(1).max(MIN_SPEED);
    }

    /// Déplace le curseur de (dx, dy) en restant dans la grille.
    pub fn move_cursor(&mut self, dx: i16, dy: i16) {
        let cx = i32::from(self.cursor.0) + i32::from(dx);
        let cy = i32::from(self.cursor.1) + i32::from(dy);
        self.cursor = (
            cx.clamp(0, i32::from(self.game.width.saturating_sub(1))) as u16,
            cy.clamp(0, i32::from(self.game.height.saturating_sub(1))) as u16,
        );
    }

    /// Dessine/efface la cellule sous le curseur.
    pub fn draw_here(&mut self) {
        let (x, y) = self.cursor;
        self.game.flip_cell(x, y);
    }

    /// Insère un pattern centré sur le curseur.
    pub fn insert_pattern(&mut self, pattern: &Pattern) {
        let (cx, cy) = self.cursor;
        let at = (
            cx.saturating_sub(pattern.width() / 2),
            cy.saturating_sub(pattern.height() / 2),
        );
        self.game.insert_pattern(pattern, Some(at));
        self.set_message(format!("{} inséré", pattern.name()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::GOSPER;

    #[test]
    fn cursor_moves_within_grid() {
        let mut app = App::new(20, 10);
        app.running = false;
        app.cursor = (0, 0);
        app.move_cursor(-1, 0);
        app.move_cursor(0, -1);
        assert_eq!(app.cursor, (0, 0), "le curseur reste dans la grille");
        app.move_cursor(1, 0);
        app.move_cursor(0, 1);
        assert_eq!(app.cursor, (1, 1));
        app.move_cursor(i16::MAX, i16::MAX);
        assert_eq!(app.cursor, (19, 9), "clampé au coin bas-droit");
    }

    #[test]
    fn speed_limits() {
        let mut app = App::new(10, 10);
        app.speed = MAX_SPEED;
        app.speed_up();
        assert_eq!(app.speed, MAX_SPEED);
        let mut downs = 0;
        while app.speed > MIN_SPEED {
            app.speed_down();
            downs += 1;
            assert!(downs < 100, "la vitesse ne descend plus");
        }
        assert_eq!(app.speed, MIN_SPEED);
    }

    #[test]
    fn insert_pattern_reports_name() {
        let mut app = App::new(60, 60);
        app.insert_pattern(&GOSPER);
        assert_eq!(app.current_message(), Some("Canon Gosper inséré"));
    }

    #[test]
    fn messages_expire() {
        let mut app = App::new(10, 10);
        app.set_message("coucou".into());
        assert_eq!(app.current_message(), Some("coucou"));
        // Forcer l'expiration.
        app.message = Some(("vieux".into(), Instant::now() - MESSAGE_DURATION * 2));
        assert_eq!(app.current_message(), None);
        assert!(app.message.is_none(), "le message expiré est retiré");
    }
}
