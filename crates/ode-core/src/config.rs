use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionConfig {
    pub page_range: (usize, usize),
    pub zoom: f64,
    pub fit_width: Option<f64>,
    pub fit_height: Option<f64>,
    pub use_cropbox: bool,
    pub desired_dpi: f64,
    pub max_dpi: Option<f64>,
    pub text_dpi: f64,

    pub embed_css: bool,
    pub embed_font: bool,
    pub embed_image: bool,
    pub embed_javascript: bool,
    pub embed_outline: bool,
    pub split_pages: bool,

    pub dest_dir: Option<String>,
    pub css_filename: Option<String>,
    pub page_filename: Option<String>,
    pub outline_filename: Option<String>,

    pub process_nontext: bool,
    pub process_outline: bool,
    pub process_annotation: bool,
    pub process_form: bool,
    pub correct_text_visibility: bool,
    pub printing: bool,
    pub fallback: bool,
    pub tmp_file_size_limit: Option<usize>,

    pub font_format: FontFormat,
    pub decompose_ligature: bool,
    pub turn_off_ligatures: bool,
    pub auto_hint: bool,
    pub external_hint_tool: Option<String>,
    pub stretch_narrow_glyph: bool,
    pub squeeze_wide_glyph: bool,
    pub override_fstype: bool,
    pub process_type3: bool,

    pub h_eps: f64,
    pub v_eps: f64,
    pub space_threshold: f64,
    pub font_size_multiplier: f64,
    pub space_as_offset: bool,
    pub tounicode: bool,
    pub optimize_text: bool,

    pub bg_format: BackgroundFormat,
    pub svg_node_count_limit: Option<usize>,
    pub svg_embed_bitmap: bool,

    pub owner_password: Option<String>,
    pub user_password: Option<String>,
    pub no_drm: bool,

    pub clean_tmp: bool,
    pub debug: bool,
    pub proof: bool,
    pub quiet: bool,

    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FontFormat {
    Woff2,
    Woff,
    Ttf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackgroundFormat {
    Png,
    Jpeg,
    Svg,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            page_range: (1, usize::MAX),
            zoom: 1.0,
            fit_width: None,
            fit_height: None,
            use_cropbox: false,
            desired_dpi: 72.0,
            max_dpi: None,
            text_dpi: 72.0,

            embed_css: true,
            embed_font: true,
            embed_image: true,
            embed_javascript: false,
            embed_outline: true,
            split_pages: false,

            dest_dir: None,
            css_filename: None,
            page_filename: None,
            outline_filename: None,

            process_nontext: true,
            process_outline: true,
            process_annotation: true,
            process_form: true,
            correct_text_visibility: true,
            printing: false,
            fallback: true,
            tmp_file_size_limit: Some(50 * 1024 * 1024),

            font_format: FontFormat::Woff2,
            decompose_ligature: false,
            turn_off_ligatures: false,
            auto_hint: false,
            external_hint_tool: None,
            stretch_narrow_glyph: false,
            squeeze_wide_glyph: false,
            override_fstype: false,
            process_type3: true,

            h_eps: 0.5,
            v_eps: 0.5,
            space_threshold: 0.1,
            font_size_multiplier: 1.0,
            space_as_offset: false,
            tounicode: true,
            optimize_text: true,

            bg_format: BackgroundFormat::Png,
            svg_node_count_limit: Some(1000),
            svg_embed_bitmap: true,

            owner_password: None,
            user_password: None,
            no_drm: false,

            clean_tmp: true,
            debug: false,
            proof: false,
            quiet: false,

            timeout_ms: Some(30000),
        }
    }
}
