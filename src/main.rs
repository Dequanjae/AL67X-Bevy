//! AL67X Survivor — faithful Bevy port of the Godot survivor-like.
//! Auto-fire, horde, XP, cards, boss. Colored shapes for now.

use bevy::prelude::*;
use rand::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

// ── Data ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
struct EnemyDef {
    id: String,
    hp: f32,
    speed: f32,
    damage: f32,
    radius: f32,
    xp: f32,
    weight: f32,
    min_level: i32,
    size_m: f32,
    behavior: BehaviorDef,
}

#[derive(Debug, Clone, Deserialize)]
struct BehaviorDef {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    interval: f32,
    #[serde(default)]
    trigger_range: f32,
    #[serde(default)]
    windup_sec: f32,
    #[serde(default)]
    dash_sec: f32,
    #[serde(default)]
    dash_speed_mult: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct BossDef {
    id: String,
    enemy_base: String,
    hp_mult: f32,
    patterns: Vec<PatternDef>,
    order: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct PatternDef {
    r#type: String,
    interval: f32,
    #[serde(default)]
    count: i32,
    #[serde(default)]
    proj_speed: f32,
    #[serde(default)]
    damage: f32,
    #[serde(default)]
    spread_deg: f32,
    #[serde(default)]
    windup_sec: f32,
    #[serde(default)]
    dash_speed: f32,
    #[serde(default)]
    dash_sec: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct CardDef {
    id: String,
    name: String,
    #[serde(default)]
    base: bool,
    #[serde(default)]
    weight: f32,
    effect: EffectDef,
}

#[derive(Debug, Clone, Deserialize)]
struct EffectDef {
    r#type: String,
    #[serde(default)]
    value: f32,
    #[serde(default)]
    multiplier: f32,
    #[serde(default)]
    duration_sec: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct PowerupDef {
    id: String,
    popup: String,
    weight: f32,
    r#type: String,
    #[serde(default)]
    value: f32,
    #[serde(default)]
    multiplier: f32,
    #[serde(default)]
    duration_sec: f32,
    #[serde(default)]
    count: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct RunConfig {
    survival_timer: SurvivalTimer,
    enemy_spawn: EnemySpawn,
    player_xp: PlayerXp,
    player_base: PlayerBase,
    boss: BossConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct SurvivalTimer {
    base_seconds: f32,
    growth_rate: f32,
    growth_decay: f32,
    cap_seconds: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct EnemySpawn {
    base_interval_sec: f32,
    min_interval_sec: f32,
    level_factor: f32,
    time_factor: f32,
    level1_interval_mult: f32,
    batch_per_levels: f32,
    batch_time_sec: f32,
    batch_max: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct PlayerXp {
    base_kills_to_level: f32,
    kills_growth_per_level: f32,
    blob_xp_value: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct PlayerBase {
    max_hearts: i32,
    damage: f32,
    fire_interval_sec: f32,
    attack_range: f32,
    projectile_speed: f32,
    projectile_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct BossConfig {
    hp_multiplier_base: f32,
    hp_growth_per_level: f32,
    damage_multiplier: f32,
    speed_multiplier: f32,
    scale: f32,
    warning_seconds: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct LevelTheme {
    id: String,
    colors: HashMap<String, String>,
    generation: GenerationDef,
    spawns: SpawnDef,
    order: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct GenerationDef {
    algorithm: String,
    arena_size: [f32; 2],
    prop_density: f32,
    min_clearance: f32,
    spawn_clear_radius: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct SpawnDef {
    shawarma_max: i32,
    shawarma_respawn_sec: f32,
    powerup_interval_sec: [f32; 2],
    lootbox_chance: f32,
}

// ── Components ───────────────────────────────────────────────────────────────

#[derive(Component)]
struct Player {
    hearts: i32,
    max_hearts: i32,
    shield: i32,
    speed_mult: f32,
    damage: f32,
    damage_mult: f32,
    fire_interval: f32,
    attack_range: f32,
    projectile_count: i32,
    projectile_scale: f32,
    projectile_speed: f32,
    projectile_bounces: i32,
    projectile_pierce: i32,
    orbital_count: i32,
    nova_interval: f32,
    xp: f32,
    level: i32,
    fire_timer: f32,
    iframe: f32,
    dead: bool,
}

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct EnemyProjectile;

#[derive(Clone)]
struct EnemyState {
    id: String,
    hp: f32,
    max_hp: f32,
    speed: f32,
    damage: f32,
    radius: f32,
    xp: f32,
    behavior: BehaviorDef,
    bmode: i32,
    btimer: f32,
    bdir: Vec2,
    flash: f32,
    is_boss: bool,
    boss_id: String,
    boss_patterns: Vec<PatternDef>,
    boss_dash_dir: Vec2,
    boss_dash_speed: f32,
    boss_dash_time: f32,
    boss_telegraph: bool,
}

#[derive(Clone)]
struct ProjectileState {
    dir: Vec2,
    ttl: f32,
    bounces: i32,
    pierce: i32,
    damage: f32,
    radius: f32,
    speed: f32,
}

#[derive(Clone)]
struct EnemyProjectileState {
    dir: Vec2,
    speed: f32,
    damage: f32,
    ttl: f32,
}

#[derive(Resource, Default)]
struct EnemyStates(HashMap<Entity, EnemyState>);

#[derive(Resource, Default)]
struct ProjectileStates(HashMap<Entity, ProjectileState>);

#[derive(Resource, Default)]
struct EnemyProjectileStates(HashMap<Entity, EnemyProjectileState>);

#[derive(Component)]
struct Pickup {
    kind: PickupKind,
    payload: String,
}

#[derive(Clone, Copy, PartialEq)]
enum PickupKind {
    Shawarma,
    Powerup,
    LootBox,
}

#[derive(Component)]
struct DamageNumber {
    timer: f32,
    velocity: Vec3,
}

// ── Resources ────────────────────────────────────────────────────────────────

#[derive(Resource)]
struct GameState {
    level: i32,
    time_survived: f32,
    survival_remaining: f32,
    boss_phase: bool,
    boss_warning_timer: f32,
    boss_spawned: bool,
    run_blobs: i32,
    ended: bool,
    victory: bool,
    spawn_timer: f32,
    powerup_timer: f32,
    shawarma_respawn_timer: f32,
    vacuum_timer: f32,
    freeze_timer: f32,
    orbital_angle: f32,
    orbital_hit_timer: f32,
    nova_timer: f32,
    timed_boosts: Vec<TimedBoost>,
    level_up_pending: bool,
    level_up_options: Vec<CardDef>,
    card_pool: Vec<CardDef>,
    owned_cards: Vec<String>,
    enemy_defs: Vec<EnemyDef>,
    boss_defs: Vec<BossDef>,
    powerup_defs: Vec<PowerupDef>,
    config: RunConfig,
    theme: LevelTheme,
    arena_half: Vec2,
    rng: StdRng,
}

#[derive(Clone)]
struct TimedBoost {
    boost_type: String,
    multiplier: f32,
    remaining: f32,
}

// ── Balance functions (from run_balance.gd) ──────────────────────────────────

fn survival_seconds(level: i32, config: &RunConfig) -> f32 {
    let cfg = &config.survival_timer;
    let mut seconds = cfg.base_seconds;
    for i in 0..(level - 1).max(0) {
        seconds *= 1.0 + cfg.growth_rate * cfg.growth_decay.powi(i);
    }
    seconds.min(cfg.cap_seconds)
}

fn spawn_interval(level: i32, time_survived: f32, config: &RunConfig) -> f32 {
    let cfg = &config.enemy_spawn;
    let mut interval = (cfg.base_interval_sec
        / (1.0 + cfg.level_factor * (level - 1) as f32 + cfg.time_factor * time_survived))
        .max(cfg.min_interval_sec);
    if level <= 1 {
        interval *= cfg.level1_interval_mult;
    }
    interval
}

fn spawn_batch(level: i32, time_survived: f32, config: &RunConfig) -> i32 {
    let cfg = &config.enemy_spawn;
    if level <= 1 {
        return 1;
    }
    let per_levels = cfg.batch_per_levels.max(0.5);
    let time_sec = cfg.batch_time_sec.max(10.0);
    let batch = 1 + ((level - 1) as f32 / per_levels) as i32 + (time_survived / time_sec) as i32;
    batch.clamp(1, cfg.batch_max)
}

fn xp_threshold(level: i32, config: &RunConfig) -> f32 {
    let cfg = &config.player_xp;
    (cfg.base_kills_to_level * cfg.kills_growth_per_level.powi(level)).ceil()
}

// ── Setup ────────────────────────────────────────────────────────────────────

fn load_json<T: for<'de> Deserialize<'de>>(path: &str) -> T {
    let data = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path));
    serde_json::from_str(&data).unwrap_or_else(|e| panic!("Failed to parse {}: {}", path, e))
}

fn main() {
    let config: RunConfig = load_json("data/balance/run_curves.json");

    let mut all_enemies = Vec::new();
    let enemy_dir = std::fs::read_dir("data/enemies").unwrap();
    for entry in enemy_dir {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(def) = serde_json::from_str::<EnemyDef>(&std::fs::read_to_string(&path).unwrap()) {
                all_enemies.push(def);
            }
        }
    }

    let mut all_bosses = Vec::new();
    let boss_dir = std::fs::read_dir("data/bosses").unwrap();
    for entry in boss_dir {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(def) = serde_json::from_str::<BossDef>(&std::fs::read_to_string(&path).unwrap()) {
                all_bosses.push(def);
            }
        }
    }
    all_bosses.sort_by_key(|b| b.order);

    let mut all_cards = Vec::new();
    for rarity in &["common", "rare", "epic", "legendary"] {
        let card_dir = format!("data/cards/{}", rarity);
        if let Ok(dir) = std::fs::read_dir(&card_dir) {
            for entry in dir {
                let path = entry.unwrap().path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(def) = serde_json::from_str::<CardDef>(&std::fs::read_to_string(&path).unwrap()) {
                        all_cards.push(def);
                    }
                }
            }
        }
    }

    let powerup_raw: serde_json::Value = load_json("data/powerups/powerups.json");
    let powerup_defs: Vec<PowerupDef> = serde_json::from_value(powerup_raw["effects"].clone()).unwrap();

    let theme: LevelTheme = load_json("data/levels/theme_shop_classic.json");

    let arena_half = Vec2::new(theme.generation.arena_size[0] * 0.5, theme.generation.arena_size[1] * 0.5);
    let survival = survival_seconds(1, &config);

    let mut rng = StdRng::seed_from_u64(42);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(EnemyStates::default())
        .insert_resource(ProjectileStates::default())
        .insert_resource(EnemyProjectileStates::default())
        .insert_resource(GameState {
            level: 1,
            time_survived: 0.0,
            survival_remaining: survival,
            boss_phase: false,
            boss_warning_timer: -1.0,
            boss_spawned: false,
            run_blobs: 0,
            ended: false,
            victory: false,
            spawn_timer: 1.5,
            powerup_timer: 20.0,
            shawarma_respawn_timer: 0.0,
            vacuum_timer: 0.0,
            freeze_timer: 0.0,
            orbital_angle: 0.0,
            orbital_hit_timer: 0.0,
            nova_timer: 0.0,
            timed_boosts: Vec::new(),
            level_up_pending: false,
            level_up_options: Vec::new(),
            card_pool: all_cards.clone(),
            owned_cards: Vec::new(),
            enemy_defs: all_enemies.clone(),
            boss_defs: all_bosses.clone(),
            powerup_defs: powerup_defs.clone(),
            config: config.clone(),
            theme: theme.clone(),
            arena_half,
            rng,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                auto_fire,
                enemy_and_projectile_update,
                pickup_update,
                xp_and_leveling,
                boss_logic,
                powerup_spawn,
                shawarma_spawn,
                orbital_update,
                nova_update,
                timed_boosts_tick,
                damage_numbers_update,
                camera_follow,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, mut state: ResMut<GameState>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.3, 0.7, 1.0),
            Vec2::splat(1.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player,
    ));

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.29, 0.31, 0.34),
            Vec2::splat(state.arena_half.x * 2.0),
        ),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    let wall_color = Color::srgb(0.42, 0.35, 0.27);
    let wall_thickness = 1.0;
    let wall_height = 2.2;
    let half = state.arena_half;
    for (pos, size) in [
        (Vec3::new(0.0, wall_height * 0.5, -half.y - wall_thickness * 0.5), Vec3::new(half.x * 2.0 + wall_thickness * 2.0, wall_height, wall_thickness)),
        (Vec3::new(0.0, wall_height * 0.5, half.y + wall_thickness * 0.5), Vec3::new(half.x * 2.0 + wall_thickness * 2.0, wall_height, wall_thickness)),
        (Vec3::new(-half.x - wall_thickness * 0.5, wall_height * 0.5, 0.0), Vec3::new(wall_thickness, wall_height, half.y * 2.0)),
        (Vec3::new(half.x + wall_thickness * 0.5, wall_height * 0.5, 0.0), Vec3::new(wall_thickness, wall_height, half.y * 2.0)),
    ] {
        commands.spawn((
            Sprite::from_color(wall_color, Vec2::new(size.x, size.z)),
            Transform::from_xyz(pos.x, pos.y, pos.z),
        ));
    }

    for _ in 0..state.theme.spawns.shawarma_max {
        spawn_shawarma(&mut commands, &mut state);
    }
}

// ── Systems ──────────────────────────────────────────────────────────────────

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Player)>,
    time: Res<Time>,
) {
    let (mut transform, mut player) = query.single_mut().unwrap();
    if player.dead {
        return;
    }

    let mut dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }

    if dir != Vec2::ZERO {
        dir = dir.normalize();
        let speed = 7.0 * player.speed_mult;
        transform.translation.x += dir.x * speed * time.delta_secs();
        transform.translation.z += dir.y * speed * time.delta_secs();
    }

    player.fire_timer -= time.delta_secs();
    player.iframe -= time.delta_secs();
}

fn auto_fire(
    mut commands: Commands,
    mut player_q: Query<(&Transform, &mut Player)>,
    enemy_q: Query<(&Transform, &Enemy), Without<Projectile>>,
    state: Res<GameState>,
) {
    let (player_transform, mut player) = player_q.single_mut().unwrap();
    if player.dead || player.fire_timer > 0.0 {
        return;
    }

    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.z);

    let mut nearest: Option<(Vec2, f32)> = None;
    for (transform, enemy) in &enemy_q {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let dist = pos.distance(player_pos);
        if dist < player.attack_range {
            if nearest.is_none() || dist < nearest.unwrap().1 {
                nearest = Some((pos, dist));
            }
        }
    }

    if let Some((target, _)) = nearest {
        player.fire_timer = player.fire_interval;
        let base_dir = (target - player_pos).normalize();
        let count = player.projectile_count;
        for i in 0..count {
            let spread = (i as f32 - (count - 1) as f32 * 0.5).to_radians() * 10.0;
            let angle = base_dir.y.atan2(base_dir.x);
            let dir = Vec2::from_angle(angle + spread);
            commands.spawn((
                Sprite::from_color(Color::srgb(0.95, 1.0, 0.98), Vec2::splat(0.55)),
                Transform::from_xyz(player_pos.x, 0.8, player_pos.y),
                Projectile {
                    dir,
                    ttl: 2.0,
                    bounces: player.projectile_bounces,
                    pierce: player.projectile_pierce,
                    damage: player.damage * player.damage_mult,
                    radius: 0.3 * player.projectile_scale,
                    speed: player.projectile_speed,
                },
            ));
        }
    }
}

fn enemy_and_projectile_update(
    mut commands: Commands,
    enemy_q: Query<(Entity, &Transform), With<Enemy>>,
    proj_q: Query<(Entity, &Transform), With<Projectile>>,
    eproj_q: Query<(Entity, &Transform), With<EnemyProjectile>>,
    mut enemy_states: ResMut<EnemyStates>,
    mut proj_states: ResMut<ProjectileStates>,
    mut eproj_states: ResMut<EnemyProjectileStates>,
    mut player_q: Query<(&Transform, &mut Player)>,
    state: Res<GameState>,
    time: Res<Time>,
) {
    let (player_transform, mut player) = player_q.single_mut().unwrap();
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.z);
    let delta = time.delta_secs();

    // Enemy AI
    for (entity, transform) in &enemy_q {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let mut enemy = match enemy_states.0.get(&entity) {
            Some(e) => e.clone(),
            None => continue,
        };
        let mut vel = Vec2::ZERO;

        if enemy.is_boss && enemy.boss_dash_time > 0.0 {
            vel = enemy.boss_dash_dir * enemy.boss_dash_speed;
        } else if enemy.is_boss && enemy.boss_telegraph {
            vel = Vec2::ZERO;
        } else if enemy.behavior.r#type == "charge" && !enemy.is_boss {
            enemy.btimer -= delta;
            match enemy.bmode {
                1 => {
                    if enemy.btimer <= 0.0 {
                        enemy.bmode = 2;
                        enemy.btimer = enemy.behavior.dash_sec;
                        let to_player = player_pos - pos;
                        enemy.bdir = if to_player.length() > 0.01 {
                            to_player.normalize()
                        } else {
                            Vec2::X
                        };
                    }
                }
                2 => {
                    vel = enemy.bdir * enemy.speed * enemy.behavior.dash_speed_mult;
                    if enemy.btimer <= 0.0 {
                        enemy.bmode = 0;
                        enemy.btimer = enemy.behavior.interval;
                    }
                }
                _ => {
                    let to_player = player_pos - pos;
                    let dist = to_player.length();
                    if dist > 0.001 {
                        vel = to_player / dist * enemy.speed;
                    }
                    if enemy.btimer <= 0.0 && dist < enemy.behavior.trigger_range {
                        enemy.bmode = 1;
                        enemy.btimer = enemy.behavior.windup_sec;
                    }
                }
            }
        } else {
            let to_player = player_pos - pos;
            let dist = to_player.length();
            if dist > 0.001 {
                vel = to_player / dist * enemy.speed;
            }
        }

        let new_pos = pos + vel * delta;
        let clamped = Vec2::new(
            new_pos.x.clamp(-state.arena_half.x + 0.6, state.arena_half.x - 0.6),
            new_pos.y.clamp(-state.arena_half.y + 0.6, state.arena_half.y - 0.6),
        );

        enemy.flash = (enemy.flash - delta * 5.0).max(0.0);
        enemy_states.0.insert(entity, enemy);

        if !player.dead && player.iframe <= 0.0 && clamped.distance(player_pos) < enemy.radius + 0.5 {
            if player.shield > 0 {
                player.shield -= 1;
            } else {
                player.hearts -= 1;
            }
            player.iframe = 0.8;
            if player.hearts <= 0 {
                player.dead = true;
            }
        }
    }

