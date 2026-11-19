use serde::{Deserialize, Serialize};

/// Timings (seconds) and cost of one kind of strike.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StrikeConfig {
    pub windup: f32,
    pub active: f32,
    pub recovery: f32,
    pub stamina_cost: f32,
    pub damage: f32,
}

impl Default for StrikeConfig {
    fn default() -> Self {
        Self { windup: 0.12, active: 0.14, recovery: 0.28, stamina_cost: 15.0, damage: 20.0 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StaminaConfig {
    pub max: f32,
    /// Points per second.
    pub regen: f32,
    /// Seconds without spending before regen starts.
    pub regen_delay: f32,
}

impl Default for StaminaConfig {
    fn default() -> Self {
        Self { max: 100.0, regen: 35.0, regen_delay: 0.8 }
    }
}

/// Holding attack charges a heavy strike.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChargeConfig {
    pub strike: StrikeConfig,
    /// Releasing before this many seconds is a light attack.
    pub tap_threshold: f32,
    /// Seconds held (from press) to reach full charge.
    pub full_charge: f32,
    pub min_mult: f32,
    pub max_mult: f32,
}

impl Default for ChargeConfig {
    fn default() -> Self {
        Self {
            strike: StrikeConfig { windup: 0.08, active: 0.18, recovery: 0.45, stamina_cost: 30.0, damage: 30.0 },
            tap_threshold: 0.2,
            full_charge: 1.0,
            min_mult: 1.0,
            max_mult: 2.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BlockConfig {
    /// Stamina drained per point of blocked damage.
    pub stamina_per_damage: f32,
    /// Seconds after raising the guard in which a block is a Parry.
    pub parry_window: f32,
    /// Seconds stunned after a Guard Break.
    pub guard_break_stun: f32,
    /// Seconds a parried attacker stays unbalanced (the counter window).
    pub parry_unbalance: f32,
    /// Seconds the guard must stay down before a new block can parry.
    pub parry_cooldown: f32,
}

impl Default for BlockConfig {
    fn default() -> Self {
        Self { stamina_per_damage: 1.0, parry_window: 0.15, guard_break_stun: 1.0, parry_unbalance: 1.2, parry_cooldown: 0.35 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DodgeConfig {
    pub stamina_cost: f32,
    pub duration: f32,
    /// Seconds from the start of the dodge in which strikes don't land.
    pub invulnerable: f32,
}

impl Default for DodgeConfig {
    fn default() -> Self {
        Self { stamina_cost: 20.0, duration: 0.35, invulnerable: 0.22 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CombatConfig {
    pub max_health: f32,
    /// Seconds a clean hit interrupts the one who takes it.
    pub hit_stagger: f32,
    pub stamina: StaminaConfig,
    pub light: StrikeConfig,
    pub charged: ChargeConfig,
    pub block: BlockConfig,
    pub dodge: DodgeConfig,
    pub kick: StrikeConfig,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            max_health: 100.0,
            hit_stagger: 0.25,
            stamina: StaminaConfig::default(),
            light: StrikeConfig::default(),
            charged: ChargeConfig::default(),
            block: BlockConfig::default(),
            dodge: DodgeConfig::default(),
            kick: StrikeConfig { windup: 0.15, active: 0.12, recovery: 0.35, stamina_cost: 20.0, damage: 5.0 },
        }
    }
}

impl CombatConfig {
    pub fn strike(&self, strike: crate::Strike) -> &StrikeConfig {
        match strike {
            crate::Strike::Light => &self.light,
            crate::Strike::Charged(_) => &self.charged.strike,
            crate::Strike::Kick => &self.kick,
        }
    }

    pub fn damage(&self, strike: crate::Strike) -> f32 {
        match strike {
            crate::Strike::Light => self.light.damage,
            crate::Strike::Kick => self.kick.damage,
            crate::Strike::Charged(c) => {
                let k = &self.charged;
                k.strike.damage * (k.min_mult + (k.max_mult - k.min_mult) * c)
            }
        }
    }
}
