use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Team {
    Blue,
    Red,
}

impl Team {
    pub fn name(&self) -> &'static str {
        match self {
            Team::Blue => "BLUE",
            Team::Red => "RED",
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

/// Structured character voice callout differentiating which character says what,
/// their team affiliation, the voice line identifier, the exact spoken transcript,
/// and the corresponding audio asset path (`voices/<speaker_id>/<line_id>.mp3`).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoiceCallout {
    pub speaker: String,
    pub speaker_team: Team,
    pub line_id: &'static str,
    pub spoken_text: String,
    pub color: Color,
    pub timer: f32,
    pub played_audio: bool,
}

impl VoiceCallout {
    pub fn new(
        speaker: impl Into<String>,
        speaker_team: Team,
        line_id: &'static str,
        spoken_text: impl Into<String>,
        color: Color,
        timer: f32,
    ) -> Self {
        Self {
            speaker: speaker.into(),
            speaker_team,
            line_id,
            spoken_text: spoken_text.into(),
            color,
            timer,
            played_audio: false,
        }
    }

    /// Relative asset path under `assets/` for this character's voice clip:
    /// e.g. `"voices/anders/match_start.mp3"` or `"voices/siri/took_flag.mp3"`.
    #[allow(dead_code)]
    pub fn audio_asset_path(&self) -> String {
        format!(
            "voices/{}/{}.mp3",
            self.speaker.to_lowercase(),
            self.line_id
        )
    }

    /// Formats the on-screen radio banner so the player can immediately see which character is speaking:
    /// e.g. `ANDERS [BLUE]: «Capture the Red Flag on the East Platform and bring it to Blue Base!»`
    pub fn formatted_banner(&self) -> String {
        format!(
            "{} [{}]: «{}»",
            self.speaker.to_uppercase(),
            self.speaker_team.name(),
            self.spoken_text
        )
    }
}
