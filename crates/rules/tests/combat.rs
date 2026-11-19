use rules::*;

fn cfg() -> CombatConfig {
    CombatConfig::default()
}

/// Advance the fighter in small steps, collecting events.
pub fn run(f: &mut Fighter, cfg: &CombatConfig, seconds: f32) -> Vec<Event> {
    let mut out = Vec::new();
    let mut t = 0.0;
    while t < seconds {
        out.extend(f.tick(0.01, cfg));
        t += 0.01;
    }
    out
}

fn light_attack(f: &mut Fighter, cfg: &CombatConfig) {
    f.act(Intent::AttackPressed, cfg);
    f.act(Intent::AttackReleased, cfg);
}

#[test]
fn light_attack_goes_windup_active_recovery_then_idle() {
    let cfg = cfg();
    let mut f = Fighter::new(&cfg);
    light_attack(&mut f, &cfg);
    assert!(matches!(f.phase(), Phase::Windup(Strike::Light)));
    assert!(f.striking().is_none());

    run(&mut f, &cfg, cfg.light.windup + 0.02);
    assert!(matches!(f.phase(), Phase::Active(Strike::Light)));
    assert!(f.striking().is_some());

    run(&mut f, &cfg, cfg.light.active);
    assert!(matches!(f.phase(), Phase::Recovery(Strike::Light)));
    assert!(f.striking().is_none());

    run(&mut f, &cfg, cfg.light.recovery);
    assert!(matches!(f.phase(), Phase::Idle));
}

#[test]
fn light_attack_spends_stamina() {
    let cfg = cfg();
    let mut f = Fighter::new(&cfg);
    light_attack(&mut f, &cfg);
    assert_eq!(f.stamina(), cfg.stamina.max - cfg.light.stamina_cost);
}

#[test]
fn stamina_regenerates_only_after_delay() {
    let cfg = cfg();
    let mut f = Fighter::new(&cfg);
    light_attack(&mut f, &cfg);
    let spent = f.stamina();
    run(&mut f, &cfg, cfg.stamina.regen_delay - 0.05);
    assert_eq!(f.stamina(), spent);
    run(&mut f, &cfg, 0.5);
    assert!(f.stamina() > spent);
    run(&mut f, &cfg, 10.0);
    assert_eq!(f.stamina(), cfg.stamina.max);
}

#[test]
fn cannot_attack_without_enough_stamina() {
    let cfg = CombatConfig { stamina: StaminaConfig { max: 10.0, ..Default::default() }, ..cfg() };
    let mut f = Fighter::new(&cfg);
    light_attack(&mut f, &cfg);
    assert_eq!(f.phase(), Phase::Idle);
    assert_eq!(f.stamina(), 10.0);
}

/// Puts `f` into the active window of a light attack.
fn swing_to_active(f: &mut Fighter, cfg: &CombatConfig) {
    light_attack(f, cfg);
    run(f, cfg, cfg.light.windup + 0.02);
}

#[test]
fn active_light_attack_hits_once_per_swing() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    swing_to_active(&mut a, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::Hit { damage: cfg.light.damage }]);
    assert_eq!(d.health(), cfg.max_health - cfg.light.damage);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![]);
}

#[test]
fn no_hit_outside_active_window() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    light_attack(&mut a, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![]);
}

#[test]
fn lethal_hit_kills() {
    let cfg = CombatConfig { max_health: 10.0, ..cfg() };
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    swing_to_active(&mut a, &cfg);
    let ev = exchange(&mut a, &mut d, &cfg);
    assert_eq!(ev, vec![Event::Hit { damage: cfg.light.damage }, Event::Died]);
    assert_eq!(d.phase(), Phase::Dead);
    light_attack(&mut d, &cfg);
    assert_eq!(d.phase(), Phase::Dead);
}

fn hold_attack(f: &mut Fighter, cfg: &CombatConfig, seconds: f32) {
    f.act(Intent::AttackPressed, cfg);
    run(f, cfg, seconds);
    f.act(Intent::AttackReleased, cfg);
}

