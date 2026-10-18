//! Rendu ratatui : la grille centrée dans un cadre (avec densité de rendu
//! réglable), le curseur, et le pied de page.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::game::Game;

const ALIVE: char = '█';
const CURSOR: char = '◆';

fn style_alive() -> Style {
    Style::new().fg(Color::Green).bold()
}

fn style_cursor() -> Style {
    Style::new().fg(Color::Yellow).bold()
}

/// Curseur sur un caractère dense (demi-bloc / braille) : inversion vidéo,
/// faut bien distinguer les cellules mortes autour du curseur.
fn style_cursor_dense() -> Style {
    Style::new().fg(Color::Black).bg(Color::Yellow)
}

/// Niveau de densité du rendu : combien de cellules de jeu par caractère du
/// terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    /// 1 cellule = 1 caractère (`█` ou espace).
    #[default]
    Full,
    /// 1 caractère = 2 cellules empilées (`▀` `▄` `█`).
    Half,
    /// 1 caractère = 8 cellules en 2×4 (points braille).
    Braille,
}

impl Density {
    /// Nombre de cellules de jeu (largeur, hauteur) par caractère du terminal.
    pub fn cells_per_char(&self) -> (u16, u16) {
        match self {
            Density::Full => (1, 1),
            Density::Half => (1, 2),
            Density::Braille => (2, 4),
        }
    }

    /// Niveau suivant dans le cycle.
    pub fn next(&self) -> Self {
        match self {
            Density::Full => Density::Half,
            Density::Half => Density::Braille,
            Density::Braille => Density::Full,
        }
    }

    /// Description lisible pour le message de statut.
    pub fn label(&self) -> &'static str {
        match self {
            Density::Full => "plein (1×1)",
            Density::Half => "demi-bloc (1×2)",
            Density::Braille => "braille (2×4)",
        }
    }
}

/// Symbole braille (U+2800…U+28FF) à partir d'un masque de 8 points :
/// le bit N correspond au point N+1, en ordre colonne-major
/// (1=(0,0), 2=(1,0), 3=(0,1), 4=(1,1), 5=(0,2), 6=(1,2), 7=(0,3), 8=(1,3)).
pub fn braille_symbol(bits: u16) -> char {
    char::from_u32(0x2800 + u32::from(bits & 0xFF)).expect("masque braille sur 8 bits")
}

/// Symbole demi-bloc selon l'état des deux cellules du caractère.
fn half_symbol(top: bool, bottom: bool) -> char {
    match (top, bottom) {
        (false, false) => ' ',
        (true, false) => '▀',
        (false, true) => '▄',
        (true, true) => '█',
    }
}

/// Symbole braille d'un caractère 2×4 dont le coin haut-gauche est (bx, by).
/// Les cellules hors grille sont mortes.
fn braille_symbol_for(game: &Game, bx: u16, by: u16) -> char {
    let mut bits = 0u16;
    for row in 0..4usize {
        for col in 0..2usize {
            if game.is_alive(bx + col as u16, by + row as u16) {
                bits |= 1u16 << (row * 2 + col);
            }
        }
    }
    braille_symbol(bits)
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).split(area);

    let title_flag = if app.running { " ▶ " } else { " ⏸ " };
    let border_mode = if app.game.toroidal { "torique" } else { "bords morts" };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" La Vie{title_flag}– {border_mode} "))
        .border_style(Style::new().fg(Color::DarkGray));
    let inner = block.inner(chunks[0]);
    f.render_widget(block, chunks[0]);

    // La densité change la couverture des caractères : on nettoie la zone pour
    // éviter de laisser des restes de la frame précédente.
    for y in inner.y..inner.bottom() {
        for x in inner.x..inner.right() {
            if let Some(cell) = f.buffer_mut().cell_mut((x, y)) {
                cell.reset();
            }
        }
    }

    let view = render_grid(f.buffer_mut(), inner, &app.game, app.cursor, app.density);
    app.view = view;

    render_footer(f, chunks[1], app);
}

