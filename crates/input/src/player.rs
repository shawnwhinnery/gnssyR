/// Identifies a local player (up to 4 for couch co-op).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub const P1: PlayerId = PlayerId(0);
    pub const P2: PlayerId = PlayerId(1);
    pub const P3: PlayerId = PlayerId(2);
    pub const P4: PlayerId = PlayerId(3);
}
