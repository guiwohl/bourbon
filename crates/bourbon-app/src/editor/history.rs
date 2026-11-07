use bourbon_core::NoteId;

#[derive(Default)]
pub(crate) struct History {
    entries: Vec<NoteId>,
    position: usize,
}

impl History {
    pub(crate) fn visit(&mut self, id: &NoteId) {
        if self.entries.get(self.position) == Some(id) {
            return;
        }
        self.entries.truncate(self.position + 1);
        self.entries.push(id.clone());
        self.position = self.entries.len() - 1;
    }

    pub(crate) fn back(&mut self, usable: &dyn Fn(&NoteId) -> bool) -> Option<NoteId> {
        let found = (0..self.position)
            .rev()
            .find(|&index| usable(&self.entries[index]))?;
        Some(self.move_to(found))
    }

    pub(crate) fn forward(&mut self, usable: &dyn Fn(&NoteId) -> bool) -> Option<NoteId> {
        let found =
            (self.position + 1..self.entries.len()).find(|&index| usable(&self.entries[index]))?;
        Some(self.move_to(found))
    }

    pub(crate) fn can_go_back(&self) -> bool {
        self.position > 0
    }

    pub(crate) fn can_go_forward(&self) -> bool {
        self.position + 1 < self.entries.len()
    }

    fn move_to(&mut self, index: usize) -> NoteId {
        self.position = index;
        self.entries[index].clone()
    }
}
