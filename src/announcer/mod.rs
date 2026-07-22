use bevy::prelude::*;

use crate::audio_fx::VoKind;

#[derive(Resource, Debug, Default)]
pub struct AnnouncerQueue {
    pub pending: Vec<String>,
    pub last_bark: String,
}

impl AnnouncerQueue {
    pub fn push(&mut self, line: impl Into<String>) {
        let line = line.into();
        self.last_bark = line.clone();
        self.pending.push(line);
    }

    pub fn push_with_vo(&mut self, vo: &mut crate::audio_fx::VoQueue, kind: VoKind, line: impl Into<String>) {
        self.push(line);
        vo.push(kind);
    }
}

pub struct AnnouncerPlugin;

impl Plugin for AnnouncerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnnouncerQueue>();
    }
}
