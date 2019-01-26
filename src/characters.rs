use crate::graphics::Image;
use crate::Rect;
use crate::Context;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct CharacterEntry {
    name: String,
    gun: Rect,
    hold: Rect,
    machine: Rect,
    reload: Rect,
    silencer: Rect,
    stand: Rect,
}

pub struct Characters {
    pub image: Rc<Image>,
    pub entries: Vec<CharacterEntry>,
}

impl Characters {
    pub fn load(ctx: &mut Context) -> Self {
        let mut entries: Vec<CharacterEntry> = Vec::new();

        entries.push(CharacterEntry {
            name: String::from("hitman"),
            gun: Rect::new(164.0, 88.0, 49.0, 43.0),
            hold: Rect::new(386.0, 88.0, 35.0, 43.0),
            machine: Rect::new(214.0, 44.0, 49.0, 43.0),
            reload: Rect::new(313.0, 132.0, 39.0, 43.0),
            silencer: Rect::new(0.0, 132.0, 54.0, 43.0),
            stand: Rect::new(426.0, 176.0, 33.0, 43.0),
        });

        entries.push(CharacterEntry {
            name: String::from("man_blue"),
            gun: Rect::new(263.0, 132.0, 49.0, 43.0),
            hold: Rect::new(390.0, 132.0, 35.0, 43.0),
            machine: Rect::new(212.0, 176.0, 49.0, 43.0),
            reload: Rect::new(309.0, 0.0, 39.0, 43.0),
            silencer: Rect::new(58.0, 0.0, 54.0, 43.0),
            stand: Rect::new(426.0, 132.0, 33.0, 43.0),
        });

        entries.push(CharacterEntry {
            name: String::from("man_brown"),
            gun: Rect::new(262.0, 176.0, 49.0, 43.0),
            hold: Rect::new(388.0, 0.0, 35.0, 43.0),
            machine: Rect::new(214.0, 88.0, 49.0, 43.0),
            reload: Rect::new(312.0, 176.0, 39.0, 43.0),
            silencer: Rect::new(0.0, 176.0, 54.0, 43.0),
            stand: Rect::new(459.0, 44.0, 33.0, 43.0),
        });

        entries.push(CharacterEntry {
            name: String::from("man_old"),
            gun: Rect::new(216.0, 0.0, 49.0, 43.0),
            hold: Rect::new(422.0, 88.0, 35.0, 43.0),
            machine: Rect::new(213.0, 132.0, 49.0, 43.0),
            reload: Rect::new(307.0, 44.0, 39.0, 43.0),
            silencer: Rect::new(55.0, 132.0, 54.0, 43.0),
            stand: Rect::new(460.0, 132.0, 33.0, 43.0),
        });

        entries.push(CharacterEntry {
            name: String::from("robot"),
            gun: Rect::new(164.0, 44.0, 49.0, 43.0),
            hold: Rect::new(423.0, 44.0, 35.0, 43.0),
            machine: Rect::new(166.0, 0.0, 49.0, 43.0),
            reload: Rect::new(306.0, 88.0, 39.0, 43.0),
            silencer: Rect::new(55.0, 176.0, 54.0, 43.0),
            stand: Rect::new(458.0, 88.0, 33.0, 43.0),
        });

        entries.push(CharacterEntry {
            name: String::from("soldier"),
            gun: Rect::new(113.0, 0.0, 52.0, 43.0),
            hold: Rect::new(349.0, 0.0, 38.0, 43.0),
            machine: Rect::new(110.0, 132.0, 52.0, 43.0),
            reload: Rect::new(264.0, 44.0, 42.0, 43.0),
            silencer: Rect::new(0.0, 0.0, 57.0, 43.0),
            stand: Rect::new(353.0, 132.0, 36.0, 43.0),
        });

        entries.push(CharacterEntry {
            name: String::from("survivor"),
            gun: Rect::new(112.0, 88.0, 51.0, 43.0),
            hold: Rect::new(352.0, 176.0, 37.0, 43.0),
            machine: Rect::new(110.0, 176.0, 51.0, 43.0),
            reload: Rect::new(264.0, 88.0, 41.0, 43.0),
            silencer: Rect::new(0.0, 88.0, 56.0, 43.0),
            stand: Rect::new(390.0, 176.0, 35.0, 43.0),
        });

        entries.push(CharacterEntry {
            name: String::from("woman_green"),
            gun: Rect::new(58.0, 44.0, 52.0, 43.0),
            hold: Rect::new(347.0, 44.0, 38.0, 43.0),
            machine: Rect::new(111.0, 44.0, 52.0, 43.0),
            reload: Rect::new(266.0, 0.0, 42.0, 43.0),
            silencer: Rect::new(0.0, 44.0, 57.0, 43.0),
            stand: Rect::new(386.0, 44.0, 36.0, 43.0),
        });

        entries.push(CharacterEntry {
            name: String::from("zombie"),
            gun: Rect::new(163.0, 132.0, 49.0, 43.0),
            hold: Rect::new(424.0, 0.0, 35.0, 43.0),
            machine: Rect::new(162.0, 176.0, 49.0, 43.0),
            reload: Rect::new(346.0, 88.0, 39.0, 43.0),
            silencer: Rect::new(57.0, 88.0, 54.0, 43.0),
            stand: Rect::new(460.0, 0.0, 33.0, 43.0),
        });

        Characters {
            image: Rc::new(Image::new(ctx, "/characters.png").unwrap()),
            entries,
        }
    }

    pub fn get_entry(&self, name: &str) -> CharacterEntry {
        let entry = self.entries
                .iter()
                .find(|&entry| entry.name == name).unwrap();
        entry.clone()
    }
}