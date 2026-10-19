# la_vie — règles pour agents

## Commandes
- `cargo run --release` : lance l'application graphique (fenêtre eframe/egui).
- `cargo test` : tout le jeu de tests (règles, patterns, UI, app, save/load).
- Un seul test : `cargo test <nom>`, ex. `cargo test lwss_moves_right`, `cargo test gosper_produces_gliders`.
- Avant de considérer le travail fini : `cargo clippy --all-targets -- -D warnings`
  (zéro warning exigé) puis `cargo build --release`.
- Pas de CI/pre-commit : la barre de qualité est clippy strict + tests.

## Structure et testabilité
- Crate binaire seul (pas de `lib.rs`) : les tests sont dans `#[cfg(test)]` à
  l'intérieur des modules de `src/`. On ne peut pas écrire de tests d'intégration
  qui importent le bin.
- `game.rs` : logique pure (bords morts/toriques, double buffer).
  `patterns.rs` : art ASCII → cellules. `app.rs` : état de l'application.
  `ui.rs` : rendu egui (cellules rectangulaires ajustées à la fenêtre).
  `main.rs` : application eframe (`App::logic` pour simulation + clavier,
  `App::ui` pour l'affichage).
- Les boutons des patterns (barre d'outils) sont générés depuis
  `patterns::insertable()` : pour un nouveau pattern, il faut art + `LazyLock` +
  entrée dans `insertable()` + touche dans `main.rs::handle_keys` + test de forme.
- Les conversions écran → grille passent par `ui::cell_at` (fonction pure,
  testable sans affichage) ; ne jamais mapper les coordonnées à la main.

## egui/eframe — pièges vérifiés
- eframe 0.37 a changé l'API : le trait `eframe::App` expose `logic(&mut self,
  ctx, frame)` (simulation, clavier, `request_repaint_after`) et `ui(&mut self,
  ui, frame)` (affichage) — plus de `update(ctx, frame)`.
- Les panneaux prennent désormais `&mut Ui` : `egui::Panel::top("id").show(ui,
  …)` (et non `.show(ctx, …)`), idem `CentralPanel::show`.
- `Painter::rect_stroke` demandent un `StrokeKind` en dernier argument.
- Les coordonnées de peinture sont des `Pos2`, pas des `Vec2` (`pos + vecteur`
  → `Pos2` ; `Rect::contains` attend un `Pos2`).

## Aliasing d'époque (obsolète depuis la migration egui)
- La densité plein/demi-bloc/braille (rendu terminal) n'existe plus : la grille
  est peinte entière, taille de cellule calculée pour tenir dans la fenêtre
  (`canvas` dans `ui.rs`).

## État persistant
- `la_vie_save.txt` est un **artefact d'exécution** écrit dans le répertoire
  courant (`s`/`o` dans l'application) ; format `la_vie v1 W H GEN TOROIDAL` +
  rangées en `#`/`.`. Ne pas le committer ni l'inclure comme fixture dans les
  tests (les tests utilisent `std::env::temp_dir()`).

## Documentation à maintenir en pair
- Toute nouvelle touche ou bouton → mettre à jour la table des contrôles et la
  section correspondante de `README.md`.
