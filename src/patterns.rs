//! Patterns prédéfinis du Jeu de la Vie.
//!
//! Chaque pattern est défini par un art ASCII (`#` = vivant) analysé une
//! seule fois et partagé via `LazyLock`.

use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: &'static str,
    cells: Vec<(u16, u16)>,
    width: u16,
    height: u16,
}

impl Pattern {
    /// Analyse un art ASCII (`#` = cellule vivante, tout le reste = morte).
    fn parse(name: &'static str, art: &str) -> Self {
        let mut cells = Vec::new();
        let mut width = 0u16;
        let mut height = 0u16;
        for (y, line) in art.lines().enumerate() {
            height = height.max((y + 1) as u16);
            for (x, c) in line.chars().enumerate() {
                width = width.max((x + 1) as u16);
                if c == '#' {
                    cells.push((x as u16, y as u16));
                }
            }
        }
        Self {
            name,
            cells,
            width,
            height,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn cells(&self) -> &[(u16, u16)] {
        &self.cells
    }
}

/// Art ASCII des patterns.
pub const GLIDER_ART: &str = "\
.#.
..#
###";

pub const PULSAR_ART: &str = "\
..###...###..
.............
#....#.#....#
#....#.#....#
#....#.#....#
..###...###..
.............
..###...###..
#....#.#....#
#....#.#....#
#....#.#....#
.............
..###...###..";

pub const LWSS_ART: &str = "\
.####.
#...#.
....#.
#..#..";

pub const GOSPER_ART: &str = "\
........................#
......................#.#
............##......##............##
...........#...#....##............##
##........#.....#...##..............
##........#...#.##....#.#...........
..........#.....#.......#
...........#...#
............##";

/// Glider : 5 cellules, se déplace en diagonale (bas-droite ici).
pub static GLIDER: LazyLock<Pattern> = LazyLock::new(|| Pattern::parse("Glider", GLIDER_ART));

/// Pulsar : oscillateur de période 3, 13x13, 48 cellules.
pub static PULSAR: LazyLock<Pattern> = LazyLock::new(|| Pattern::parse("Pulsar", PULSAR_ART));

/// LWSS (spaceship léger) : se déplace vers la droite.
pub static LWSS: LazyLock<Pattern> = LazyLock::new(|| Pattern::parse("LWSS", LWSS_ART));

/// Canon à gliders de Gosper : produit un glider toutes les 30 générations,
/// indéfiniment. Boîte 36x9, 36 cellules.
pub static GOSPER: LazyLock<Pattern> =
    LazyLock::new(|| Pattern::parse("Canon Gosper", GOSPER_ART));

/// Les patterns insérables au curseur, dans l'ordre des touches 1/2/3/4.
pub fn insertable() -> [&'static LazyLock<Pattern>; 4] {
    [&GLIDER, &PULSAR, &LWSS, &GOSPER]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glider_shape() {
        assert_eq!(GLIDER.cells().len(), 5);
        assert_eq!(GLIDER.width(), 3);
        assert_eq!(GLIDER.height(), 3);
    }

    #[test]
    fn pulsar_shape() {
        assert_eq!(PULSAR.cells().len(), 48);
        assert_eq!(PULSAR.width(), 13);
        assert_eq!(PULSAR.height(), 13);
    }

    #[test]
    fn lwss_shape() {
        // LWSS canonique : 9 cellules dans une boîte 6x4.
        assert_eq!(LWSS.cells().len(), 9);
        assert_eq!(LWSS.width(), 6);
        assert_eq!(LWSS.height(), 4);
    }

    #[test]
    fn gosper_shape() {
        // Canon de Gosper : 36 cellules, boîte 36x9.
        assert_eq!(GOSPER.cells().len(), 36);
        assert_eq!(GOSPER.width(), 36);
        assert_eq!(GOSPER.height(), 9);
    }

    #[test]
    fn insertable_list() {
        assert_eq!(insertable().len(), 4);
        assert_eq!(insertable()[1].cells().len(), 48);
    }
}