#[test]
fn holding_attack_charges() {
    let cfg = cfg();
    let mut f = Fighter::new(&cfg);
    f.act(Intent::AttackPressed, &cfg);
    run(&mut f, &cfg, cfg.charged.tap_threshold + 0.05);
    assert_eq!(f.phase(), Phase::Charging);
    assert!(f.charge() > 0.0 && f.charge() < 1.0);
}

#[test]
fn quick_tap_is_a_light_attack() {
    let cfg = cfg();
    let mut f = Fighter::new(&cfg);
    hold_attack(&mut f, &cfg, cfg.charged.tap_threshold * 0.5);
    assert!(matches!(f.phase(), Phase::Windup(Strike::Light)));
}

#[test]
fn full_charge_hits_for_max_multiplier_and_costs_its_own_stamina() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    hold_attack(&mut a, &cfg, cfg.charged.full_charge + 0.5);
    assert!(matches!(a.phase(), Phase::Windup(Strike::Charged(c)) if c == 1.0));
    assert_eq!(a.stamina(), cfg.stamina.max - cfg.charged.strike.stamina_cost);
    run(&mut a, &cfg, cfg.charged.strike.windup + 0.02);
    let ev = exchange(&mut a, &mut d, &cfg);
    assert_eq!(ev, vec![Event::Hit { damage: cfg.charged.strike.damage * cfg.charged.max_mult }]);
}

#[test]
fn half_charge_scales_damage_between_multipliers() {
    let cfg = CombatConfig {
        charged: ChargeConfig { tap_threshold: 0.2, full_charge: 1.2, min_mult: 1.0, max_mult: 3.0, ..Default::default() },
        ..cfg()
    };
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    hold_attack(&mut a, &cfg, 0.7); // halfway between 0.2 and 1.2
    run(&mut a, &cfg, cfg.charged.strike.windup + 0.02);
    let Event::Hit { damage } = exchange(&mut a, &mut d, &cfg)[0] else { panic!() };
    assert!((damage - cfg.charged.strike.damage * 2.0).abs() < 1.0, "{damage}");
}

/// Defender raises the guard and waits out the parry window.
fn late_block(d: &mut Fighter, cfg: &CombatConfig) {
    d.act(Intent::BlockPressed, cfg);
    run(d, cfg, cfg.block.parry_window + 0.05);
}

fn charged_to_active(a: &mut Fighter, cfg: &CombatConfig) {
    hold_attack(a, cfg, cfg.charged.full_charge);
    run(a, cfg, cfg.charged.strike.windup + 0.02);
}

#[test]
fn block_stops_damage_and_spends_stamina() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    late_block(&mut d, &cfg);
    assert_eq!(d.phase(), Phase::Blocking);
    swing_to_active(&mut a, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::Blocked]);
    assert_eq!(d.health(), cfg.max_health);
    assert_eq!(d.stamina(), cfg.stamina.max - cfg.light.damage * cfg.block.stamina_per_damage);
}

#[test]
fn releasing_block_returns_to_idle() {
    let cfg = cfg();
    let mut d = Fighter::new(&cfg);
    d.act(Intent::BlockPressed, &cfg);
    d.act(Intent::BlockReleased, &cfg);
    assert_eq!(d.phase(), Phase::Idle);
}

#[test]
fn guard_breaks_when_block_runs_out_of_stamina() {
    let cfg = CombatConfig { stamina: StaminaConfig { max: 5.0, ..Default::default() }, ..cfg() };
    let a_cfg = CombatConfig::default();
    let (mut a, mut d) = (Fighter::new(&a_cfg), Fighter::new(&cfg));
    late_block(&mut d, &cfg);
    swing_to_active(&mut a, &a_cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::GuardBroken]);
    assert_eq!(d.stamina(), 0.0);
    assert_eq!(d.phase(), Phase::Stunned);
}

#[test]
fn charged_strike_breaks_block() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    late_block(&mut d, &cfg);
    charged_to_active(&mut a, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::GuardBroken]);
    assert_eq!(d.phase(), Phase::Stunned);
    assert_eq!(d.health(), cfg.max_health);
}

