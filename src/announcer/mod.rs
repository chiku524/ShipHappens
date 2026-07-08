use bevy::prelude::*;

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
}

pub struct AnnouncerPlugin;

impl Plugin for AnnouncerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnnouncerQueue>();
    }
}