    // Projectile update
    let mut to_despawn = Vec::new();
    for (entity, transform) in &proj_q {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let mut proj = match proj_states.0.get(&entity) {
            Some(p) => p.clone(),
            None => continue,
        };

        proj.ttl -= delta;
        if proj.ttl <= 0.0 {
            to_despawn.push(entity);
            continue;
        }

        let new_pos = pos + proj.dir * proj.speed * delta;

        let mut hit_wall = false;
        let mut normal = Vec2::ZERO;
        if new_pos.x < -state.arena_half.x + proj.radius {
            hit_wall = true;
            normal = Vec2::X;
        } else if new_pos.x > state.arena_half.x - proj.radius {
            hit_wall = true;
            normal = Vec2::NEG_X;
        } else if new_pos.y < -state.arena_half.y + proj.radius {
            hit_wall = true;
            normal = Vec2::NEG_Y;
        } else if new_pos.y > state.arena_half.y - proj.radius {
            hit_wall = true;
            normal = Vec2::Y;
        }

        if hit_wall {
            if proj.bounces > 0 {
                proj.bounces -= 1;
                proj.dir = proj.dir.reflect(normal);
            } else {
                to_despawn.push(entity);
                continue;
            }
        }

        proj_states.0.insert(entity, proj);
    }

