use crate::CombatConfig;

/// What a player (or AI) wants to do this instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Intent {
    AttackPressed,
    AttackReleased,
    BlockPressed,
    BlockReleased,
    Dodge,
    Kick,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Strike {
    Light,
    /// Charge level in `0.0..=1.0`.
    Charged(f32),
    /// Pushes and breaks guard; cannot be parried.
    Kick,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Idle,
    Charging,
    Windup(Strike),
    Active(Strike),
    Recovery(Strike),
    Blocking,
    Dodging,
    /// Interrupted by a hit.
    Staggered,
    /// After a Guard Break: cannot act or defend.
    Stunned,
    /// Parried: open to a counter, cannot act or defend.
    Unbalanced,
    Dead,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// A strike's active window just opened.
    StrikeActive(Strike),
    Hit { damage: f32 },
    /// The game should push the defender away from the attacker.
    Kicked,
    Blocked,
    Parried,
    GuardBroken,
    Died,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    phase: Phase,
    /// Time spent in the current phase.
    t: f32,
    /// How long the current phase lasts, for timed phases.
    duration: f32,
    stamina: f32,
    since_spend: f32,
    health: f32,
    /// Whether the current strike already connected.
    connected: bool,
    /// (tap_threshold, full_charge) captured when charging starts.
    charge_cfg: (f32, f32),
    /// Seconds since the guard was last lowered.
    since_block: f32,
    /// Whether the current block started with a parry window.
    can_parry: bool,
}