#[test]
fn stunned_fighter_cannot_act_and_recovers() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    late_block(&mut d, &cfg);
    charged_to_active(&mut a, &cfg);
    exchange(&mut a, &mut d, &cfg);
    d.act(Intent::BlockPressed, &cfg);
    light_attack(&mut d, &cfg);
    assert_eq!(d.phase(), Phase::Stunned);
    run(&mut d, &cfg, cfg.block.guard_break_stun + 0.02);
    assert_eq!(d.phase(), Phase::Idle);
}

#[test]
fn stunned_fighter_takes_full_hits() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    late_block(&mut d, &cfg);
    charged_to_active(&mut a, &cfg);
    exchange(&mut a, &mut d, &cfg);
    let mut b = Fighter::new(&cfg);
    swing_to_active(&mut b, &cfg);
    assert_eq!(exchange(&mut b, &mut d, &cfg), vec![Event::Hit { damage: cfg.light.damage }]);
}

#[test]
fn cannot_block_with_empty_stamina() {
    let cfg = CombatConfig { stamina: StaminaConfig { max: 0.0, ..Default::default() }, ..cfg() };
    let mut d = Fighter::new(&cfg);
    d.act(Intent::BlockPressed, &cfg);
    assert_eq!(d.phase(), Phase::Idle);
}

#[test]
fn being_hit_interrupts_the_defender() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    light_attack(&mut d, &cfg);
    swing_to_active(&mut a, &cfg);
    exchange(&mut a, &mut d, &cfg);
    assert_eq!(d.phase(), Phase::Staggered);
    run(&mut d, &cfg, cfg.hit_stagger + 0.02);
    assert_eq!(d.phase(), Phase::Idle);
}

#[test]
fn block_at_the_last_instant_parries_for_free_and_unbalances_attacker() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    swing_to_active(&mut a, &cfg);
    d.act(Intent::BlockPressed, &cfg);
    run(&mut d, &cfg, cfg.block.parry_window * 0.5);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::Parried]);
    assert_eq!(d.stamina(), cfg.stamina.max);
    assert_eq!(a.phase(), Phase::Unbalanced);
    run(&mut a, &cfg, cfg.block.parry_unbalance + 0.02);
    assert_eq!(a.phase(), Phase::Idle);
}

#[test]
fn parry_also_beats_a_charged_strike() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    charged_to_active(&mut a, &cfg);
    d.act(Intent::BlockPressed, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::Parried]);
}

#[test]
fn unbalanced_attacker_cannot_block_the_counter() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    swing_to_active(&mut a, &cfg);
    d.act(Intent::BlockPressed, &cfg);
    exchange(&mut a, &mut d, &cfg);
    a.act(Intent::BlockPressed, &cfg);
    assert_eq!(a.phase(), Phase::Unbalanced);
}

#[test]
fn dodge_spends_stamina_and_lasts_its_duration() {
    let cfg = cfg();
    let mut d = Fighter::new(&cfg);
    d.act(Intent::Dodge, &cfg);
    assert_eq!(d.phase(), Phase::Dodging);
    assert_eq!(d.stamina(), cfg.stamina.max - cfg.dodge.stamina_cost);
    run(&mut d, &cfg, cfg.dodge.duration + 0.02);
    assert_eq!(d.phase(), Phase::Idle);
}

#[test]
fn strike_during_invulnerability_does_not_land() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    swing_to_active(&mut a, &cfg);
    d.act(Intent::Dodge, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![]);
    assert_eq!(d.health(), cfg.max_health);
}

#[test]
fn strike_after_invulnerability_lands() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    d.act(Intent::Dodge, &cfg);
    run(&mut d, &cfg, cfg.dodge.invulnerable + 0.02);
    swing_to_active(&mut a, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::Hit { damage: cfg.light.damage }]);
}

#[test]
fn cannot_dodge_without_stamina() {
    let cfg = CombatConfig { stamina: StaminaConfig { max: 5.0, ..Default::default() }, ..cfg() };
    let mut d = Fighter::new(&cfg);
    d.act(Intent::Dodge, &cfg);
    assert_eq!(d.phase(), Phase::Idle);
}