    // Projectile-enemy collision
    {
        let mut proj_data: Vec<(Entity, Vec2, f32, f32)> = Vec::new();
        for (entity, proj) in &proj_states.0 {
            let transform = proj_q.get(*entity).unwrap().1;
            proj_data.push((*entity, Vec2::new(transform.translation.x, transform.translation.z), proj.radius, proj.damage));
        }

        let mut collision_despawn = Vec::new();
        for (proj_entity, pos, radius, damage) in &proj_data {
            for (enemy_entity, enemy) in &enemy_states.0 {
                let enemy_transform = enemy_q.get(*enemy_entity).unwrap().1;
                let enemy_pos = Vec2::new(enemy_transform.translation.x, enemy_transform.translation.z);
                if pos.distance(enemy_pos) < enemy.radius + radius {
                    if let Some(e) = enemy_states.0.get_mut(enemy_entity) {
                        e.hp -= damage;
                        e.flash = 1.0;
                        if e.hp <= 0.0 {
                            commands.entity(*enemy_entity).despawn();
                            enemy_states.0.remove(enemy_entity);
                        }
                    }
                    collision_despawn.push(*proj_entity);
                    break;
                }
            }
        }
        for entity in collision_despawn {
            commands.entity(entity).despawn();
            proj_states.0.remove(&entity);
        }
    }

