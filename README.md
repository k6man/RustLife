# la_vie

Le Jeu de la Vie de Conway dans votre terminal, écrit en Rust avec [ratatui](https://ratatui.rs).

## Lancement

```sh
cargo run --release
```

## Les règles

- Une cellule vivante avec **2 ou 3** voisines vivantes survit.
- Une cellule morte avec **exactement 3** voisines vivantes naît.
- Sinon elle meurt ou reste morte.
- **Bords morts** par défaut : toute cellule hors de la grille est considérée comme morte.
- La touche `t` bascule en mode **torique** : la grille se replie sur elle-même et les
  vaisseaux traversent les bords.

## Densité de rendu (zoom)

Un caractère du terminal peut afficher plus d'une cellule — la touche `z` fait
tourner trois niveaux :

| Niveau | Caractères | Cellules par caractère | Écran 80×24 affiche… |
|---|---|---|---|
| Plein | `█` / espace | 1×1 | 78×19 |
| Demi-bloc | `▀` `▄` `█` | 1×2 | 78×38 |
| Braille | points de braille | 2×4 | 156×76 |

Le curseur est mis en évidence par inversion vidéo dans les modes denses, la
souris reste fonctionnelle (un clic cible la mini-cellule du coin haut-gauche).

## Contrôles

| Touche | Action |
|---|---|
| `Espace` | lecture / pause |
| `n` | avancer d'une génération |
| `+` / `-` | augmente / baisse la vitesse |
| Flèches ou `hjkl` | déplacer le curseur |
| `Entrée` | dessiner / effacer la cellule sous le curseur |
| `1` / `2` / `3` / `4` | insérer un Glider / un Pulsar / un LWSS / le canon de Gosper au curseur |
| Souris | clic (ou glisser) pour dessiner des cellules |
| Molette | déplacer le curseur verticalement |
| `t` | basculer bords morts ↔ bords toriques |
| `z` | faire tourner la densité : plein (1×1) → demi-bloc (1×2) → braille (2×4) |
| `s` | sauvegarder l'état dans `la_vie_save.txt` |
| `o` | charger `la_vie_save.txt` |
| `R` | regénérer une population aléatoire |
| `c` | vider la grille |
| `q` ou `Ctrl-C` | quitter proprement |

## Architecture

- `src/game.rs` — la logique : comptage des 8 voisins (bords morts ou toriques, avec
  déduplication des voisins des petites grilles), évolution en double buffer, insertion
  de patterns, remplissage aléatoire (LCG interne), sauvegarde/chargement de l'état.
- `src/patterns.rs` — définition des patterns (Glider, Pulsar, LWSS, canon de Gosper)
  en art ASCII.
- `src/ui.rs` — le rendu ratatui : grille centrée avec densité de rendu réglable
  (plein / demi-bloc / braille), curseur, ligne de statut, messages transitoires.
- `src/app.rs` — l'état de l'application (curseur, lecture, vitesse, dessin à la souris,
  bascule de bords, densité, sauvegarde).
- `src/main.rs` — la boucle d'événements crossterm et le pilotage du terminal.

## Tests

```sh
cargo test
```

Couvre les règles (oscillateur blinker, bords morts au coin, repliement torique), les
formes des patterns, le déplacement du LWSS (vaisseau c/2 : +1 cellule vers la droite
toutes les 2 générations), la production de gliders du canon de Gosper, et la
sauvegarde/chargement (aller-retour + rejet des fichiers invalides).
