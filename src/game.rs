/// Logique du Jeu de la Vie de Conway avec bords morts.
///
/// Toute cellule située hors de la grille est considérée comme morte :
/// les règles ne s'appliquent qu'à l'intérieur de la grille.
use crate::patterns::Pattern;

#[derive(Debug, Clone)]
pub struct Game {
    /// Grille aplatie, indexée par `y * width + x`.
    pub grid: Vec<bool>,
    pub width: u16,
    pub height: u16,
    pub generation: u64,
    /// Si vrai, la grille se replie sur elle-même (bords toriques) ;
    /// sinon, tout ce qui est hors grille est mort (bords morts).
    pub toroidal: bool,
}

impl Game {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            grid: vec![false; usize::from(width) * usize::from(height)],
            width,
            height,
            generation: 0,
            toroidal: false,
        }
    }

    pub fn index(&self, x: u16, y: u16) -> usize {
        usize::from(y) * usize::from(self.width) + usize::from(x)
    }

    pub fn is_alive(&self, x: u16, y: u16) -> bool {
        if x >= self.width || y >= self.height {
            return false; // bords morts
        }
        self.grid[self.index(x, y)]
    }

    pub fn set_alive(&mut self, x: u16, y: u16, alive: bool) {
        if x < self.width && y < self.height {
            let i = self.index(x, y);
            self.grid[i] = alive;
        }
    }

    pub fn flip_cell(&mut self, x: u16, y: u16) {
        if x < self.width && y < self.height {
            let i = self.index(x, y);
            self.grid[i] = !self.grid[i];
        }
    }

    /// Compte les 8 voisins d'une cellule.
    ///
    /// En mode torique, la grille se replie sur elle-même ; sinon tout ce qui
    /// est hors grille est mort. Les voisins sont dédupliqués pour que les
    /// grilles de 1 ou 2 cellules de large/haut ne comptent pas deux fois la
    /// même cellule.
    pub fn neighbors(&self, x: u16, y: u16) -> u8 {
        let mut count = 0u8;
        let mut seen = [(u16::MAX, u16::MAX); 8];
        let mut seen_n = 0usize;
        let (w, h) = (i32::from(self.width), i32::from(self.height));
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if (dx, dy) == (0, 0) {
                    continue;
                }
                let (mut nx, mut ny) = (i32::from(x) + dx, i32::from(y) + dy);
                if self.toroidal {
                    // Modulo positif (Rust : % peut être négatif).
                    nx = (nx % w + w) % w;
                    ny = (ny % h + h) % h;
                } else if !(0..w).contains(&nx) || !(0..h).contains(&ny) {
                    continue; // bord mort
                }
                let (nx, ny) = (nx as u16, ny as u16);
                if (nx, ny) == (x, y) || seen[..seen_n].contains(&(nx, ny)) {
                    continue;
                }
                seen[seen_n] = (nx, ny);
                seen_n += 1;
                if self.is_alive(nx, ny) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Applique une génération des règles de Conway :
    /// - survie : vivante avec 2 ou 3 voisins
    /// - naissance : morte avec exactement 3 voisins
    /// - sinon : mort / reste morte
    pub fn step(&mut self) {
        let mut next = self.grid.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let n = self.neighbors(x, y);
                let alive = self.is_alive(x, y);
                let i = self.index(x, y);
                next[i] = matches!((alive, n), (true, 2 | 3) | (false, 3));
            }
        }
        self.grid = next;
        self.generation += 1;
    }

    /// Remplit la grille aléatoirement (~30 % de cellules vivantes).
    pub fn randomize(&mut self) {
        // LCG simple, sans dépendance externe, déterministe mais varié
        // grâce à un germe pris sur l'horloge.
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x2545_F491_4F6C_DD1D);
        let mut state = seed | 1;
        for cell in &mut self.grid {
            state = state.wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            *cell = (state >> 33) % 100 < 30;
        }
        self.generation = 0;
    }

    pub fn clear(&mut self) {
        self.grid.fill(false);
        self.generation = 0;
    }

    /// Nombre de cellules vivantes.
    pub fn population(&self) -> usize {
        self.grid.iter().filter(|&&c| c).count()
    }

    /// Sauvegarde l'état dans un fichier texte lisible :
    /// en-tête `la_vie v1 W H GEN TOROIDAL`, puis une ligne par rangée
    /// (`#` = vivante).
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        writeln!(f, "la_vie v1 {} {} {} {}", self.width, self.height, self.generation, self.toroidal)?;
        for y in 0..self.height {
            let mut line = String::with_capacity(self.width as usize);
            for x in 0..self.width {
                line.push(if self.is_alive(x, y) { '#' } else { '.' });
            }
            writeln!(f, "{line}")?;
        }
        Ok(())
    }

    /// Charge un état écrit par `save`. Erreur si format invalide.
    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read(path)?;
        let mut lines = content.split(|&b| b == b'\n');
        let header = lines
            .next()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "fichier vide"))?;
        let header = std::str::from_utf8(header)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut parts = header.split_whitespace();
        if parts.next() != Some("la_vie") || parts.next() != Some("v1") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "en-tête inconnu",
            ));
        }
        let width: u16 = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let height: u16 = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let generation: u64 = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let toroidal = matches!(parts.next(), Some("true"));
        if width == 0 || height == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "dimensions invalides",
            ));
        }
        let mut game = Game::new(width, height);
        game.toroidal = toroidal;
        game.generation = generation;
        for (y, line) in lines.enumerate() {
            if y as u16 >= height {
                break;
            }
            let line = std::str::from_utf8(line)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            for (x, c) in line.chars().enumerate() {
                if (x as u16) < width {
                    game.set_alive(x as u16, y as u16, c == '#');
                }
            }
        }
        Ok(game)
    }

    /// Coordonnées x des cellules vivantes (utile pour les tests de déplacement).
    #[cfg(test)]
    pub fn alive_xs(&self) -> impl Iterator<Item = u16> + '_ {
        (0..self.grid.len() as u16).filter(|&i| self.grid[i as usize]).map(|i| i % self.width)
    }

    /// Insère un pattern avec son coin haut-gauche en (x, y) ; centre si None.
    pub fn insert_pattern(&mut self, pattern: &Pattern, at: Option<(u16, u16)>) {
        let (x0, y0) = at.unwrap_or_else(|| {
            let x = self.width.saturating_sub(pattern.width()) / 2;
            let y = self.height.saturating_sub(pattern.height()) / 2;
            (x, y)
        });
        for (dx, dy) in pattern.cells() {
            self.set_alive(x0 + dx, y0 + dy, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::{GLIDER, GOSPER, LWSS, PULSAR};

    fn game_from(rows: &[&str]) -> Game {
        let height = rows.len() as u16;
        let width = rows[0].len() as u16;
        let mut g = Game::new(width, height);
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                g.set_alive(x as u16, y as u16, c == '#');
            }
        }
        g
    }

    #[test]
    fn blinker_oscillates() {
        let mut g = game_from(&[
            "   ",
            "###",
            "   ",
        ]);
        g.step(); // phase verticale
        assert!(g.is_alive(1, 0));
        assert!(g.is_alive(1, 1));
        assert!(g.is_alive(1, 2));
        assert!(!g.is_alive(0, 1));
        g.step(); // retour à la phase horizontale
        assert!(g.is_alive(0, 1));
        assert!(g.is_alive(1, 1));
        assert!(g.is_alive(2, 1));
        assert_eq!(g.generation, 2);
    }

    #[test]
    fn dead_borders() {
        // Cellule Seule dans un coin : elle meurt (0 voisin).
        let mut g = game_from(&["# ", "  "]);
        g.step();
        assert!(!g.is_alive(0, 0));

        // Coin : trois cellules vivantes autour d'un coin naissant.
        let mut g = game_from(&[
            "##",
            "# ",
        ]);
        // Coordonnées relatives du bloc : toutes survivent avec exactement 3 voisins.
        g.step();
        assert!(g.is_alive(0, 0));
        assert!(g.is_alive(1, 0));
        assert!(g.is_alive(0, 1));
        // (1,1) a 3 voisins : une naissance.
        assert!(g.is_alive(1, 1));
    }

    #[test]
    fn population_counts() {
        let g = game_from(&["#  ", " # ", "  #"]);
        assert_eq!(g.population(), 3);
        assert_eq!(g.neighbors(1, 1), 2);
        // Bords morts : le coin (0,0) ne voit que 1 voisin.
        assert_eq!(g.neighbors(0, 0), 1);
    }

    #[test]
    fn pattern_insert_centered() {
        let mut g = Game::new(20, 20);
        g.insert_pattern(&PULSAR, None); // centre : aucune cellule ne sort
        assert_eq!(g.population(), PULSAR.cells().len());
        assert_eq!(g.neighbors(19, 19), 0); // le coin bas-droit reste vide
    }

    #[test]
    fn lwss_moves_right() {
        // Le LWSS est un vaisseau c/2 : il se translate d'une cellule vers la
        // droite toutes les 2 générations.
        let mut g = Game::new(20, 10);
        g.insert_pattern(&LWSS, Some((3, 3)));
        let x0 = g.alive_xs().min().unwrap_or(0);
        g.step();
        g.step();
        let x1 = g.alive_xs().min().unwrap_or(0);
        assert_eq!(x1, x0 + 1);
        assert_eq!(g.population(), LWSS.cells().len());
    }

    #[test]
    fn toroidal_neighbors_wrap() {
        let mut g = Game::new(4, 4);
        g.toroidal = true;
        g.set_alive(0, 0, true);
        // (3,3) voit (0,0) à travers les bords.
        assert_eq!(g.neighbors(3, 3), 1);
        // Une cellule au bord (0,0) voit aussi (3,0), (0,3), (3,3)…
        assert_eq!(g.neighbors(0, 0), 0);

        // Un puzzle glider aide à traverser : un glider qui ressort par la
        // droite revient par la gauche.
        let mut g = Game::new(8, 8);
        g.toroidal = true;
        g.insert_pattern(&GLIDER, Some((5, 5)));
        for _ in 0..8 {
            g.step();
        }
        // Le glider ne meurt pas au bord en torique : il reste exactement
        // 5 cellules vivantes.
        assert_eq!(g.population(), 5);
    }

    #[test]
    fn gosper_produces_gliders() {
        // Le canon émet un glider toutes les 30 générations ; sur une grille
        // torique (les gliders ne meurent pas au bord), la population doit
        // dépasser les 36 cellules du canon lui-même et rester stable au-dessus
        // une fois le premier glider parti (le canon ne s'effondre pas).
        let mut g = Game::new(60, 60);
        g.toroidal = true;
        g.insert_pattern(&GOSPER, Some((4, 4)));
        let mut max_pop = g.population();
        for _ in 0..60 {
            g.step();
            max_pop = max_pop.max(g.population());
        }
        assert!(max_pop > 38, "la population stagne : max = {max_pop}");
        assert!(g.population() >= 40, "le canon s'effondre : pop = {}", g.population());
    }

    #[test]
    fn save_load_roundtrip() {
        let mut g = Game::new(7, 5);
        g.toroidal = true;
        g.generation = 42;
        g.insert_pattern(&PULSAR, None);
        let path = std::env::temp_dir().join("la_vie_test_roundtrip.txt");
        g.save(&path).unwrap();
        let g2 = Game::load(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(g2.width, 7);
        assert_eq!(g2.height, 5);
        assert_eq!(g2.generation, 42);
        assert!(g2.toroidal);
        assert_eq!(g2.grid, g.grid);
    }

    #[test]
    fn load_rejects_bad_file() {
        let path = std::env::temp_dir().join("la_vie_test_broken.txt");
        std::fs::write(&path, "n'importe quoi\n").unwrap();
        assert!(Game::load(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn pattern_insert_clips() {
        let mut g = Game::new(6, 6);
        g.insert_pattern(&GLIDER, Some((4, 4))); // partiellement hors cadre
        // Glider 3x3 : en (4,4) dans une grille 6x6, seule la cellule (1,0)
        // (soit (5,4)) reste dans la grille.
        assert_eq!(g.population(), 1);
    }
}