    for entity in to_despawn {
        commands.entity(entity).despawn();
        proj_states.0.remove(&entity);
    }

    // Enemy projectile update
    let mut eproj_to_despawn = Vec::new();
    for (entity, transform) in &eproj_q {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let mut proj = match eproj_states.0.get(&entity) {
            Some(p) => p.clone(),
            None => continue,
        };

        proj.ttl -= delta;
        if proj.ttl <= 0.0 {
            eproj_to_despawn.push(entity);
            continue;
        }

        let new_pos = pos + proj.dir * proj.speed * delta;

        if new_pos.x < -state.arena_half.x + 0.3
            || new_pos.x > state.arena_half.x - 0.3
            || new_pos.y < -state.arena_half.y + 0.3
            || new_pos.y > state.arena_half.y - 0.3
        {
            eproj_to_despawn.push(entity);
            continue;
        }

        if new_pos.distance(player_pos) < 0.3 + 0.45 {
            if player.iframe <= 0.0 {
                if player.shield > 0 {
                    player.shield -= 1;
                } else {
                    player.hearts -= 1;
                }
                player.iframe = 0.4;
                if player.hearts <= 0 {
                    player.dead = true;
                }
            }
            eproj_to_despawn.push(entity);
        } else {
            eproj_states.0.insert(entity, proj);
        }
    }

