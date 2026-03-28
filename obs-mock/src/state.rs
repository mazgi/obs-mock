use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

pub struct SceneItem {
    pub id: u32,
    pub source_name: String,
    pub source_uuid: String,
    pub source_kind: String,
    pub enabled: bool,
    pub locked: bool,
    pub blend_mode: String,
    pub transform: SceneItemTransform,
}

pub struct SceneItemTransform {
    pub position_x: f64,
    pub position_y: f64,
    pub rotation: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub width: f64,
    pub height: f64,
    pub source_width: f64,
    pub source_height: f64,
}

pub struct Scene {
    pub name: String,
    pub uuid: String,
    pub items: Vec<SceneItem>,
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
    fn make_item(id: u32, name: &str, kind: &str, x: f64, y: f64, w: f64, h: f64) -> SceneItem {
        SceneItem {
            id,
            source_name: name.to_string(),
            source_uuid: Uuid::new_v4().to_string(),
            source_kind: kind.to_string(),
            enabled: true,
            locked: false,
            blend_mode: "OBS_BLEND_NORMAL".to_string(),
            transform: SceneItemTransform {
                position_x: x,
                position_y: y,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                width: w,
                height: h,
                source_width: w,
                source_height: h,
            },
        }
    }

    pub fn new() -> Self {
        let mut next_id: u32 = 1;
        let mut id = || {
            let v = next_id;
            next_id += 1;
            v
        };

        Self {
            scenes: vec![
                Scene {
                    name: "Main".to_string(),
                    uuid: Uuid::new_v4().to_string(),
                    items: vec![
                        Self::make_item(id(), "Camera", "v4l2_input", 0.0, 0.0, 1920.0, 1080.0),
                        Self::make_item(id(), "Microphone", "pulse_input_capture", 0.0, 0.0, 0.0, 0.0),
                        Self::make_item(id(), "Desktop Audio", "pulse_output_capture", 0.0, 0.0, 0.0, 0.0),
                    ],
                },
                Scene {
                    name: "Screen Share".to_string(),
                    uuid: Uuid::new_v4().to_string(),
                    items: vec![
                        Self::make_item(id(), "Screen Capture", "monitor_capture", 0.0, 0.0, 1920.0, 1080.0),
                        Self::make_item(id(), "Webcam Overlay", "v4l2_input", 1580.0, 820.0, 320.0, 240.0),
                        Self::make_item(id(), "Desktop Audio", "pulse_output_capture", 0.0, 0.0, 0.0, 0.0),
                    ],
                },
                Scene {
                    name: "BRB".to_string(),
                    uuid: Uuid::new_v4().to_string(),
                    items: vec![
                        Self::make_item(id(), "BRB Image", "image_source", 0.0, 0.0, 1920.0, 1080.0),
                        Self::make_item(id(), "Background Music", "ffmpeg_source", 0.0, 0.0, 0.0, 0.0),
                    ],
                },
                Scene {
                    name: "Starting Soon".to_string(),
                    uuid: Uuid::new_v4().to_string(),
                    items: vec![
                        Self::make_item(id(), "Starting Soon Image", "image_source", 0.0, 0.0, 1920.0, 1080.0),
                        Self::make_item(id(), "Countdown Timer", "browser_source", 660.0, 600.0, 600.0, 200.0),
                        Self::make_item(id(), "Background Music", "ffmpeg_source", 0.0, 0.0, 0.0, 0.0),
                    ],
                },
                Scene {
                    name: "Ending".to_string(),
                    uuid: Uuid::new_v4().to_string(),
                    items: vec![
                        Self::make_item(id(), "Ending Image", "image_source", 0.0, 0.0, 1920.0, 1080.0),
                        Self::make_item(id(), "Background Music", "ffmpeg_source", 0.0, 0.0, 0.0, 0.0),
                    ],
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

    pub fn find_scene_item(&self, scene_idx: usize, item_id: u32) -> Option<usize> {
        self.scenes[scene_idx]
            .items
            .iter()
            .position(|i| i.id == item_id)
    }

    pub fn next_scene_item_id(&self) -> u32 {
        self.scenes
            .iter()
            .flat_map(|s| s.items.iter())
            .map(|i| i.id)
            .max()
            .unwrap_or(0)
            + 1
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