/// Dessine la portion visible de la grille, centrée, à la densité demandée.
/// Renvoie la vue (en caractères) utilisée pour convertir les événements
/// souris en coordonnées de grille.
fn render_grid(
    buf: &mut Buffer,
    area: Rect,
    game: &Game,
    cursor: (u16, u16),
    density: Density,
) -> Rect {
    let (cw, ch) = density.cells_per_char();
    let cols = game.width.min(area.width.saturating_mul(cw));
    let rows = game.height.min(area.height.saturating_mul(ch));
    let vp_w = cols.div_ceil(cw).max(1).min(area.width);
    let vp_h = rows.div_ceil(ch).max(1).min(area.height);
    let gx = (game.width - cols) / 2;
    let gy = (game.height - rows) / 2;
    let view = Rect::new(
        area.x + area.width.saturating_sub(vp_w) / 2,
        area.y + area.height.saturating_sub(vp_h) / 2,
        vp_w,
        vp_h,
    );

    for c_row in 0..vp_h {
        for c_col in 0..vp_w {
            let bx = gx + c_col * cw;
            let by = gy + c_row * ch;
            let has_cursor =
                cursor.0 >= bx && cursor.0 < bx + cw && cursor.1 >= by && cursor.1 < by + ch;

            let (symbol, style) = draw_char(density, game, bx, by, has_cursor);
            if let Some(cell) = buf.cell_mut((view.x + c_col, view.y + c_row)) {
                cell.set_char(symbol).set_style(style);
            }
        }
    }
    view
}

/// Symbole + style du caractère couvrant les cellules de (bx, by) à
/// (bx + cw - 1, by + ch - 1), selon la densité.
fn draw_char(density: Density, game: &Game, bx: u16, by: u16, has_cursor: bool) -> (char, Style) {
    match density {
        Density::Full => {
            if has_cursor {
                (CURSOR, style_cursor())
            } else if game.is_alive(bx, by) {
                (ALIVE, style_alive())
            } else {
                (' ', Style::new())
            }
        }
        Density::Half => {
            let top = game.is_alive(bx, by);
            let bottom = game.is_alive(bx, by + 1);
            let style = if has_cursor {
                style_cursor_dense()
            } else if top || bottom {
                style_alive()
            } else {
                Style::new()
            };
            (half_symbol(top, bottom), style)
        }
        Density::Braille => {
            let any_alive = (0..4usize).any(|row| {
                (0..2usize).any(|col| game.is_alive(bx + col as u16, by + row as u16))
            });
            let style = if has_cursor {
                style_cursor_dense()
            } else if any_alive {
                style_alive()
            } else {
                Style::new()
            };
            (braille_symbol_for(game, bx, by), style)
        }
    }
}

fn render_footer(f: &mut Frame, area: Rect, app: &mut App) {
    let (cursor_x, cursor_y) = app.cursor;
    let status = Line::from(vec![
        Span::from(" Gen ").cyan(),
        Span::raw(app.game.generation.to_string()).cyan().bold(),
        Span::from("  Pop "),
        Span::raw(app.game.population().to_string()).green(),
        Span::from("  "),
        Span::from("pop. au curseur").dark_gray(),
        Span::from(format!(" ({}, {}) ", cursor_x, cursor_y)).dark_gray(),
    ]);
    let hints = Line::from({
        let mut spans = vec![
            Span::from("[Espace] lecture/pause "),
            Span::from("[n] pas "),
            Span::from("[-/+] vitesse "),
            Span::from("[Entrée] dessiner "),
            Span::from("[hjkl/flèches] curseur "),
            Span::from("[z] densité "),
        ];
        for (i, pattern) in crate::patterns::insertable().iter().enumerate() {
            spans.push(Span::from(format!("[{}] {} ", i + 1, pattern.name())));
        }
        spans.extend([
            Span::from("[t] bords "),
            Span::from("[s] sauver "),
            Span::from("[o] charger "),
            Span::from("[R] aléa "),
            Span::from("[c] vider "),
            Span::from("[q] quitter").red(),
        ]);
        spans
    });

    let footer = Paragraph::new(vec![status, hints]).style(Style::new().dark_gray());
    f.render_widget(footer, area);

    // Vitesse au centre-droit ; remplacée par le message transitoire s'il y en
    // a un (sauvegarde, bascule de bords, densité…).
    let msg = app.current_message();
    if let Some(msg) = msg {
        let shown = format!("{msg} ");
        let x = area.x + area.width.saturating_sub(shown.len() as u16 + 1);
        f.buffer_mut().set_string(x, area.y, &shown, Style::new().green().bold());
    } else {
        let speed = format!("{} gen/s ", app.speed);
        let x = area.x + area.width.saturating_sub(speed.len() as u16 + 1);
        f.buffer_mut()
            .set_string(x, area.y, &speed, Style::new().yellow());
    }
}