    for entity in eproj_to_despawn {
        commands.entity(entity).despawn();
        eproj_states.0.remove(&entity);
    }
}

fn pickup_update(
    mut commands: Commands,
    mut pickup_q: Query<(Entity, &Transform, &Pickup)>,
    mut player_q: Query<(&Transform, &mut Player)>,
    mut state: ResMut<GameState>,
) {
    let (player_transform, mut player) = player_q.single_mut().unwrap();
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.z);
    let powerup_defs = state.powerup_defs.clone();
    let blob_xp = state.config.player_xp.blob_xp_value;

    for (entity, transform, pickup) in &pickup_q {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        if pos.distance(player_pos) < 1.0 {
            match pickup.kind {
                PickupKind::Shawarma => {
                    state.run_blobs += 1;
                    player.xp += blob_xp;
                }
                PickupKind::Powerup => {
                    if let Some(def) = powerup_defs.iter().find(|p| p.id == pickup.payload) {
                        apply_powerup(&mut player, def, &mut state);
                    }
                }
                PickupKind::LootBox => {}
            }
            commands.entity(entity).despawn();
        }
    }
}

fn apply_powerup(player: &mut Player, def: &PowerupDef, state: &mut GameState) {
    match def.r#type.as_str() {
        "heart_restore" => {
            player.hearts = (player.hearts + def.value as i32).min(player.max_hearts);
        }
        "damage_boost" => {
            player.damage_mult *= def.multiplier;
            state.timed_boosts.push(TimedBoost {
                boost_type: "damage".to_string(),
                multiplier: def.multiplier,
                remaining: def.duration_sec,
            });
        }
        "speed_boost" => {
            player.speed_mult *= def.multiplier;
            state.timed_boosts.push(TimedBoost {
                boost_type: "speed".to_string(),
                multiplier: def.multiplier,
                remaining: def.duration_sec,
            });
        }
        "succ" => {
            state.vacuum_timer = def.duration_sec.max(state.vacuum_timer);
        }
        "invincibility" => {
            player.iframe = def.duration_sec.max(player.iframe);
        }
        "freeze" => {
            state.freeze_timer = def.duration_sec.max(state.freeze_timer);
        }
        "goon_nova" => {}
        _ => {}
    }
}

