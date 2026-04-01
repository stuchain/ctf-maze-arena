use crate::maze::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DpState {
    pub cell: Cell,
    pub keys: u32,
}

impl DpState {
    pub fn initial(start: Cell) -> Self {
        Self {
            cell: start,
            keys: 0,
        }
    }

    pub fn with_key(&self, key_id: u8) -> Self {
        Self {
            cell: self.cell,
            keys: self.keys | (1 << key_id),
        }
    }

    pub fn has_key(&self, key_id: u8) -> bool {
        (self.keys & (1 << key_id)) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::DpState;
    use crate::maze::Cell;

    #[test]
    fn state_bitmask_tracks_keys() {
        let start = Cell::new(0, 0);
        let s = DpState::initial(start).with_key(0);
        assert!(s.has_key(0));
        assert!(!s.has_key(1));
    }
}