/// Convertit des coordonnées écran en coordonnées de grille (None si dehors).
/// Le clic cible la mini-cellule en haut à gauche du caractère, selon la
/// densité courante.
pub fn screen_to_game(app: &App, x: u16, y: u16) -> Option<(u16, u16)> {
    let view = app.view;
    if view.width == 0 || view.height == 0 {
        return None;
    }
    if x < view.x || y < view.y || x >= view.x + view.width || y >= view.y + view.height {
        return None;
    }
    let (cw, ch) = app.density.cells_per_char();
    let cols = app.game.width.min(view.width.saturating_mul(cw));
    let rows = app.game.height.min(view.height.saturating_mul(ch));
    // La fenêtre est centrée dans la grille : ajouter son décalage.
    let gx = (app.game.width - cols) / 2;
    let gy = (app.game.height - rows) / 2;
    Some((
        (gx + (x - view.x) * cw).min(app.game.width.saturating_sub(1)),
        (gy + (y - view.y) * ch).min(app.game.height.saturating_sub(1)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::GLIDER;

    #[test]
    fn braille_symbols() {
        assert_eq!(braille_symbol(0), '\u{2800}');
        assert_eq!(braille_symbol(1), '\u{2801}'); // point 1 = (0,0)
        assert_eq!(braille_symbol(0b0000_0000_0000_0010), '\u{2802}'); // point 2 = (1,0)
        assert_eq!(braille_symbol(0x40), '\u{2840}'); // point 7 = (0,3)
        assert_eq!(braille_symbol(0xFF), '\u{28FF}');
    }

    #[test]
    fn braille_by_layout() {
        let mut g = Game::new(4, 5);
        // Une vivante en (1,0) du caractère => point 2.
        g.set_alive(1, 0, true);
        assert_eq!(braille_symbol_for(&g, 0, 0), '\u{2802}');
        // Une vivante en (0,3) => point 7 (0x40).
        g.clear();
        g.set_alive(0, 3, true);
        assert_eq!(braille_symbol_for(&g, 0, 0), '\u{2840}');
        // Hors grille = morte (le glider tient dans la boîte 3x3).
        let mut g = Game::new(3, 3);
        g.insert_pattern(&GLIDER, Some((0, 0)));
        let c = braille_symbol_for(&g, 0, 0);
        assert_ne!(c, '\u{2800}');
    }

    #[test]
    fn density_cycle() {
        assert_eq!((1, 1), Density::Full.cells_per_char());
        assert_eq!((1, 2), Density::Half.cells_per_char());
        assert_eq!((2, 4), Density::Braille.cells_per_char());
        assert_eq!(Density::Full.next(), Density::Half);
        assert_eq!(Density::Half.next(), Density::Braille);
        assert_eq!(Density::Braille.next(), Density::Full);
        for d in [Density::Full, Density::Half, Density::Braille] {
            assert!(!d.label().is_empty());
        }
    }

    #[test]
    fn mouse_mapping_by_density() {
        let mut app = App::new(60, 40);
        app.view = Rect::new(0, 0, 30, 5);

        // Plein : 1 cellule par caractère, fenêtre 30×5 centrée dans 60×40.
        app.density = Density::Full;
        assert_eq!(screen_to_game(&app, 0, 0), Some((15, 17)));
        assert_eq!(screen_to_game(&app, 2, 4), Some((17, 21)));

        // Braille : 2 cellules par colonne, 4 par ligne. Ici la fenêtre
        // couvre toute la largeur (30×2 = 60) mais la moitié de la hauteur.
        app.density = Density::Braille;
        assert_eq!(screen_to_game(&app, 0, 0), Some((0, 10)));
        assert_eq!(screen_to_game(&app, 1, 1), Some((2, 14)));
    }
}