fn xp_and_leveling(
    mut player_q: Query<&mut Player>,
    mut state: ResMut<GameState>,
) {
    let mut player = player_q.single_mut().unwrap();
    if player.dead {
        return;
    }

    let threshold = xp_threshold(player.level, &state.config);
    if player.xp >= threshold {
        player.xp -= threshold;
        player.level += 1;

        let pool: Vec<CardDef> = state
            .card_pool
            .iter()
            .filter(|c| c.base || state.owned_cards.contains(&c.id))
            .cloned()
            .collect();

        let mut options = Vec::new();
        let mut remaining = pool.clone();
        while options.len() < 3 && !remaining.is_empty() {
            let total: f32 = remaining.iter().map(|c| c.weight).sum();
            let roll = state.rng.r#gen::<f32>() * total;
            let mut acc = 0.0;
            for i in 0..remaining.len() {
                acc += remaining[i].weight;
                if roll <= acc {
                    options.push(remaining[i].clone());
                    remaining.remove(i);
                    break;
                }
            }
        }

        state.level_up_options = options;
        state.level_up_pending = true;
    }
}

fn boss_logic(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut enemy_q: Query<(&mut Transform, &mut Enemy)>,
    player_q: Query<(&Transform, &Player)>,
    time: Res<Time>,
) {
    if state.ended {
        return;
    }

    let delta = time.delta_secs();
    state.time_survived += delta;

    if !state.boss_phase {
        state.survival_remaining -= delta;
        if state.survival_remaining <= 0.0 {
            state.boss_phase = true;
            state.boss_warning_timer = state.config.boss.warning_seconds;
        }
    } else if state.boss_warning_timer > 0.0 {
        state.boss_warning_timer -= delta;
        if state.boss_warning_timer <= 0.0 && !state.boss_spawned {
            state.boss_spawned = true;
            let (player_transform, _) = player_q.single().unwrap();
            let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.z);

            let boss_index = ((state.level - 1) as usize) % state.boss_defs.len().max(1);
            let boss_def = &state.boss_defs[boss_index];

            let base_enemy = state
                .enemy_defs
                .iter()
                .find(|e| e.id == boss_def.enemy_base)
                .or_else(|| state.enemy_defs.first())
                .cloned()
                .unwrap_or(EnemyDef {
                    id: "fallback".to_string(),
                    hp: 100.0,
                    speed: 2.0,
                    damage: 10.0,
                    radius: 0.5,
                    xp: 10.0,
                    weight: 1.0,
                    min_level: 1,
                    size_m: 1.4,
                    behavior: BehaviorDef {
                        r#type: "chase".to_string(),
                        interval: 3.0,
                        trigger_range: 8.0,
                        windup_sec: 0.7,
                        dash_sec: 0.5,
                        dash_speed_mult: 4.0,
                    },
                });

            let hp = base_enemy.hp
                * state.config.boss.hp_multiplier_base
                * (1.0 + state.config.boss.hp_growth_per_level * (state.level - 1) as f32)
                * boss_def.hp_mult;

            let spawn_pos = Vec2::new(player_pos.x + 15.0, player_pos.y + 15.0);

            commands.spawn((
                Sprite::from_color(
                    Color::srgb(1.0, 0.3, 0.3),
                    Vec2::splat(base_enemy.size_m * state.config.boss.scale),
                ),
                Transform::from_xyz(spawn_pos.x, 0.5, spawn_pos.y),
                Enemy {
                    id: base_enemy.id.clone(),
                    hp,
                    max_hp: hp,
                    speed: base_enemy.speed * state.config.boss.speed_multiplier,
                    damage: base_enemy.damage * state.config.boss.damage_multiplier,
                    radius: base_enemy.radius * state.config.boss.scale,
                    xp: base_enemy.xp,
                    behavior: base_enemy.behavior.clone(),
                    bmode: 0,
                    btimer: 0.0,
                    bdir: Vec2::ZERO,
                    flash: 0.0,
                    is_boss: true,
                    boss_id: boss_def.id.clone(),
                    boss_patterns: boss_def.patterns.clone(),
                    boss_dash_dir: Vec2::ZERO,
                    boss_dash_speed: 0.0,
                    boss_dash_time: 0.0,
                    boss_telegraph: false,
                },
            ));
        }
    }
}

