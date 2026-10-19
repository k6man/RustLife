# la_vie

Le Jeu de la Vie de Conway dans une fenêtre graphique, écrit en Rust avec
[egui](https://egui.rs) / [eframe](https://github.com/emilk/egui).

## Lancement

```sh
cargo run --release
```

Ouvre une fenêtre (120×80 cellules, affichées entièrement et ajustées à la
taille de la fenêtre).

## Les règles

- Une cellule vivante avec **2 ou 3** voisines vivantes survit.
- Une cellule morte avec **exactement 3** voisines vivantes naît.
- Sinon elle meurt ou reste morte.
- **Bords morts** par défaut : toute cellule hors de la grille est considérée comme morte.
- Le bouton `Bords` (touche `t`) bascule en mode **torique** : la grille se replie
  sur elle-même et les vaisseaux traversent les bords.

## Contrôles

| Touche / souris | Action |
|---|---|
| `Espace` | lecture / pause |
| `n` | avancer d'une génération |
| `+` / `-` (ou le slider) | augmente / baisse la vitesse |
| Flèches ou `hjkl` | déplacer le curseur |
| `Entrée` | dessiner / effacer la cellule sous le curseur |
| `1` / `2` / `3` / `4` (ou les boutons) | insérer un Glider / un Pulsar / un LWSS / le canon de Gosper au curseur |
| Clic + glisser sur la grille | dessiner / effacer des cellules |
| `t` (ou bouton `Bords`) | basculer bords morts ↔ bords toriques |
| `s` (bouton `Sauver`) | sauvegarder l'état dans `la_vie_save.txt` |
| `o` (bouton `Charger`) | charger `la_vie_save.txt` |
| `R` (bouton `Aléa`) | regénérer une population aléatoire |
| `c` (bouton `Vider`) | vider la grille |
| `q` ou `Échap` (bouton `Quitter`) | quitter proprement |

## Architecture

- `src/game.rs` — la logique : comptage des 8 voisins (bords morts ou toriques, avec
  déduplication des voisins des petites grilles), évolution en double buffer, insertion
  de patterns, remplissage aléatoire (LCG interne), sauvegarde/chargement de l'état.
- `src/patterns.rs` — définition des patterns (Glider, Pulsar, LWSS, canon de Gosper)
  en art ASCII.
- `src/ui.rs` — le rendu egui : grille peinte entièrement dans le panneau central
  (taille de cellule ajustée à la fenêtre), curseur, ligne de statut, messages
  transitoires.
- `src/app.rs` — l'état de l'application (curseur, lecture, vitesse, dessin à la souris,
  bascule de bords, sauvegarde).
- `src/main.rs` — l'application eframe : interface (`App::ui`), simulation et clavier
  (`App::logic`).

## Tests

```sh
cargo test
```

Couvre les règles (oscillateur blinker, bords morts au coin, repliement torique), les
formes des patterns, le déplacement du LWSS (vaisseau c/2 : +1 cellule vers la droite
toutes les 2 générations), la production de gliders du canon de Gosper, la
sauvegarde/chargement (aller-retour + rejet des fichiers invalides) et l'état de
l'application (curseur, vitesse, messages).
