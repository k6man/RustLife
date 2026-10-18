# la_vie — règles pour agents

## Commandes
- `cargo run --release` : lance la TUI (nécessite un vrai terminal ; hors TTY elle
  s'arrête proprement avec le code 1 — c'est attendu, pas un bug).
- `cargo test` : tout le jeu de tests (règles, patterns, UI, save/load).
- Un seul test : `cargo test <nom>`, ex. `cargo test lwss_moves_right`, `cargo test gosper_produces_gliders`.
- Avant de considérer le travail fini : `cargo clippy --all-targets -- -D warnings`
  (zéro warning exigé) puis `cargo build --release`.
- Pas de CI/pre-commit : la barre de qualité est clippy strict + tests.

## Structure et testabilité
- Crate binaire seul (pas de `lib.rs`) : les tests sont dans `#[cfg(test)]` à
  l'intérieur des modules de `src/`. On ne peut pas écrire de tests d'intégration
  qui importent le bin.
- `game.rs` : logique pure (bords morts/toriques, double buffer).
  `patterns.rs` : art ASCII → cellules. `app.rs` : état TUI. `ui.rs` : rendu
  ratatui (densité plein/demi-bloc/braille). `main.rs` : boucle crossterm.
- Les raccourcis listés dans le footer sont générés depuis
  `patterns::insertable()` : pour un nouveau pattern, il faut art + `LazyLock` +
  entrée dans `insertable()` + touche dans `main.rs::handle_key` + test de forme.

## Pièges vérifiés (source : erreurs des sessions précédentes)
- Reproduire un pattern de tête n'est pas fiable : le LWSS canonique fait
  9 cellules en 6×4 (pas 8 en 5×4). Les tests exécutables
  (`lwss_moves_right`, `gosper_produces_gliders`, `blinker_oscillates`) sont la
  seule vérité — si tu modifies un art ASCII, relance-les.
- ratatui 0.29 : `Style::new().fg(...)` n'est pas `const` — utiliser des fns
  d'aide (`style_alive()`) et pas des `const STYLE`.
- `i16::From<u16>` n'existe pas : convertir via `i32` (voir `App::move_cursor`).
- API buffer ratatui : `buf.cell_mut((x, y))` renvoie `Option<&mut Cell>` ;
  `set_string` est sur le `Buffer`, pas sur `Cell`.
- `ratatui::init()/restore()` dans `main.rs` installent le hook de panique qui
  restaure le terminal. Ces appels et la restauration après `run()` sont
  critiques — ne pas les supprimer.
- Les lint clippy récents attrapent `trim().split_whitespace()` et les `match`
  convertibles en `matches!` : suivre ses suggestions plutôt qu'`allow`.

## Rendu dense (Density)
- `app.view` est en **caractères d'écran**, pas en cellules de jeu. Toute
  conversion écran↔grille passe par `ui::screen_to_game`, qui multiplie par
  `Density::cells_per_char`. Ne jamais mapper (col,row) → (x,y) 1:1.
- La zone de grille est réinitialisée à chaque frame (`cell.reset()`) puisqu'un
  changement de densité change la couverture des caractères — garder ce
  nettoyage si tu touches au rendu.

## État persistant
- `la_vie_save.txt` est un **artefact d'exécution** écrit dans le répertoire
  courant (`s`/`o` en jeu) ; format `la_vie v1 W H GEN TOROIDAL` + rangées en
  `#`/`.`. Ne pas le committer ni l'inclure comme fixture dans les tests
  (les tests utilisent `std::env::temp_dir()`).

## Documentation à maintenir en pair
- Toute nouvelle touche ou option → mettre à jour la table des contrôles et la
  section correspondante de `README.md` (les hints du footer sont générés, le
  README ne l'est pas).