fn powerup_spawn(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    time: Res<Time>,
) {
    if state.ended {
        return;
    }
    let delta = time.delta_secs();
    state.powerup_timer -= delta;
    if state.powerup_timer <= 0.0 {
        let interval = state.theme.spawns.powerup_interval_sec;
        state.powerup_timer = state.rng.r#gen_range(interval[0]..interval[1]);

        let total: f32 = state.powerup_defs.iter().map(|p| p.weight).sum();
        let roll = state.rng.r#gen::<f32>() * total;
        let mut acc = 0.0;
        let mut chosen: Option<String> = None;
        for def in &state.powerup_defs {
            acc += def.weight;
            if roll <= acc {
                chosen = Some(def.id.clone());
                break;
            }
        }
        if let Some(id) = chosen {
            let half = state.arena_half;
            let pos = Vec2::new(
                state.rng.r#gen_range(-half.x + 2.0..half.x - 2.0),
                state.rng.r#gen_range(-half.y + 2.0..half.y - 2.0),
            );
            commands.spawn((
                Sprite::from_color(Color::srgb(1.0, 0.8, 0.2), Vec2::splat(0.65)),
                Transform::from_xyz(pos.x, 0.65, pos.y),
                Pickup {
                    kind: PickupKind::Powerup,
                    payload: id,
                },
            ));
        }
    }
}

