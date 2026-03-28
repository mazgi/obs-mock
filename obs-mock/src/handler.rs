use serde_json::{json, Value};
use uuid::Uuid;

use crate::protocol::{RequestResponse, REQUEST_STATUS_UNKNOWN};
use crate::state::{Input, ObsState, Scene};

pub fn handle_request(
    state: &mut ObsState,
    request_type: &str,
    request_id: &str,
    data: Option<&Value>,
) -> RequestResponse {
    let empty = json!({});
    let data = data.unwrap_or(&empty);

    match request_type {
        // General
        "GetVersion" => success(request_type, request_id, json!({
            "obsVersion": "31.0.0",
            "obsWebSocketVersion": "5.5.4",
            "rpcVersion": 1,
            "availableRequests": [],
            "supportedImageFormats": ["png", "jpg", "bmp"],
            "platform": "linux",
            "platformDescription": "OBS Mock Server"
        })),

        "GetStats" => success(request_type, request_id, json!({
            "cpuUsage": 2.5,
            "memoryUsage": 512.0,
            "availableDiskSpace": 100000.0,
            "activeFps": 60.0,
            "averageFrameRenderTime": 5.0,
            "renderSkippedFrames": 0,
            "renderTotalFrames": 10000,
            "outputSkippedFrames": 0,
            "outputTotalFrames": 10000,
            "webSocketSessionIncomingMessages": 0,
            "webSocketSessionOutgoingMessages": 0
        })),

        "GetHotkeyList" => success(request_type, request_id, json!({
            "hotkeys": []
        })),

        "BroadcastCustomEvent" | "TriggerHotkeyByName" | "TriggerHotkeyByKeySequence" | "Sleep" => {
            success_empty(request_type, request_id)
        }

        "CallVendorRequest" => success(request_type, request_id, json!({
            "vendorName": data.get("vendorName").unwrap_or(&json!("")),
            "requestType": data.get("requestType").unwrap_or(&json!("")),
            "responseData": {}
        })),

        // Config
        "GetSceneCollectionList" => success(request_type, request_id, json!({
            "currentSceneCollectionName": state.current_scene_collection,
            "sceneCollections": state.scene_collections
        })),

        "SetCurrentSceneCollection" => {
            if let Some(name) = data.get("sceneCollectionName").and_then(|v| v.as_str()) {
                state.current_scene_collection = name.to_string();
            }
            success_empty(request_type, request_id)
        }

        "CreateSceneCollection" => {
            if let Some(name) = data.get("sceneCollectionName").and_then(|v| v.as_str()) {
                state.scene_collections.push(name.to_string());
                state.current_scene_collection = name.to_string();
            }
            success_empty(request_type, request_id)
        }

        "GetProfileList" => success(request_type, request_id, json!({
            "currentProfileName": state.current_profile,
            "profiles": state.profiles
        })),

        "SetCurrentProfile" => {
            if let Some(name) = data.get("profileName").and_then(|v| v.as_str()) {
                state.current_profile = name.to_string();
            }
            success_empty(request_type, request_id)
        }

        "CreateProfile" => {
            if let Some(name) = data.get("profileName").and_then(|v| v.as_str()) {
                state.profiles.push(name.to_string());
            }
            success_empty(request_type, request_id)
        }

        "RemoveProfile" => {
            if let Some(name) = data.get("profileName").and_then(|v| v.as_str()) {
                state.profiles.retain(|p| p != name);
            }
            success_empty(request_type, request_id)
        }

        "GetProfileParameter" => success(request_type, request_id, json!({
            "parameterValue": null,
            "defaultParameterValue": null
        })),

        "SetProfileParameter" => success_empty(request_type, request_id),

        "GetVideoSettings" => success(request_type, request_id, json!({
            "fpsNumerator": 60,
            "fpsDenominator": 1,
            "baseWidth": 1920,
            "baseHeight": 1080,
            "outputWidth": 1920,
            "outputHeight": 1080
        })),

        "SetVideoSettings" => success_empty(request_type, request_id),

        "GetStreamServiceSettings" => success(request_type, request_id, json!({
            "streamServiceType": "rtmp_common",
            "streamServiceSettings": {
                "server": "rtmp://localhost/live",
                "key": "mock-stream-key"
            }
        })),

        "SetStreamServiceSettings" => success_empty(request_type, request_id),

        "GetRecordDirectory" => success(request_type, request_id, json!({
            "recordDirectory": "/tmp/obs-recordings"
        })),

        "SetRecordDirectory" => success_empty(request_type, request_id),

        "GetPersistentData" => {
            let realm = data.get("realm").and_then(|v| v.as_str()).unwrap_or("");
            let slot = data.get("slotName").and_then(|v| v.as_str()).unwrap_or("");
            let value = state
                .persistent_data
                .get(realm)
                .and_then(|m| m.get(slot))
                .cloned()
                .unwrap_or(Value::Null);
            success(request_type, request_id, json!({ "slotValue": value }))
        }

        "SetPersistentData" => {
            let realm = data.get("realm").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let slot = data.get("slotName").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let value = data.get("slotValue").cloned().unwrap_or(Value::Null);
            state
                .persistent_data
                .entry(realm)
                .or_default()
                .insert(slot, value);
            success_empty(request_type, request_id)
        }

        // Scenes
        "GetSceneList" => {
            let scenes: Vec<Value> = state
                .scenes
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    json!({
                        "sceneName": s.name,
                        "sceneUuid": s.uuid,
                        "sceneIndex": i
                    })
                })
                .collect();
            let prog = &state.scenes[state.current_program_scene];
            let prev = &state.scenes[state.current_preview_scene];
            success(request_type, request_id, json!({
                "currentProgramSceneName": prog.name,
                "currentProgramSceneUuid": prog.uuid,
                "currentPreviewSceneName": prev.name,
                "currentPreviewSceneUuid": prev.uuid,
                "scenes": scenes
            }))
        }

        "GetGroupList" => success(request_type, request_id, json!({ "groups": [] })),

        "GetCurrentProgramScene" => {
            let scene = &state.scenes[state.current_program_scene];
            success(request_type, request_id, json!({
                "sceneName": scene.name,
                "sceneUuid": scene.uuid,
                "currentProgramSceneName": scene.name,
                "currentProgramSceneUuid": scene.uuid
            }))
        }

        "SetCurrentProgramScene" => {
            if let Some(idx) = state.resolve_scene(data) {
                state.current_program_scene = idx;
                success_empty(request_type, request_id)
            } else {
                not_found(request_type, request_id, "Scene not found")
            }
        }

        "GetCurrentPreviewScene" => {
            let scene = &state.scenes[state.current_preview_scene];
            success(request_type, request_id, json!({
                "sceneName": scene.name,
                "sceneUuid": scene.uuid,
                "currentPreviewSceneName": scene.name,
                "currentPreviewSceneUuid": scene.uuid
            }))
        }

        "SetCurrentPreviewScene" => {
            if let Some(idx) = state.resolve_scene(data) {
                state.current_preview_scene = idx;
                success_empty(request_type, request_id)
            } else {
                not_found(request_type, request_id, "Scene not found")
            }
        }

        "CreateScene" => {
            let name = data
                .get("sceneName")
                .and_then(|v| v.as_str())
                .unwrap_or("New Scene")
                .to_string();
            let uuid = Uuid::new_v4().to_string();
            state.scenes.push(Scene {
                name,
                uuid: uuid.clone(),
            });
            success(request_type, request_id, json!({ "sceneUuid": uuid }))
        }

        "RemoveScene" => {
            if let Some(idx) = state.resolve_scene(data) {
                state.scenes.remove(idx);
                if state.current_program_scene >= state.scenes.len() {
                    state.current_program_scene = 0;
                }
                if state.current_preview_scene >= state.scenes.len() {
                    state.current_preview_scene = 0;
                }
                success_empty(request_type, request_id)
            } else {
                not_found(request_type, request_id, "Scene not found")
            }
        }

        "SetSceneName" => {
            if let Some(idx) = state.resolve_scene(data) {
                if let Some(new_name) = data.get("newSceneName").and_then(|v| v.as_str()) {
                    state.scenes[idx].name = new_name.to_string();
                }
                success_empty(request_type, request_id)
            } else {
                not_found(request_type, request_id, "Scene not found")
            }
        }

        "GetSceneSceneTransitionOverride" => success(request_type, request_id, json!({
            "transitionName": null,
            "transitionDuration": null
        })),

        "SetSceneSceneTransitionOverride" => success_empty(request_type, request_id),

        // Inputs
        "GetInputList" => {
            let filter_kind = data.get("inputKind").and_then(|v| v.as_str());
            let inputs: Vec<Value> = state
                .inputs
                .iter()
                .filter(|i| filter_kind.map_or(true, |k| i.kind == k))
                .map(|i| {
                    json!({
                        "inputName": i.name,
                        "inputUuid": i.uuid,
                        "inputKind": i.kind,
                        "unversionedInputKind": i.kind
                    })
                })
                .collect();
            success(request_type, request_id, json!({ "inputs": inputs }))
        }

        "GetInputKindList" => success(request_type, request_id, json!({
            "inputKinds": [
                "pulse_output_capture",
                "pulse_input_capture",
                "ffmpeg_source",
                "image_source",
                "browser_source",
                "vlc_source",
                "window_capture",
                "monitor_capture"
            ]
        })),

        "GetSpecialInputs" => success(request_type, request_id, json!({
            "desktop1": state.inputs.first().map(|i| i.name.as_str()).unwrap_or(""),
            "desktop2": null,
            "mic1": state.inputs.get(1).map(|i| i.name.as_str()),
            "mic2": null,
            "mic3": null,
            "mic4": null
        })),

        "CreateInput" => {
            let name = data.get("inputName").and_then(|v| v.as_str()).unwrap_or("New Input").to_string();
            let kind = data.get("inputKind").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let settings = data.get("inputSettings").cloned().unwrap_or(json!({}));
            let uuid = Uuid::new_v4().to_string();
            state.inputs.push(Input {
                name,
                uuid: uuid.clone(),
                kind,
                muted: false,
                volume_mul: 1.0,
                volume_db: 0.0,
                settings,
            });
            success(request_type, request_id, json!({
                "inputUuid": uuid,
                "sceneItemId": 1
            }))
        }

        "RemoveInput" => {
            if let Some(idx) = state.resolve_input(data) {
                state.inputs.remove(idx);
                success_empty(request_type, request_id)
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "SetInputName" => {
            if let Some(idx) = state.resolve_input(data) {
                if let Some(new_name) = data.get("newInputName").and_then(|v| v.as_str()) {
                    state.inputs[idx].name = new_name.to_string();
                }
                success_empty(request_type, request_id)
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "GetInputDefaultSettings" => success(request_type, request_id, json!({
            "defaultInputSettings": {}
        })),

        "GetInputSettings" => {
            if let Some(idx) = state.resolve_input(data) {
                let input = &state.inputs[idx];
                success(request_type, request_id, json!({
                    "inputSettings": input.settings,
                    "inputKind": input.kind
                }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "SetInputSettings" => {
            if let Some(idx) = state.resolve_input(data) {
                if let Some(new_settings) = data.get("inputSettings") {
                    let overlay = data.get("overlay").and_then(|v| v.as_bool()).unwrap_or(true);
                    if overlay {
                        if let (Some(existing), Some(new_obj)) =
                            (state.inputs[idx].settings.as_object_mut(), new_settings.as_object())
                        {
                            for (k, v) in new_obj {
                                existing.insert(k.clone(), v.clone());
                            }
                        }
                    } else {
                        state.inputs[idx].settings = new_settings.clone();
                    }
                }
                success_empty(request_type, request_id)
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "GetInputMute" => {
            if let Some(idx) = state.resolve_input(data) {
                success(request_type, request_id, json!({
                    "inputMuted": state.inputs[idx].muted
                }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "SetInputMute" => {
            if let Some(idx) = state.resolve_input(data) {
                if let Some(muted) = data.get("inputMuted").and_then(|v| v.as_bool()) {
                    state.inputs[idx].muted = muted;
                }
                success_empty(request_type, request_id)
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "ToggleInputMute" => {
            if let Some(idx) = state.resolve_input(data) {
                state.inputs[idx].muted = !state.inputs[idx].muted;
                success(request_type, request_id, json!({
                    "inputMuted": state.inputs[idx].muted
                }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "GetInputVolume" => {
            if let Some(idx) = state.resolve_input(data) {
                let input = &state.inputs[idx];
                success(request_type, request_id, json!({
                    "inputVolumeMul": input.volume_mul,
                    "inputVolumeDb": input.volume_db
                }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "SetInputVolume" => {
            if let Some(idx) = state.resolve_input(data) {
                if let Some(v) = data.get("inputVolumeMul").and_then(|v| v.as_f64()) {
                    state.inputs[idx].volume_mul = v;
                    state.inputs[idx].volume_db = 20.0 * v.max(0.0001).log10();
                }
                if let Some(v) = data.get("inputVolumeDb").and_then(|v| v.as_f64()) {
                    state.inputs[idx].volume_db = v;
                    state.inputs[idx].volume_mul = 10.0_f64.powf(v / 20.0);
                }
                success_empty(request_type, request_id)
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "GetInputAudioBalance" => {
            if state.resolve_input(data).is_some() {
                success(request_type, request_id, json!({ "inputAudioBalance": 0.5 }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "SetInputAudioBalance" | "SetInputAudioSyncOffset" | "SetInputAudioMonitorType"
        | "SetInputAudioTracks" | "SetInputDeinterlaceMode" | "SetInputDeinterlaceFieldOrder"
        | "PressInputPropertiesButton" => {
            if state.resolve_input(data).is_some() {
                success_empty(request_type, request_id)
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "GetInputAudioSyncOffset" => {
            if state.resolve_input(data).is_some() {
                success(request_type, request_id, json!({ "inputAudioSyncOffset": 0 }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "GetInputAudioMonitorType" => {
            if state.resolve_input(data).is_some() {
                success(request_type, request_id, json!({ "monitorType": "OBS_MONITORING_TYPE_NONE" }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "GetInputAudioTracks" => {
            if state.resolve_input(data).is_some() {
                success(request_type, request_id, json!({
                    "inputAudioTracks": { "1": true, "2": false, "3": false, "4": false, "5": false, "6": false }
                }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "GetInputDeinterlaceMode" => {
            if state.resolve_input(data).is_some() {
                success(request_type, request_id, json!({ "inputDeinterlaceMode": "OBS_DEINTERLACE_MODE_DISABLE" }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "GetInputDeinterlaceFieldOrder" => {
            if state.resolve_input(data).is_some() {
                success(request_type, request_id, json!({ "inputDeinterlaceFieldOrder": "OBS_DEINTERLACE_FIELD_ORDER_TOP" }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        "GetInputPropertiesListPropertyItems" => {
            if state.resolve_input(data).is_some() {
                success(request_type, request_id, json!({ "propertyItems": [] }))
            } else {
                not_found(request_type, request_id, "Input not found")
            }
        }

        // Sources
        "GetSourceActive" => success(request_type, request_id, json!({
            "videoActive": true,
            "videoShowing": true
        })),

        "GetSourceScreenshot" => success(request_type, request_id, json!({
            "imageData": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
        })),

        "SaveSourceScreenshot" => success_empty(request_type, request_id),

        // Streaming
        "GetStreamStatus" => success(request_type, request_id, json!({
            "outputActive": state.streaming,
            "outputReconnecting": false,
            "outputTimecode": state.stream_timecode,
            "outputDuration": 0,
            "outputCongestion": 0.0,
            "outputBytes": 0,
            "outputSkippedFrames": 0,
            "outputTotalFrames": 0
        })),

        "ToggleStream" => {
            state.streaming = !state.streaming;
            success(request_type, request_id, json!({ "outputActive": state.streaming }))
        }

        "StartStream" => {
            state.streaming = true;
            success_empty(request_type, request_id)
        }

        "StopStream" => {
            state.streaming = false;
            success_empty(request_type, request_id)
        }

        "SendStreamCaption" => success_empty(request_type, request_id),

        // Recording
        "GetRecordStatus" => success(request_type, request_id, json!({
            "outputActive": state.recording,
            "outputPaused": state.record_paused,
            "outputTimecode": state.record_timecode,
            "outputDuration": 0,
            "outputBytes": 0
        })),

        "ToggleRecord" => {
            state.recording = !state.recording;
            if !state.recording {
                state.record_paused = false;
            }
            success(request_type, request_id, json!({ "outputActive": state.recording }))
        }

        "StartRecord" => {
            state.recording = true;
            state.record_paused = false;
            success_empty(request_type, request_id)
        }

        "StopRecord" => {
            state.recording = false;
            state.record_paused = false;
            success(request_type, request_id, json!({
                "outputPath": "/tmp/obs-recordings/mock-recording.mkv"
            }))
        }

        "ToggleRecordPause" => {
            if state.recording {
                state.record_paused = !state.record_paused;
            }
            success_empty(request_type, request_id)
        }

        "PauseRecord" => {
            state.record_paused = true;
            success_empty(request_type, request_id)
        }

        "ResumeRecord" => {
            state.record_paused = false;
            success_empty(request_type, request_id)
        }

        "SplitRecordFile" | "CreateRecordChapter" => success_empty(request_type, request_id),

        // Outputs
        "GetVirtualCamStatus" => success(request_type, request_id, json!({
            "outputActive": state.virtual_cam
        })),

        "ToggleVirtualCam" => {
            state.virtual_cam = !state.virtual_cam;
            success(request_type, request_id, json!({ "outputActive": state.virtual_cam }))
        }

        "StartVirtualCam" => {
            state.virtual_cam = true;
            success_empty(request_type, request_id)
        }

        "StopVirtualCam" => {
            state.virtual_cam = false;
            success_empty(request_type, request_id)
        }

        "GetReplayBufferStatus" => success(request_type, request_id, json!({
            "outputActive": state.replay_buffer
        })),

        "ToggleReplayBuffer" => {
            state.replay_buffer = !state.replay_buffer;
            success(request_type, request_id, json!({ "outputActive": state.replay_buffer }))
        }

        "StartReplayBuffer" => {
            state.replay_buffer = true;
            success_empty(request_type, request_id)
        }

        "StopReplayBuffer" => {
            state.replay_buffer = false;
            success_empty(request_type, request_id)
        }

        "SaveReplayBuffer" => success_empty(request_type, request_id),

        "GetLastReplayBufferReplay" => success(request_type, request_id, json!({
            "savedReplayPath": "/tmp/obs-replays/mock-replay.mkv"
        })),

        "GetOutputList" => success(request_type, request_id, json!({ "outputs": [] })),

        "GetOutputStatus" => success(request_type, request_id, json!({
            "outputActive": false,
            "outputReconnecting": false,
            "outputTimecode": "00:00:00.000",
            "outputDuration": 0,
            "outputCongestion": 0.0,
            "outputBytes": 0,
            "outputSkippedFrames": 0,
            "outputTotalFrames": 0
        })),

        "ToggleOutput" => success(request_type, request_id, json!({ "outputActive": true })),
        "StartOutput" | "StopOutput" | "SetOutputSettings" => success_empty(request_type, request_id),
        "GetOutputSettings" => success(request_type, request_id, json!({ "outputSettings": {} })),

        // Scene Items
        "GetSceneItemList" | "GetGroupSceneItemList" => success(request_type, request_id, json!({
            "sceneItems": []
        })),

        "GetSceneItemId" => success(request_type, request_id, json!({ "sceneItemId": 1 })),

        "GetSceneItemSource" => success(request_type, request_id, json!({
            "sourceName": "Mock Source",
            "sourceUuid": Uuid::new_v4().to_string()
        })),

        "CreateSceneItem" => success(request_type, request_id, json!({ "sceneItemId": 1 })),

        "RemoveSceneItem" | "SetSceneItemTransform" | "SetSceneItemEnabled"
        | "SetSceneItemLocked" | "SetSceneItemIndex" | "SetSceneItemBlendMode" => {
            success_empty(request_type, request_id)
        }

        "DuplicateSceneItem" => success(request_type, request_id, json!({ "sceneItemId": 2 })),

        "GetSceneItemTransform" => success(request_type, request_id, json!({
            "sceneItemTransform": {
                "positionX": 0.0,
                "positionY": 0.0,
                "rotation": 0.0,
                "scaleX": 1.0,
                "scaleY": 1.0,
                "width": 1920.0,
                "height": 1080.0,
                "alignment": 5,
                "boundsType": "OBS_BOUNDS_NONE",
                "boundsAlignment": 0,
                "boundsWidth": 0.0,
                "boundsHeight": 0.0,
                "cropLeft": 0,
                "cropRight": 0,
                "cropTop": 0,
                "cropBottom": 0,
                "sourceWidth": 1920.0,
                "sourceHeight": 1080.0
            }
        })),

        "GetSceneItemEnabled" => success(request_type, request_id, json!({ "sceneItemEnabled": true })),
        "GetSceneItemLocked" => success(request_type, request_id, json!({ "sceneItemLocked": false })),
        "GetSceneItemIndex" => success(request_type, request_id, json!({ "sceneItemIndex": 0 })),
        "GetSceneItemBlendMode" => success(request_type, request_id, json!({ "sceneItemBlendMode": "OBS_BLEND_NORMAL" })),

        // Transitions
        "GetTransitionKindList" => success(request_type, request_id, json!({
            "transitionKinds": ["cut_transition", "fade_transition", "swipe_transition", "slide_transition"]
        })),

        "GetSceneTransitionList" => success(request_type, request_id, json!({
            "currentSceneTransitionName": state.current_transition,
            "currentSceneTransitionUuid": Uuid::new_v4().to_string(),
            "currentSceneTransitionKind": "fade_transition",
            "transitions": [
                {
                    "transitionName": "Cut",
                    "transitionUuid": Uuid::new_v4().to_string(),
                    "transitionKind": "cut_transition",
                    "transitionFixed": false,
                    "transitionConfigurable": false
                },
                {
                    "transitionName": "Fade",
                    "transitionUuid": Uuid::new_v4().to_string(),
                    "transitionKind": "fade_transition",
                    "transitionFixed": false,
                    "transitionConfigurable": true
                }
            ]
        })),

        "GetCurrentSceneTransition" => success(request_type, request_id, json!({
            "transitionName": state.current_transition,
            "transitionUuid": Uuid::new_v4().to_string(),
            "transitionKind": "fade_transition",
            "transitionFixed": false,
            "transitionDuration": state.transition_duration,
            "transitionConfigurable": true,
            "transitionSettings": {}
        })),

        "SetCurrentSceneTransition" => {
            if let Some(name) = data.get("transitionName").and_then(|v| v.as_str()) {
                state.current_transition = name.to_string();
            }
            success_empty(request_type, request_id)
        }

        "SetCurrentSceneTransitionDuration" => {
            if let Some(d) = data.get("transitionDuration").and_then(|v| v.as_u64()) {
                state.transition_duration = d as u32;
            }
            success_empty(request_type, request_id)
        }

        "SetCurrentSceneTransitionSettings" | "TriggerStudioModeTransition" | "SetTBarPosition" => {
            success_empty(request_type, request_id)
        }

        "GetCurrentSceneTransitionCursor" => success(request_type, request_id, json!({
            "transitionCursor": 0.0
        })),

        // Filters
        "GetSourceFilterKindList" => success(request_type, request_id, json!({
            "sourceFilterKinds": ["color_filter", "crop_filter", "gain_filter", "noise_gate_filter"]
        })),

        "GetSourceFilterList" => success(request_type, request_id, json!({ "filters": [] })),

        "GetSourceFilterDefaultSettings" => success(request_type, request_id, json!({
            "defaultFilterSettings": {}
        })),

        "CreateSourceFilter" | "RemoveSourceFilter" | "SetSourceFilterName"
        | "SetSourceFilterIndex" | "SetSourceFilterSettings" | "SetSourceFilterEnabled" => {
            success_empty(request_type, request_id)
        }

        "GetSourceFilter" => success(request_type, request_id, json!({
            "filterEnabled": true,
            "filterIndex": 0,
            "filterKind": "color_filter",
            "filterSettings": {}
        })),

        // Media Inputs
        "GetMediaInputStatus" => success(request_type, request_id, json!({
            "mediaState": "OBS_MEDIA_STATE_NONE",
            "mediaDuration": 0,
            "mediaCursor": 0
        })),

        "SetMediaInputCursor" | "OffsetMediaInputCursor" | "TriggerMediaInputAction" => {
            success_empty(request_type, request_id)
        }

        // UI
        "GetStudioModeEnabled" => success(request_type, request_id, json!({
            "studioModeEnabled": state.studio_mode
        })),

        "SetStudioModeEnabled" => {
            if let Some(enabled) = data.get("studioModeEnabled").and_then(|v| v.as_bool()) {
                state.studio_mode = enabled;
            }
            success_empty(request_type, request_id)
        }

        "OpenInputPropertiesDialog" | "OpenInputFiltersDialog" | "OpenInputInteractDialog"
        | "OpenVideoMixProjector" | "OpenSourceProjector" => {
            success_empty(request_type, request_id)
        }

        "GetMonitorList" => success(request_type, request_id, json!({ "monitors": [] })),

        // Canvases
        "GetCanvasList" => success(request_type, request_id, json!({ "canvases": [] })),

        // Unknown request
        _ => {
            log::warn!("Unknown request type: {}", request_type);
            RequestResponse::error(
                request_type.to_string(),
                request_id.to_string(),
                REQUEST_STATUS_UNKNOWN,
                format!("Unknown request type: {}", request_type),
            )
        }
    }
}

fn success(request_type: &str, request_id: &str, response_data: Value) -> RequestResponse {
    RequestResponse::success(
        request_type.to_string(),
        request_id.to_string(),
        Some(response_data),
    )
}

fn success_empty(request_type: &str, request_id: &str) -> RequestResponse {
    RequestResponse::success(request_type.to_string(), request_id.to_string(), None)
}

fn not_found(request_type: &str, request_id: &str, comment: &str) -> RequestResponse {
    RequestResponse::error(
        request_type.to_string(),
        request_id.to_string(),
        600,
        comment.to_string(),
    )
}
