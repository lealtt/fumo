use crate::constants::icon;
use crate::functions::time::time::ResetPeriod;

#[derive(Clone, Copy)]
pub enum RewardKind {
    Daily,
    Weekly,
    Monthly,
}

impl RewardKind {
    pub const ALL: [Self; 3] = [Self::Daily, Self::Weekly, Self::Monthly];

    pub fn custom_id(self) -> &'static str {
        match self {
            Self::Daily => "eco_daily",
            Self::Weekly => "eco_weekly",
            Self::Monthly => "eco_monthly",
        }
    }

    pub fn db_name(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
        }
    }

    pub fn button_label(self) -> &'static str {
        match self {
            Self::Daily => "Diária",
            Self::Weekly => "Semanal",
            Self::Monthly => "Mensal",
        }
    }

    pub fn field_title(self) -> &'static str {
        match self {
            Self::Daily => "Recompensa diária",
            Self::Weekly => "Recompensa semanal",
            Self::Monthly => "Recompensa mensal",
        }
    }

    pub fn button_emoji(self) -> &'static crate::constants::CustomEmoji {
        match self {
            Self::Daily => &icon::DOLLAR,
            Self::Weekly => &icon::GIFT,
            Self::Monthly => &icon::DIAMOND,
        }
    }

    pub fn money_range(self) -> (i64, i64) {
        match self {
            Self::Daily => (250, 400),
            Self::Weekly => (800, 1400),
            Self::Monthly => (4000, 6000),
        }
    }

    pub fn from_custom_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.custom_id() == id)
    }

    pub fn reset_period(self) -> ResetPeriod {
        match self {
            Self::Daily => ResetPeriod::Daily,
            Self::Weekly => ResetPeriod::Weekly,
            Self::Monthly => ResetPeriod::Monthly,
        }
    }
}
