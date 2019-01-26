use crate::Point2;

pub trait Enemy {
    fn health(&self) -> u8;
    fn damage(&mut self, amount: u8);
    fn alive(&self) -> bool;
    fn see_player(&self) -> Option<Point2>;
}

pub struct Bulldozer {
    health: u8,
}

impl Bulldozer {
    pub fn new(health: u8) -> Self {
        Bulldozer { health }
    }
}

impl Enemy for Bulldozer {
    fn health(&self) -> u8 {
        self.health
    }

    fn damage(&mut self, amount: u8) {
        self.health -= std::cmp::min(amount, self.health);
    }

    fn alive(&self) -> bool {
        self.health > 0
    }

    fn see_player(&self) -> Option<Point2> {
        None
    }
}

pub struct Sheriff {
    health: u8,
}

impl Sheriff {
    pub fn new(health: u8) -> Self {
        Sheriff { health }
    }
}

impl Enemy for Sheriff {
    fn health(&self) -> u8 {
        self.health
    }

    fn damage(&mut self, amount: u8) {
        self.health -= std::cmp::min(amount, self.health);
    }

    fn alive(&self) -> bool {
        self.health > 0
    }

    fn see_player(&self) -> Option<Point2> {
        None
    }
}
