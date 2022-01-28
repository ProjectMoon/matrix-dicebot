use strum::{AsRefStr, Display, EnumIter, EnumString};

#[derive(EnumString, EnumIter, AsRefStr, Display)]
pub(crate) enum GameSystem {
    ChroniclesOfDarkness,
    Changeling,
    MageTheAwakening,
    WerewolfTheForsaken,
    DeviantTheRenegades,
    MummyTheCurse,
    PrometheanTheCreated,
    CallOfCthulhu,
    DungeonsAndDragons5e,
    DungeonsAndDragons4e,
    DungeonsAndDragons35e,
    DungeonsAndDragons2e,
    DungeonsAndDragons1e,
    None,
}

impl GameSystem {}
