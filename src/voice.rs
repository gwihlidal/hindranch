use ggez::audio;
use std::collections::VecDeque;

pub struct VoiceQueue {
    active: Option<audio::Source>,
    pending: VecDeque<audio::Source>,
}

impl VoiceQueue {
    pub fn new() -> Self {
        VoiceQueue {
            active: None,
            pending: VecDeque::new(),
        }
    }

    pub fn process(&mut self) {
        let inactive = if let Some(ref active) = self.active {
            !active.playing()
        } else {
            false
        };

        if inactive {
            self.active = None;
        }

        if self.pending.len() > 0 && self.active.is_none() {
            self.active = self.pending.pop_back();
            if let Some(ref mut active) = self.active {
                active.play().expect("failed to play voice");
            }
        }
    }
}
