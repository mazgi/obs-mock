use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

pub struct Scene {
    pub name: String,
    pub uuid: String,
}

pub struct Input {
    pub name: String,
    pub uuid: String,
    pub kind: String,
    pub muted: bool,
    pub volume_mul: f64,
    pub volume_db: f64,
    pub settings: Value,
}

pub struct ObsState {
    pub scenes: Vec<Scene>,
    pub current_program_scene: usize,
    pub current_preview_scene: usize,
    pub inputs: Vec<Input>,
    pub streaming: bool,
    pub stream_timecode: String,
    pub recording: bool,
    pub record_paused: bool,
    pub record_timecode: String,
    pub virtual_cam: bool,
    pub replay_buffer: bool,
    pub studio_mode: bool,
    pub current_transition: String,
    pub transition_duration: u32,
    pub scene_collections: Vec<String>,
    pub current_scene_collection: String,
    pub profiles: Vec<String>,
    pub current_profile: String,
    pub persistent_data: HashMap<String, HashMap<String, Value>>,
}

impl ObsState {
    pub fn new() -> Self {
        Self {
            scenes: vec![
                Scene {
                    name: "Scene".to_string(),
                    uuid: Uuid::new_v4().to_string(),
                },
                Scene {
                    name: "Scene 2".to_string(),
                    uuid: Uuid::new_v4().to_string(),
                },
            ],
            current_program_scene: 0,
            current_preview_scene: 0,
            inputs: vec![
                Input {
                    name: "Desktop Audio".to_string(),
                    uuid: Uuid::new_v4().to_string(),
                    kind: "pulse_output_capture".to_string(),
                    muted: false,
                    volume_mul: 1.0,
                    volume_db: 0.0,
                    settings: json!({}),
                },
                Input {
                    name: "Mic/Aux".to_string(),
                    uuid: Uuid::new_v4().to_string(),
                    kind: "pulse_input_capture".to_string(),
                    muted: false,
                    volume_mul: 1.0,
                    volume_db: 0.0,
                    settings: json!({}),
                },
            ],
            streaming: false,
            stream_timecode: "00:00:00.000".to_string(),
            recording: false,
            record_paused: false,
            record_timecode: "00:00:00.000".to_string(),
            virtual_cam: false,
            replay_buffer: false,
            studio_mode: false,
            current_transition: "Fade".to_string(),
            transition_duration: 300,
            scene_collections: vec!["Default".to_string()],
            current_scene_collection: "Default".to_string(),
            profiles: vec!["Default".to_string()],
            current_profile: "Default".to_string(),
            persistent_data: HashMap::new(),
        }
    }

    pub fn find_scene_by_name(&self, name: &str) -> Option<usize> {
        self.scenes.iter().position(|s| s.name == name)
    }

    pub fn find_scene_by_uuid(&self, uuid: &str) -> Option<usize> {
        self.scenes.iter().position(|s| s.uuid == uuid)
    }

    pub fn find_input_by_name(&self, name: &str) -> Option<usize> {
        self.inputs.iter().position(|i| i.name == name)
    }

    pub fn find_input_by_uuid(&self, uuid: &str) -> Option<usize> {
        self.inputs.iter().position(|i| i.uuid == uuid)
    }

    pub fn resolve_scene(&self, data: &Value) -> Option<usize> {
        if let Some(name) = data.get("sceneName").and_then(|v| v.as_str()) {
            return self.find_scene_by_name(name);
        }
        if let Some(uuid) = data.get("sceneUuid").and_then(|v| v.as_str()) {
            return self.find_scene_by_uuid(uuid);
        }
        None
    }

    pub fn resolve_input(&self, data: &Value) -> Option<usize> {
        if let Some(name) = data.get("inputName").and_then(|v| v.as_str()) {
            return self.find_input_by_name(name);
        }
        if let Some(uuid) = data.get("inputUuid").and_then(|v| v.as_str()) {
            return self.find_input_by_uuid(uuid);
        }
        None
    }
}