fn shawarma_spawn(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    time: Res<Time>,
) {
    if state.ended {
        return;
    }
    let delta = time.delta_secs();
    state.shawarma_respawn_timer -= delta;
    if state.shawarma_respawn_timer <= 0.0 {
        state.shawarma_respawn_timer = state.theme.spawns.shawarma_respawn_sec;
        spawn_shawarma(&mut commands, &mut state);
    }
}

fn spawn_shawarma(commands: &mut Commands, state: &mut GameState) {
    let half = state.arena_half;
    let pos = Vec2::new(
        state.rng.r#gen_range(-half.x + 1.0..half.x - 1.0),
        state.rng.r#gen_range(-half.y + 1.0..half.y - 1.0),
    );
    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.7, 0.3), Vec2::splat(0.5)),
        Transform::from_xyz(pos.x, 0.5, pos.y),
        Pickup {
            kind: PickupKind::Shawarma,
            payload: String::new(),
        },
    ));
}

fn orbital_update(
    mut player_q: Query<(&Transform, &Player)>,
    mut state: ResMut<GameState>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let (player_transform, player) = player_q.single().unwrap();
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.z);

    state.orbital_angle += delta * 2.4;
    state.orbital_hit_timer -= delta;

    let count = player.orbital_count.min(12);
    if count > 0 && state.orbital_hit_timer <= 0.0 {
        state.orbital_hit_timer = 0.28;
        for j in 0..count {
            let angle = state.orbital_angle + std::f32::consts::TAU * j as f32 / count as f32;
            let _p = player_pos + Vec2::from_angle(angle) * 2.3;
        }
    }
}

fn nova_update(
    mut commands: Commands,
    mut player_q: Query<(&Transform, &Player)>,
    mut state: ResMut<GameState>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let (player_transform, player) = player_q.single().unwrap();
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.z);

    if player.nova_interval > 0.0 {
        state.nova_timer -= delta;
        if state.nova_timer <= 0.0 {
            state.nova_timer = player.nova_interval;
            for i in 0..20 {
                let angle = std::f32::consts::TAU * i as f32 / 20.0;
                commands.spawn((
                    Sprite::from_color(Color::srgb(0.95, 1.0, 0.98), Vec2::splat(0.55)),
                    Transform::from_xyz(player_pos.x, 0.8, player_pos.y),
                    Projectile {
                        dir: Vec2::from_angle(angle),
                        ttl: 2.0,
                        bounces: 0,
                        pierce: 0,
                        damage: player.damage * player.damage_mult,
                        radius: 0.3,
                        speed: player.projectile_speed,
                    },
                ));
            }
        }
    }
}

fn timed_boosts_tick(mut state: ResMut<GameState>, time: Res<Time>) {
    let delta = time.delta_secs();
    let mut i = 0;
    while i < state.timed_boosts.len() {
        state.timed_boosts[i].remaining -= delta;
        if state.timed_boosts[i].remaining <= 0.0 {
            state.timed_boosts.remove(i);
        } else {
            i += 1;
        }
    }
}

fn damage_numbers_update(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Transform, &mut DamageNumber)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut dn) in &mut q {
        dn.timer -= delta;
        if dn.timer <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation += dn.velocity * delta;
        }
    }
}

fn camera_follow(
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
    player_q: Query<&Transform, (With<Player>, Without<Camera2d>)>,
) {
    let mut camera = camera_q.single_mut().unwrap();
    let player_transform = player_q.single().unwrap();
    camera.translation.x = player_transform.translation.x;
    camera.translation.y = player_transform.translation.y;
}
