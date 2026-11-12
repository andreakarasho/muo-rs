//! `LRReason`: why a lift was refused.
//! The rest of the player packets are `packets::player`.

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum LRReason {
    #[default]
    CannotLift,
    OutOfRange,
    OutOfSight,
    TryToSteal,
    AreHolding,
    Inspecific,
}
