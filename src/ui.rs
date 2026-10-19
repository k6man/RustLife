//! Rendu egui : barre d'outils, grille ajustée au panneau central, barre de
//! statut. Les cellules sont peintes comme des rectangles ; la taille d'une
//! cellule est calculée pour que TOUTE la grille tienne dans la fenêtre.

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Ui, Vec2};

use crate::app::App;
use crate::game::Game;

const COLOR_BACKGROUND: Color32 = Color32::from_rgb(18, 18, 24);
const COLOR_ALIVE: Color32 = Color32::from_rgb(64, 200, 96);
const COLOR_GRID_LINE: Color32 = Color32::from_rgb(40, 40, 52);
const COLOR_CURSOR: Color32 = Color32::from_rgb(255, 200, 60);
const COLOR_HOVER: Color32 = Color32::from_rgb(230, 230, 230);

pub fn draw(ui: &mut Ui, app: &mut App) {
    toolbar(ui, app);
    status_bar(ui, app);
    egui::CentralPanel::default().show(ui, |ui| canvas(app, ui));
}

/// Barre d'outils : boutons. Ceux des patterns sont générés depuis
/// [`crate::patterns::insertable`] (clavier 1–4 en parallèle).
fn toolbar(ui: &mut Ui, app: &mut App) {
    egui::Panel::top("toolbar").show(ui, |ui| {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let play = if app.running { "⏸ Pause" } else { "▶ Lecture" };
            if ui.button(play).clicked() {
                app.running = !app.running;
            }
            if ui.button("Pas →").clicked() {
                app.step();
            }
            ui.separator();
            ui.add(egui::Slider::new(&mut app.speed, 1..=60).text("gen/s"));
            ui.separator();
            for pattern in crate::patterns::insertable() {
                if ui.button(pattern.name()).clicked() {
                    app.insert_pattern(pattern);
                }
            }
            ui.separator();
            if ui.button("Bords").clicked() {
                app.toggle_toroidal();
            }
            if ui.button("Aléa").clicked() {
                app.game.randomize();
                app.set_message("Population aléatoire".into());
            }
            if ui.button("Vider").clicked() {
                app.game.clear();
                app.set_message("Grille vidée".into());
            }
            if ui.button("Sauver").clicked() {
                app.save();
            }
            if ui.button("Charger").clicked() {
                app.load();
            }
            if ui.button("Quitter").clicked() {
                app.quit = true;
            }
        });
        ui.add_space(4.0);
    });
}

/// Panneau central : peint la grille entière (ajustée) et gère la souris.
fn canvas(app: &mut App, ui: &mut Ui) {
    let (rect, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    let (gw, gh) = (f32::from(app.game.width), f32::from(app.game.height));
    let cell = (rect.width() / gw).min(rect.height() / gh).floor().max(2.0);
    let size = Vec2::new(cell * gw, cell * gh);
    let origin: Pos2 = rect.min + (rect.size() - size) / 2.0;
    let canvas = Rect::from_min_size(origin, size);
    app.hover = resp
        .hover_pos()
        .and_then(|pos| cell_at(canvas, cell, app.game.width, app.game.height, pos));

    let painter = ui.painter_at(rect);
    painter.rect_filled(canvas, 0.0, COLOR_BACKGROUND);
    if cell >= 8.0 {
        draw_grid_lines(&painter, canvas, cell, app.game.width, app.game.height);
    }
    for (x, y) in alive_cells(&app.game) {
        painter.rect_filled(cell_rect(origin, cell, x, y), 1.0, COLOR_ALIVE);
    }
    let stroke_cursor = egui::Stroke::new(2.0, COLOR_CURSOR);
    let stroke_hover = egui::Stroke::new(1.5, COLOR_HOVER);
    if let Some((hx, hy)) = app.hover {
        painter.rect_stroke(
            cell_rect(origin, cell, hx, hy),
            0.0,
            stroke_hover,
            egui::StrokeKind::Middle,
        );
    }

    // Peinture à la souris : appui → bascule, glisser → reprend la valeur.
    let painting = resp.is_pointer_button_down_on();
    if !painting {
        app.drawing = false;
        app.last_painted = None;
    } else if let Some((x, y)) = app.hover {
        if app.last_painted != Some((x, y)) {
            if !app.drawing {
                app.drawing = true;
                app.drag_value = !app.game.is_alive(x, y);
            }
            app.game.set_alive(x, y, app.drag_value);
            app.last_painted = Some((x, y));
        }
    }

    let (cx, cy) = app.cursor;
    painter.rect_stroke(
        cell_rect(origin, cell, cx, cy),
        0.0,
        stroke_cursor,
        egui::StrokeKind::Middle,
    );
}

/// Zone d'une cellule (en coordonnées écran).
fn cell_rect(origin: Pos2, cell: f32, x: u16, y: u16) -> Rect {
    Rect::from_min_size(
        origin + Vec2::new(f32::from(x) * cell, f32::from(y) * cell),
        Vec2::splat(cell),
    )
}

/// Lignes de grille discrètes (affichées seulement si les cellules sont
/// assez grosses pour les porter).
fn draw_grid_lines(painter: &egui::Painter, canvas: Rect, cell: f32, width: u16, height: u16) {
    let stroke = egui::Stroke::new(1.0, COLOR_GRID_LINE);
    for x in 1..u32::from(width) {
        let px = canvas.left() + x as f32 * cell;
        painter.line_segment(
            [Pos2::new(px, canvas.top()), Pos2::new(px, canvas.bottom())],
            stroke,
        );
    }
    for y in 1..u32::from(height) {
        let py = canvas.top() + y as f32 * cell;
        painter.line_segment(
            [Pos2::new(canvas.left(), py), Pos2::new(canvas.right(), py)],
            stroke,
        );
    }
}

/// Convertit une position pointeur en coordonnées de grille (None si dehors).
/// Fonction pure (testable) : c'est l'équivalent egui de `screen_to_game`.
pub fn cell_at(canvas: Rect, cell: f32, width: u16, height: u16, pos: Pos2) -> Option<(u16, u16)> {
    if cell <= 0.0 || !canvas.contains(pos) {
        return None;
    }
    let x = ((pos.x - canvas.left()) / cell) as u16;
    let y = ((pos.y - canvas.top()) / cell) as u16;
    (x < width && y < height).then_some((x, y))
}

/// Coordonnées des cellules vivantes, ordre ligne par ligne.
fn alive_cells(game: &Game) -> impl Iterator<Item = (u16, u16)> + '_ {
    (0..usize::from(game.height) * usize::from(game.width))
        .filter(|&i| game.grid[i])
        .map(|i| {
            (
                (i % usize::from(game.width)) as u16,
                (i / usize::from(game.width)) as u16,
            )
        })
}

