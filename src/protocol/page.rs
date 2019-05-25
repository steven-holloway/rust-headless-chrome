use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub id: String,
    pub parent_id: Option<String>,
    pub loader_id: String,
    pub name: Option<String>,
    pub url: String,
    pub security_origin: String,
    pub mime_type: String,
    pub unreachable_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum InternalScreenshotFormat {
    JPEG,
    PNG,
}

/// Viewport for capturing screenshot.
#[derive(Debug, Clone, Serialize)]
pub struct Viewport {
    /// X offset in device independent pixels
    pub x: f64,
    /// Y offset in device independent pixels
    pub y: f64,
    /// Rectangle width in device independent pixels
    pub width: f64,
    /// Rectangle height in device independent pixels
    pub height: f64,
    /// Page scale factor
    pub scale: f64,
}

/// The format a screenshot will be captured in
#[derive(Debug, Clone)]
pub enum ScreenshotFormat {
    /// Optionally compression quality from range [0..100]
    JPEG(Option<u8>),
    PNG,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrintToPdfOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landscape: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_header_footer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_ranges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_invalid_page_ranges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_css_page_size: Option<bool>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EmulateMediaOptions {
    pub media_type: String
}

pub mod events {
    use serde::Deserialize;
    #[derive(Deserialize, Debug, Clone)]
    pub struct LifecycleEvent {
        pub params: LifecycleParams,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct LifecycleParams {
        pub frame_id: String,
        pub loader_id: String,
        pub name: String,
        pub timestamp: f32,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct FrameStartedLoadingEvent {
        pub params: FrameStartedLoadingParams,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameStartedLoadingParams {
        pub frame_id: String,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct FrameNavigatedEvent {
        pub params: FrameNavigatedParams,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameNavigatedParams {
        pub frame: super::Frame,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct FrameStoppedLoadingEvent {
        pub params: FrameStoppedLoadingParams,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameStoppedLoadingParams {
        pub frame_id: String,
    }
}

pub mod methods {
    use super::PrintToPdfOptions;
    use super::EmulateMediaOptions;
    use crate::protocol::Method;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CaptureScreenshot {
        pub format: super::InternalScreenshotFormat,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub quality: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub clip: Option<super::Viewport>,
        pub from_surface: bool,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CaptureScreenshotReturnObject {
        pub data: String,
    }
    impl Method for CaptureScreenshot {
        const NAME: &'static str = "Page.captureScreenshot";
        type ReturnObject = CaptureScreenshotReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct PrintToPdf {
        #[serde(flatten)]
        pub options: Option<PrintToPdfOptions>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PrintToPdfReturnObject {
        pub data: String,
    }
    impl Method for PrintToPdf {
        const NAME: &'static str = "Page.printToPDF";
        type ReturnObject = PrintToPdfReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct EmulateMedia {
        #[serde(flatten)]
        pub options: Option<EmulateMediaOptions>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EmulateMediaReturnObject {
        pub data: String,
    }
    impl Method for EmulateMedia {
        const NAME: &'static str = "Emulation.setEmulatedMedia";
        type ReturnObject = EmulateMediaReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Reload<'a> {
        pub ignore_cache: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub script_to_evaluate: Option<&'a str>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ReloadReturnObject {}
    impl<'a> Method for Reload<'a> {
        const NAME: &'static str = "Page.reload";
        type ReturnObject = ReloadReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetLifecycleEventsEnabled {
        pub enabled: bool,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SetLifecycleEventsEnabledReturnObject {}
    impl Method for SetLifecycleEventsEnabled {
        const NAME: &'static str = "Page.setLifecycleEventsEnabled";
        type ReturnObject = SetLifecycleEventsEnabledReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetFrameTree {}

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameTree {
        pub frame: super::Frame,
        pub child_frames: Option<Vec<FrameTree>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetFrameTreeReturnObject {
        pub frame_tree: FrameTree,
    }
    impl Method for GetFrameTree {
        const NAME: &'static str = "Page.getFrameTree";
        type ReturnObject = GetFrameTreeReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Navigate<'a> {
        pub url: &'a str,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NavigateReturnObject {
        pub frame_id: String,
        pub loader_id: Option<String>,
        pub error_text: Option<String>,
    }
    impl<'a> Method for Navigate<'a> {
        const NAME: &'static str = "Page.navigate";
        type ReturnObject = NavigateReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Enable {}
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EnableReturnObject {}
    impl Method for Enable {
        const NAME: &'static str = "Page.enable";
        type ReturnObject = EnableReturnObject;
    }

}