fn kick_to_active(a: &mut Fighter, cfg: &CombatConfig) {
    a.act(Intent::Kick, cfg);
    run(a, cfg, cfg.kick.windup + 0.02);
}

#[test]
fn kick_costs_stamina_and_runs_its_own_timings() {
    let cfg = cfg();
    let mut a = Fighter::new(&cfg);
    a.act(Intent::Kick, &cfg);
    assert_eq!(a.phase(), Phase::Windup(Strike::Kick));
    assert_eq!(a.stamina(), cfg.stamina.max - cfg.kick.stamina_cost);
    run(&mut a, &cfg, cfg.kick.windup + 0.02);
    assert_eq!(a.striking(), Some(Strike::Kick));
}

#[test]
fn kick_breaks_a_block() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    late_block(&mut d, &cfg);
    kick_to_active(&mut a, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::Kicked, Event::GuardBroken]);
    assert_eq!(d.phase(), Phase::Stunned);
}

#[test]
fn kick_cannot_be_parried() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    kick_to_active(&mut a, &cfg);
    d.act(Intent::BlockPressed, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::Kicked, Event::GuardBroken]);
}

#[test]
fn kick_on_an_open_target_pushes_and_hurts_a_little() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    kick_to_active(&mut a, &cfg);
    assert_eq!(
        exchange(&mut a, &mut d, &cfg),
        vec![Event::Kicked, Event::Hit { damage: cfg.kick.damage }]
    );
}

#[test]
fn environment_damage_can_kill() {
    let cfg = cfg();
    let mut d = Fighter::new(&cfg);
    assert_eq!(d.take_damage(cfg.max_health, &cfg), vec![Event::Hit { damage: cfg.max_health }, Event::Died]);
    assert_eq!(d.take_damage(10.0, &cfg), vec![]);
}

#[test]
fn tick_announces_when_a_strike_becomes_active() {
    let cfg = cfg();
    let mut a = Fighter::new(&cfg);
    light_attack(&mut a, &cfg);
    let ev = run(&mut a, &cfg, cfg.light.windup + 0.02);
    assert_eq!(ev, vec![Event::StrikeActive(Strike::Light)]);
}

#[test]
fn spamming_block_does_not_keep_the_parry_window_open() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    d.act(Intent::BlockPressed, &cfg);
    run(&mut d, &cfg, 0.05);
    d.act(Intent::BlockReleased, &cfg);
    d.act(Intent::BlockPressed, &cfg);
    swing_to_active(&mut a, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::Blocked]);
}

#[test]
fn parry_window_returns_after_the_guard_rests() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    d.act(Intent::BlockPressed, &cfg);
    d.act(Intent::BlockReleased, &cfg);
    run(&mut d, &cfg, cfg.block.parry_cooldown + 0.02);
    swing_to_active(&mut a, &cfg);
    d.act(Intent::BlockPressed, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![Event::Parried]);
}

#[test]
fn counter_hit_keeps_the_parried_attacker_unbalanced() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    swing_to_active(&mut a, &cfg);
    d.act(Intent::BlockPressed, &cfg);
    exchange(&mut a, &mut d, &cfg);
    d.act(Intent::BlockReleased, &cfg);
    swing_to_active(&mut d, &cfg);
    assert_eq!(exchange(&mut d, &mut a, &cfg), vec![Event::Hit { damage: cfg.light.damage }]);
    assert_eq!(a.phase(), Phase::Unbalanced);
}

#[test]
fn strikes_pass_through_the_dead() {
    let cfg = cfg();
    let (mut a, mut d) = (Fighter::new(&cfg), Fighter::new(&cfg));
    d.take_damage(cfg.max_health, &cfg);
    swing_to_active(&mut a, &cfg);
    assert_eq!(exchange(&mut a, &mut d, &cfg), vec![]);
    assert!(a.striking().is_some(), "the strike is not spent on a corpse");
}