/// Barre du bas : génération, population, curseur, puis message transitoire
/// (ou la vitesse) collé à droite.
fn status_bar(ui: &mut Ui, app: &mut App) {
    egui::Panel::bottom("status").show(ui, |ui| {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(format!("Gen {}", app.game.generation));
            ui.separator();
            ui.label(format!("Pop {}", app.game.population()));
            ui.separator();
            let mode = if app.game.toroidal { "torique" } else { "bords morts" };
            let state = if app.running { "▶" } else { "⏸" };
            ui.label(format!("{state} La Vie · {mode}"));
            ui.separator();
            ui.label(format!("({},{})", app.cursor.0, app.cursor.1));

            let right = app
                .current_message()
                .map(str::to_owned)
                .unwrap_or_else(|| format!("{} gen/s", app.speed));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(right).strong());
            });
        });
        ui.add_space(2.0);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canvas() -> Rect {
        Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(400.0, 200.0))
    }

    #[test]
    fn cell_at_maps_inside() {
        let c = canvas();
        // 4 px par cellule, grille 100x50 : toute la grille tient.
        assert_eq!(cell_at(c, 4.0, 100, 50, Pos2::new(10.0, 20.0)), Some((0, 0)));
        assert_eq!(cell_at(c, 4.0, 100, 50, Pos2::new(409.5, 219.5)), Some((99, 49)));
        assert_eq!(cell_at(c, 4.0, 100, 50, Pos2::new(14.99, 24.0)), Some((1, 1)));
    }

    #[test]
    fn cell_at_rejects_outside() {
        let c = canvas();
        assert_eq!(cell_at(c, 4.0, 100, 50, Pos2::new(9.99, 20.0)), None);
        assert_eq!(cell_at(c, 4.0, 100, 50, Pos2::new(410.0, 100.0)), None);
        assert_eq!(cell_at(c, 4.0, 100, 50, Pos2::new(100.0, 220.1)), None);
        assert_eq!(cell_at(c, 0.0, 100, 50, Pos2::new(20.0, 30.0)), None);
    }

    #[test]
    fn cell_at_clips_to_grid() {
        let c = canvas();
        // 5 px par cellule : 400/5 = 80 colonnes de grille, au-delà → None.
        assert_eq!(cell_at(c, 5.0, 100, 50, Pos2::new(409.0, 30.0)), Some((79, 2)));
        assert_eq!(cell_at(c, 5.0, 100, 50, Pos2::new(409.99, 30.0)), Some((79, 2)));
    }

    #[test]
    fn alive_cells_are_enumerated() {
        let mut g = Game::new(3, 2);
        g.set_alive(0, 1, true);
        g.set_alive(2, 0, true);
        assert_eq!(alive_cells(&g).collect::<Vec<_>>(), vec![(2, 0), (0, 1)]);
        assert_eq!(alive_cells(&Game::new(3, 2)).count(), 0);
    }
}