impl Fighter {
    pub fn new(cfg: &CombatConfig) -> Self {
        Self {
            phase: Phase::Idle,
            t: 0.0,
            duration: 0.0,
            stamina: cfg.stamina.max,
            since_spend: f32::INFINITY,
            health: cfg.max_health,
            connected: false,
            charge_cfg: (0.0, 1.0),
            since_block: f32::INFINITY,
            can_parry: false,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Seconds spent in the current phase.
    pub fn phase_time(&self) -> f32 {
        self.t
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn stamina(&self) -> f32 {
        self.stamina
    }

    /// The strike whose active window is open right now, if any.
    pub fn striking(&self) -> Option<Strike> {
        match self.phase {
            Phase::Active(s) if !self.connected => Some(s),
            _ => None,
        }
    }

    /// Charge level while charging, in `0.0..=1.0`.
    pub fn charge(&self) -> f32 {
        match self.phase {
            Phase::Charging => {
                let (tap, full) = self.charge_cfg;
                ((self.t - tap) / (full - tap)).clamp(0.0, 1.0)
            }
            _ => 0.0,
        }
    }

    pub fn act(&mut self, intent: Intent, cfg: &CombatConfig) -> Vec<Event> {
        match (intent, self.phase) {
            (Intent::AttackPressed, Phase::Idle) => {
                self.charge_cfg = (cfg.charged.tap_threshold, cfg.charged.full_charge);
                self.enter(Phase::Charging);
            }
            (Intent::AttackReleased, Phase::Charging) => {
                let strike = if self.t < cfg.charged.tap_threshold {
                    Strike::Light
                } else {
                    Strike::Charged(self.charge())
                };
                if self.spend(cfg.strike(strike).stamina_cost) {
                    self.enter(Phase::Windup(strike));
                } else {
                    self.enter(Phase::Idle);
                }
            }
            (Intent::BlockPressed, Phase::Idle | Phase::Charging) if self.stamina > 0.0 => {
                self.can_parry = self.since_block >= cfg.block.parry_cooldown;
                self.enter(Phase::Blocking);
            }
            (Intent::BlockReleased, Phase::Blocking) => self.enter(Phase::Idle),
            (Intent::Kick, Phase::Idle | Phase::Blocking) if self.spend(cfg.kick.stamina_cost) => {
                self.enter(Phase::Windup(Strike::Kick));
            }
            (Intent::Dodge, Phase::Idle | Phase::Charging | Phase::Blocking)
                if self.spend(cfg.dodge.stamina_cost) =>
            {
                self.enter_timed(Phase::Dodging, cfg.dodge.duration);
            }
            _ => {}
        }
        Vec::new()
    }

    pub fn tick(&mut self, dt: f32, cfg: &CombatConfig) -> Vec<Event> {
        self.t += dt;
        self.since_spend += dt;
        if self.phase != Phase::Blocking {
            self.since_block += dt;
        }
        if self.since_spend >= cfg.stamina.regen_delay {
            self.stamina = (self.stamina + cfg.stamina.regen * dt).min(cfg.stamina.max);
        }
        match self.phase {
            Phase::Windup(k) if self.t >= cfg.strike(k).windup => {
                self.enter(Phase::Active(k));
                return vec![Event::StrikeActive(k)];
            }
            Phase::Active(k) if self.t >= cfg.strike(k).active => self.enter(Phase::Recovery(k)),
            Phase::Recovery(k) if self.t >= cfg.strike(k).recovery => self.enter(Phase::Idle),
            Phase::Staggered | Phase::Stunned | Phase::Unbalanced | Phase::Dodging if self.t >= self.duration => self.enter(Phase::Idle),
            _ => {}
        }
        Vec::new()
    }

    /// Applies damage from any source (strike, spikes, a fall).
    pub fn take_damage(&mut self, damage: f32, cfg: &CombatConfig) -> Vec<Event> {
        if self.phase == Phase::Dead {
            return Vec::new();
        }
        self.health -= damage;
        let mut out = vec![Event::Hit { damage }];
        if self.health <= 0.0 {
            self.health = 0.0;
            self.enter(Phase::Dead);
            out.push(Event::Died);
        } else if !matches!(self.phase, Phase::Stunned | Phase::Unbalanced) {
            // Stun and unbalance are openings: hits land without ending them.
            self.enter_timed(Phase::Staggered, cfg.hit_stagger);
        }
        out
    }

    /// Spends stamina if there is enough; returns whether it did.
    fn spend(&mut self, cost: f32) -> bool {
        if self.stamina < cost {
            return false;
        }
        self.stamina -= cost;
        self.since_spend = 0.0;
        true
    }

    /// Drains stamina down to zero; returns whether any was left after.
    fn drain(&mut self, amount: f32) -> bool {
        self.stamina = (self.stamina - amount).max(0.0);
        self.since_spend = 0.0;
        self.stamina > 0.0
    }

    fn guard_break(&mut self, cfg: &CombatConfig) -> Vec<Event> {
        self.enter_timed(Phase::Stunned, cfg.block.guard_break_stun);
        vec![Event::GuardBroken]
    }

    fn enter(&mut self, phase: Phase) {
        if self.phase == Phase::Blocking && phase != Phase::Blocking {
            self.since_block = 0.0;
        }
        if matches!(phase, Phase::Windup(_)) {
            self.connected = false;
        }
        self.phase = phase;
        self.t = 0.0;
    }

    fn enter_timed(&mut self, phase: Phase, duration: f32) {
        self.enter(phase);
        self.duration = duration;
    }
}

/// Resolves the attacker's blade touching the defender. Call it whenever the
/// game detects contact; it does nothing outside the active window and
/// connects at most once per strike.
pub fn exchange(attacker: &mut Fighter, defender: &mut Fighter, cfg: &CombatConfig) -> Vec<Event> {
    let Some(strike) = attacker.striking() else {
        return Vec::new();
    };
    if defender.phase == Phase::Dead {
        return Vec::new();
    }
    if defender.phase == Phase::Dodging && defender.t < cfg.dodge.invulnerable {
        // The blade passes through; it may still land once invulnerability ends.
        return Vec::new();
    }
    attacker.connected = true;
    let damage = cfg.damage(strike);
    if strike == Strike::Kick {
        let mut out = vec![Event::Kicked];
        if defender.phase == Phase::Blocking {
            out.extend(defender.guard_break(cfg));
        } else {
            out.extend(defender.take_damage(damage, cfg));
        }
        return out;
    }
    if defender.phase == Phase::Blocking {
        if defender.can_parry && defender.t < cfg.block.parry_window {
            attacker.enter_timed(Phase::Unbalanced, cfg.block.parry_unbalance);
            return vec![Event::Parried];
        }
        if matches!(strike, Strike::Charged(_)) {
            return defender.guard_break(cfg);
        }
        if defender.drain(damage * cfg.block.stamina_per_damage) {
            return vec![Event::Blocked];
        }
        return defender.guard_break(cfg);
    }
    defender.take_damage(damage, cfg)
}
