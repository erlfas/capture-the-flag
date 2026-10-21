use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Team {
    Blue,
    Red,
}

impl Team {
    pub fn name(&self) -> &'static str {
        match self {
            Team::Blue => "BLÅ",
            Team::Red => "RØD",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Team::Blue => Color::srgb(0.12, 0.55, 1.0),
            Team::Red => Color::srgb(0.95, 0.20, 0.18),
        }
    }

    pub fn emissive(&self) -> LinearRgba {
        match self {
            Team::Blue => LinearRgba::new(0.5, 3.5, 7.0, 1.0),
            Team::Red => LinearRgba::new(7.0, 0.8, 0.8, 1.0),
        }
    }
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone)]
pub struct Character {
    pub team: Team,
    pub name: String,
    pub is_player: bool,
    pub spawn_point: Vec3,
}

#[derive(Component, Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub regen: f32,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            regen: 25.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone, Default)]
pub struct TackleAbility {
    pub cooldown: f32,
    pub max_cooldown: f32,
    pub is_tackling: bool,
    pub tackle_timer: f32,
}

#[derive(Component, Debug, Clone, Default)]
pub struct StaggerState {
    pub duration: f32,
}
