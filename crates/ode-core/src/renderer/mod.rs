pub mod text;

use crate::config::ConversionConfig;
use crate::error::OdeError;
use crate::fonts::FontProcessor;
use crate::parser::{content_stream::ContentStreamParser, ParsedOp, PdfDocument};
use crate::render::state::GraphicsState;
use crate::util::hash::ContentHasher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSpan {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size: f64,
    pub font_id: Option<u64>,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedFont {
    pub font_id: u64,
    pub font_name: String,
    #[serde(skip)]
    pub data: Vec<u8>,
    pub format: crate::config::FontFormat,
    pub content_hash: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageImageRef {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub data_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedPage {
    pub page_number: usize,
    pub width: f64,
    pub height: f64,
    pub html: String,
    pub css: String,
    pub text_spans: Vec<TextSpan>,
    pub font_ids: Vec<u64>,
    pub background_color: Option<String>,
    pub images: Vec<PageImageRef>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputBundle {
    pub pages: Vec<RenderedPage>,
    pub fonts: Vec<RenderedFont>,
    pub css: String,
}

impl OutputBundle {
    pub fn add_page(&mut self, page: RenderedPage) {
        self.pages.push(page);
    }

    pub fn add_font(
        &mut self,
        font_id: u64,
        font_name: String,
        font_data: Vec<u8>,
        format: crate::config::FontFormat,
    ) {
        let content_hash = ContentHasher::hash_bytes(&font_data);
        let extension = match format {
            crate::config::FontFormat::Woff2 => "woff2",
            crate::config::FontFormat::Woff => "woff",
            crate::config::FontFormat::Ttf => "ttf",
        };
        let filename = ContentHasher::generate_content_addressed_filename(&font_data, extension);

        let rendered_font = RenderedFont {
            font_id,
            font_name,
            data: font_data,
            format,
            content_hash,
            filename: filename.clone(),
        };

        self.fonts.push(rendered_font);
    }

    pub fn get_font_by_id(&self, font_id: u64) -> Option<&RenderedFont> {
        self.fonts.iter().find(|f| f.font_id == font_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentOp {
    BT,
    ET,
    Td,
    TD,
    Tm,
    Tj,
    TJ,
    Tc,
    Tw,
    Tz,
    TL,
    Tstar,
    Tf,
    SC,
    SCN,
    RG,
    RGfill,
    Gstroke,
    Gfill,
    Kstroke,
    Kfill,
    GsSave,
    GsRestore,
    CM,
    M,
    L,
    C,
    H,
    S,
    Ss,
    F,
    Fs,
    B,
    Bs,
    Bx,
    Bxs,
    N,
    W,
    Ws,
    RE,
    Wstar,
    Do,
}

impl ContentOp {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "BT" => Some(ContentOp::BT),
            "ET" => Some(ContentOp::ET),
            "Td" => Some(ContentOp::Td),
            "TD" => Some(ContentOp::TD),
            "Tm" => Some(ContentOp::Tm),
            "Tj" => Some(ContentOp::Tj),
            "TJ" => Some(ContentOp::TJ),
            "Tc" => Some(ContentOp::Tc),
            "Tw" => Some(ContentOp::Tw),
            "Tz" => Some(ContentOp::Tz),
            "TL" => Some(ContentOp::TL),
            "T*" => Some(ContentOp::Tstar),
            "Tf" => Some(ContentOp::Tf),
            "RG" => Some(ContentOp::RG),
            "rg" => Some(ContentOp::RGfill),
            "G" => Some(ContentOp::Gstroke),
            "g" => Some(ContentOp::Gfill),
            "K" => Some(ContentOp::Kstroke),
            "k" => Some(ContentOp::Kfill),
            "sc" => Some(ContentOp::SC),
            "SC" => Some(ContentOp::SC),
            "scn" => Some(ContentOp::SCN),
            "SCN" => Some(ContentOp::SCN),
            "q" => Some(ContentOp::GsSave),
            "Q" => Some(ContentOp::GsRestore),
            "cm" => Some(ContentOp::CM),
            "m" => Some(ContentOp::M),
            "l" => Some(ContentOp::L),
            "c" => Some(ContentOp::C),
            "h" => Some(ContentOp::H),
            "S" => Some(ContentOp::S),
            "s" => Some(ContentOp::Ss),
            "f" => Some(ContentOp::F),
            "F" => Some(ContentOp::F),
            "f*" => Some(ContentOp::Fs),
            "B" => Some(ContentOp::B),
            "b" => Some(ContentOp::Bs),
            "B*" => Some(ContentOp::Bx),
            "b*" => Some(ContentOp::Bxs),
            "n" => Some(ContentOp::N),
            "W" => Some(ContentOp::W),
            "W*" => Some(ContentOp::Wstar),
            "Ws" => Some(ContentOp::Ws),
            "re" => Some(ContentOp::RE),
            "Do" => Some(ContentOp::Do),
            _ => None,
        }
    }
}

pub fn render_pdf_page(
    document: &PdfDocument,
    page_id: usize,
    page_number: usize,
    config: &ConversionConfig,
) -> Result<RenderedPage, OdeError> {
    use crate::util::math::TransformMatrix;

    let page = document
        .get_page(page_id as u32)
        .ok_or_else(|| OdeError::PdfParseError(format!("Cannot fetch page {}", page_number)))?;

    let page_width = page.width;
    let page_height = page.height;

    let mut text_extractor = text::TextExtractor::new();

    let ops = parse_content_stream(&page.contents)?;

    let mut graphics_state = GraphicsState::new();
    let mut ctm = TransformMatrix::identity(); // Current Transformation Matrix
    let mut text_matrix = TransformMatrix::identity();
    let mut state_stack: Vec<(TransformMatrix, GraphicsState, Option<String>)> = Vec::new();
    let mut current_font_name: Option<String> = None;
    let mut background_color: Option<String> = None;
    let mut pending_rect: Option<(f64, f64, f64, f64)> = None;
    let mut rendered_images: Vec<PageImageRef> = Vec::new();

    for op in ops {
        match op.operator {
            ContentOp::GsSave => {
                state_stack.push((ctm, graphics_state.clone(), current_font_name.clone()));
            }
            ContentOp::GsRestore => {
                if let Some((saved_ctm, saved_state, saved_font)) = state_stack.pop() {
                    ctm = saved_ctm;
                    graphics_state = saved_state;
                    current_font_name = saved_font;
                }
            }
            ContentOp::CM => {
                if op.operands.len() >= 6 {
                    let new_matrix = TransformMatrix {
                        a: op.operands[0],
                        b: op.operands[1],
                        c: op.operands[2],
                        d: op.operands[3],
                        e: op.operands[4],
                        f: op.operands[5],
                    };
                    // CTM = CTM_old * new_matrix (column-vector convention)
                    ctm = ctm * new_matrix;
                }
            }
            ContentOp::BT => {
                text_extractor.finalize_segment();
                text_matrix = TransformMatrix::identity();
            }
            ContentOp::ET => {
                text_extractor.finalize_segment();
            }
            ContentOp::Tf => {
                // Tf operands: font_size (the font name was already consumed by content stream parser)
                if !op.operands.is_empty() {
                    let font_size = op.operands[op.operands.len() - 1].abs();
                    if font_size > 0.0 {
                        graphics_state.font_size = font_size;
                    }
                }
                if let Some(ref name) = op.font_name {
                    current_font_name = Some(name.clone());
                }
                if graphics_state.font_info.is_none() {
                    graphics_state.font_info = Some(crate::render::state::FontInfo {
                        id: 0,
                        use_tounicode: true,
                        em_size: 1000.0,
                        space_width: 250.0,
                        ascent: 0.8,
                        descent: -0.2,
                        is_type3: false,
                        font_size_scale: 1.0,
                    });
                }
            }
            ContentOp::Tm => {
                if op.operands.len() >= 6 {
                    text_matrix = TransformMatrix {
                        a: op.operands[0],
                        b: op.operands[1],
                        c: op.operands[2],
                        d: op.operands[3],
                        e: op.operands[4],
                        f: op.operands[5],
                    };
                }
            }
            ContentOp::Td => {
                if op.operands.len() >= 2 {
                    let tx = op.operands[0];
                    let ty = op.operands[1];
                    text_matrix.e += tx * text_matrix.a + ty * text_matrix.c;
                    text_matrix.f += tx * text_matrix.b + ty * text_matrix.d;
                }
            }
            ContentOp::TD => {
                if op.operands.len() >= 2 {
                    let tx = op.operands[0];
                    let ty = op.operands[1];
                    text_matrix.e += tx * text_matrix.a + ty * text_matrix.c;
                    text_matrix.f += tx * text_matrix.b + ty * text_matrix.d;
                }
            }
            ContentOp::Tj | ContentOp::TJ => {
                // Decode text through ToUnicode CMap if available
                let decoded_text = if let Some(ref raw) = op.text_raw {
                    if let Some(ref font_name) = current_font_name {
                        if let Some(cmap) = page.font_cmaps.get(font_name) {
                            if !cmap.char_map.is_empty() {
                                Some(cmap.decode_bytes(raw))
                            } else {
                                op.text.clone()
                            }
                        } else {
                            op.text.clone()
                        }
                    } else {
                        op.text.clone()
                    }
                } else {
                    op.text.clone()
                };

                if let Some(text) = &decoded_text {
                    // Apply CTM to the text position from Tm
                    let (tm_x, tm_y) = text_matrix.transform_point(0.0, 0.0);
                    let (page_x, page_y) = ctm.transform_point(tm_x, tm_y);

                    // Font size = font_size × Tm scale × CTM scale
                    let tm_scale_y = (text_matrix.b * text_matrix.b + text_matrix.d * text_matrix.d).sqrt();
                    let ctm_scale_y = (ctm.b * ctm.b + ctm.d * ctm.d).sqrt();
                    let effective_font_size = graphics_state.font_size * tm_scale_y * ctm_scale_y;

                    // PDF coordinate system has Y=0 at bottom, increasing upward.
                    // HTML/CSS has Y=0 at top, increasing downward.
                    // Also adjust for baseline: PDF positions text at baseline,
                    // but CSS top positions the top of the element. Subtract
                    // approximate ascent (~85% of font size) to align correctly.
                    let html_y = page_height - page_y - effective_font_size * 0.85;

                    let mut state_for_text = graphics_state.clone();
                    state_for_text.font_size = effective_font_size;

                    text_extractor.update_state(&state_for_text);
                    text_extractor
                        .add_text(text, page_x, html_y)
                        .map_err(|e| OdeError::TextError(format!("Failed to add text: {}", e)))?;
                }
            }
            ContentOp::Tc => {
                if !op.operands.is_empty() {
                    graphics_state.letter_space = op.operands[0];
                }
            }
            ContentOp::Tw => {
                if !op.operands.is_empty() {
                    graphics_state.word_space = op.operands[0];
                }
            }
            ContentOp::Tz => {
                if !op.operands.is_empty() {
                    let scale = op.operands[0];
                    graphics_state.transform_matrix.a = scale / 100.0;
                }
            }
            ContentOp::TL => {}
            ContentOp::Tstar => {
                if let Some(ref font_info) = graphics_state.font_info {
                    if crate::util::math::equal(font_info.em_size, 0.0) {
                        text_matrix.f += -text_matrix.d;
                    }
                }
            }
            ContentOp::RGfill => {
                if op.operands.len() >= 3 {
                    let r = (op.operands[0].clamp(0.0, 1.0) * 255.0) as u8;
                    let g = (op.operands[1].clamp(0.0, 1.0) * 255.0) as u8;
                    let b = (op.operands[2].clamp(0.0, 1.0) * 255.0) as u8;
                    graphics_state.fill_color = crate::types::color::Color::new(r, g, b);
                }
            }
            ContentOp::RG => {
                if op.operands.len() >= 3 {
                    let r = (op.operands[0].clamp(0.0, 1.0) * 255.0) as u8;
                    let g = (op.operands[1].clamp(0.0, 1.0) * 255.0) as u8;
                    let b = (op.operands[2].clamp(0.0, 1.0) * 255.0) as u8;
                    graphics_state.stroke_color = crate::types::color::Color::new(r, g, b);
                }
            }
            ContentOp::Gfill => {
                if !op.operands.is_empty() {
                    let v = (op.operands[0].clamp(0.0, 1.0) * 255.0) as u8;
                    graphics_state.fill_color = crate::types::color::Color::new(v, v, v);
                }
            }
            ContentOp::Gstroke => {
                if !op.operands.is_empty() {
                    let v = (op.operands[0].clamp(0.0, 1.0) * 255.0) as u8;
                    graphics_state.stroke_color = crate::types::color::Color::new(v, v, v);
                }
            }
            ContentOp::Kfill => {
                if op.operands.len() >= 4 {
                    let c = op.operands[0];
                    let m = op.operands[1];
                    let y = op.operands[2];
                    let k = op.operands[3];
                    let r = ((1.0 - c) * (1.0 - k) * 255.0) as u8;
                    let g = ((1.0 - m) * (1.0 - k) * 255.0) as u8;
                    let b = ((1.0 - y) * (1.0 - k) * 255.0) as u8;
                    graphics_state.fill_color = crate::types::color::Color::new(r, g, b);
                }
            }
            ContentOp::SCN | ContentOp::SC => {
                if op.operands.len() >= 3 {
                    let r = (op.operands[0].clamp(0.0, 1.0) * 255.0) as u8;
                    let g = (op.operands[1].clamp(0.0, 1.0) * 255.0) as u8;
                    let b = (op.operands[2].clamp(0.0, 1.0) * 255.0) as u8;
                    graphics_state.fill_color = crate::types::color::Color::new(r, g, b);
                }
            }
            ContentOp::RE => {
                // Track rectangle for background detection
                if op.operands.len() >= 4 {
                    pending_rect = Some((op.operands[0], op.operands[1], op.operands[2], op.operands[3]));
                }
            }
            ContentOp::F => {
                // Fill operation — detect page-filling rectangles for background
                if background_color.is_none() {
                    if let Some((_x, _y, w, h)) = pending_rect {
                        // Check if rect covers a large area (likely page background)
                        let scaled_w = w * ctm.a.abs();
                        let scaled_h = h * ctm.d.abs();
                        if scaled_w >= page_width * 0.9 && scaled_h >= page_height * 0.9 {
                            background_color = Some(graphics_state.fill_color.to_css_string());
                        }
                    }
                }
                pending_rect = None;
            }
            ContentOp::Do => {
                if let Some(ref xobj_name) = op.font_name {
                    // Check if it's an image XObject
                    if let Some(img) = page.images.get(xobj_name) {
                        let (x, y) = ctm.transform_point(0.0, 0.0);
                        let (x2, _) = ctm.transform_point(1.0, 0.0);
                        let (_, y2) = ctm.transform_point(0.0, 1.0);
                        let w = (x2 - x).abs();
                        let h = (y2 - y).abs();
                        let img_y = page_height - y.max(y2);

                        use base64::Engine;
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&img.data);
                        let data_uri = format!("data:{};base64,{}", img.mime_type, b64);

                        rendered_images.push(PageImageRef {
                            name: xobj_name.clone(),
                            x,
                            y: img_y,
                            width: w,
                            height: h,
                            data_uri,
                        });
                    }
                    // Check if it's a Form XObject — render its content recursively
                    else if let Some(form) = page.form_xobjects.get(xobj_name) {
                        if let Ok(form_result) = render_form_xobject(
                            form, &ctm, page_width, page_height,
                            &graphics_state, &current_font_name, page,
                        ) {
                            text_extractor.merge_spans(&form_result.text_spans);
                            rendered_images.extend(form_result.images);
                            if background_color.is_none() {
                                background_color = form_result.background_color;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    text_extractor.finalize_segment();

    let text_spans = text_extractor.get_spans();
    let font_ids: Vec<u64> = text_spans
        .iter()
        .filter_map(|span| span.font_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let html = generate_page_html_with_images_and_text(
        page_number, page_width, page_height, config, &text_spans, &rendered_images,
    );
    let css = generate_page_css(config);

    Ok(RenderedPage {
        page_number,
        width: page_width,
        height: page_height,
        html,
        css,
        text_spans,
        font_ids,
        background_color,
        images: rendered_images,
    })
}

struct FormRenderResult {
    text_spans: Vec<TextSpan>,
    images: Vec<PageImageRef>,
    background_color: Option<String>,
}

/// Recursively render a Form XObject's content stream.
fn render_form_xobject(
    form: &crate::parser::FormXObject,
    parent_ctm: &crate::util::math::TransformMatrix,
    page_width: f64,
    page_height: f64,
    parent_gs: &GraphicsState,
    parent_font_name: &Option<String>,
    page: &crate::parser::PdfPage,
) -> Result<FormRenderResult, OdeError> {
    use crate::util::math::TransformMatrix;

    // Apply the form's own Matrix to the parent CTM
    let form_ctm = if let Some(m) = form.matrix {
        let form_matrix = TransformMatrix {
            a: m[0], b: m[1], c: m[2], d: m[3], e: m[4], f: m[5],
        };
        *parent_ctm * form_matrix
    } else {
        *parent_ctm
    };

    let mut text_extractor = text::TextExtractor::new();
    let ops = parse_content_stream(&form.content_stream)?;

    let mut ctm = form_ctm;
    let mut graphics_state = parent_gs.clone();
    let mut text_matrix = TransformMatrix::identity();
    let mut state_stack: Vec<(TransformMatrix, GraphicsState, Option<String>)> = Vec::new();
    let mut current_font_name: Option<String> = parent_font_name.clone();
    let mut background_color: Option<String> = None;
    let mut pending_rect: Option<(f64, f64, f64, f64)> = None;
    let mut rendered_images: Vec<PageImageRef> = Vec::new();

    // Use form's font_cmaps if available, fall back to page's
    let font_cmaps = if !form.font_cmaps.is_empty() {
        &form.font_cmaps
    } else {
        &page.font_cmaps
    };

    // Merge image sources: form's own + page's
    for op in ops {
        match op.operator {
            ContentOp::GsSave => {
                state_stack.push((ctm, graphics_state.clone(), current_font_name.clone()));
            }
            ContentOp::GsRestore => {
                if let Some((saved_ctm, saved_state, saved_font)) = state_stack.pop() {
                    ctm = saved_ctm;
                    graphics_state = saved_state;
                    current_font_name = saved_font;
                }
            }
            ContentOp::CM => {
                if op.operands.len() >= 6 {
                    let new_matrix = TransformMatrix {
                        a: op.operands[0], b: op.operands[1],
                        c: op.operands[2], d: op.operands[3],
                        e: op.operands[4], f: op.operands[5],
                    };
                    ctm = ctm * new_matrix;
                }
            }
            ContentOp::BT => {
                text_extractor.finalize_segment();
                text_matrix = TransformMatrix::identity();
            }
            ContentOp::ET => {
                text_extractor.finalize_segment();
            }
            ContentOp::Tf => {
                if !op.operands.is_empty() {
                    let font_size = op.operands[op.operands.len() - 1].abs();
                    if font_size > 0.0 {
                        graphics_state.font_size = font_size;
                    }
                }
                if let Some(ref name) = op.font_name {
                    current_font_name = Some(name.clone());
                }
                if graphics_state.font_info.is_none() {
                    graphics_state.font_info = Some(crate::render::state::FontInfo {
                        id: 0, use_tounicode: true, em_size: 1000.0,
                        space_width: 250.0, ascent: 0.8, descent: -0.2,
                        is_type3: false, font_size_scale: 1.0,
                    });
                }
            }
            ContentOp::Tm => {
                if op.operands.len() >= 6 {
                    text_matrix = TransformMatrix {
                        a: op.operands[0], b: op.operands[1],
                        c: op.operands[2], d: op.operands[3],
                        e: op.operands[4], f: op.operands[5],
                    };
                }
            }
            ContentOp::Td => {
                if op.operands.len() >= 2 {
                    let tx = op.operands[0];
                    let ty = op.operands[1];
                    text_matrix.e += tx * text_matrix.a + ty * text_matrix.c;
                    text_matrix.f += tx * text_matrix.b + ty * text_matrix.d;
                }
            }
            ContentOp::TD => {
                if op.operands.len() >= 2 {
                    let tx = op.operands[0];
                    let ty = op.operands[1];
                    text_matrix.e += tx * text_matrix.a + ty * text_matrix.c;
                    text_matrix.f += tx * text_matrix.b + ty * text_matrix.d;
                }
            }
            ContentOp::Tj | ContentOp::TJ => {
                let decoded_text = if let Some(ref raw) = op.text_raw {
                    if let Some(ref fname) = current_font_name {
                        if let Some(cmap) = font_cmaps.get(fname) {
                            if !cmap.char_map.is_empty() {
                                Some(cmap.decode_bytes(raw))
                            } else { op.text.clone() }
                        } else { op.text.clone() }
                    } else { op.text.clone() }
                } else { op.text.clone() };

                if let Some(text) = &decoded_text {
                    let (tm_x, tm_y) = text_matrix.transform_point(0.0, 0.0);
                    let (page_x, page_y) = ctm.transform_point(tm_x, tm_y);

                    // Font size = font_size × Tm scale × CTM scale
                    let tm_scale_y = (text_matrix.b * text_matrix.b + text_matrix.d * text_matrix.d).sqrt();
                    let ctm_scale_y = (ctm.b * ctm.b + ctm.d * ctm.d).sqrt();
                    let effective_font_size = graphics_state.font_size * tm_scale_y * ctm_scale_y;
                    let html_y = page_height - page_y - effective_font_size * 0.85;

                    let mut state_for_text = graphics_state.clone();
                    state_for_text.font_size = effective_font_size;

                    text_extractor.update_state(&state_for_text);
                    text_extractor
                        .add_text(text, page_x, html_y)
                        .map_err(|e| OdeError::TextError(format!("Form text: {}", e)))?;
                }
            }
            ContentOp::Tc => {
                if !op.operands.is_empty() { graphics_state.letter_space = op.operands[0]; }
            }
            ContentOp::Tw => {
                if !op.operands.is_empty() { graphics_state.word_space = op.operands[0]; }
            }
            ContentOp::RGfill => {
                if op.operands.len() >= 3 {
                    let r = (op.operands[0].clamp(0.0, 1.0) * 255.0) as u8;
                    let g = (op.operands[1].clamp(0.0, 1.0) * 255.0) as u8;
                    let b = (op.operands[2].clamp(0.0, 1.0) * 255.0) as u8;
                    graphics_state.fill_color = crate::types::color::Color::new(r, g, b);
                }
            }
            ContentOp::Gfill => {
                if !op.operands.is_empty() {
                    let v = (op.operands[0].clamp(0.0, 1.0) * 255.0) as u8;
                    graphics_state.fill_color = crate::types::color::Color::new(v, v, v);
                }
            }
            ContentOp::Kfill => {
                if op.operands.len() >= 4 {
                    let c = op.operands[0]; let m = op.operands[1];
                    let y = op.operands[2]; let k = op.operands[3];
                    let r = ((1.0 - c) * (1.0 - k) * 255.0) as u8;
                    let g = ((1.0 - m) * (1.0 - k) * 255.0) as u8;
                    let b = ((1.0 - y) * (1.0 - k) * 255.0) as u8;
                    graphics_state.fill_color = crate::types::color::Color::new(r, g, b);
                }
            }
            ContentOp::SCN | ContentOp::SC => {
                if op.operands.len() >= 3 {
                    let r = (op.operands[0].clamp(0.0, 1.0) * 255.0) as u8;
                    let g = (op.operands[1].clamp(0.0, 1.0) * 255.0) as u8;
                    let b = (op.operands[2].clamp(0.0, 1.0) * 255.0) as u8;
                    graphics_state.fill_color = crate::types::color::Color::new(r, g, b);
                }
            }
            ContentOp::RE => {
                if op.operands.len() >= 4 {
                    pending_rect = Some((op.operands[0], op.operands[1], op.operands[2], op.operands[3]));
                }
            }
            ContentOp::F => { pending_rect = None; }
            ContentOp::Do => {
                // Nested Form XObjects or images within the form
                if let Some(ref nested_name) = op.font_name {
                    if let Some(img) = form.images.get(nested_name).or_else(|| page.images.get(nested_name)) {
                        let (x, y) = ctm.transform_point(0.0, 0.0);
                        let (x2, _) = ctm.transform_point(1.0, 0.0);
                        let (_, y2) = ctm.transform_point(0.0, 1.0);
                        let w = (x2 - x).abs();
                        let h = (y2 - y).abs();
                        let img_y = page_height - y.max(y2);

                        use base64::Engine;
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&img.data);
                        let data_uri = format!("data:{};base64,{}", img.mime_type, b64);

                        rendered_images.push(PageImageRef {
                            name: nested_name.clone(),
                            x, y: img_y, width: w, height: h, data_uri,
                        });
                    }
                    // Nested form XObject
                    else if let Some(nested_form) = form.form_xobjects.get(nested_name) {
                        if let Ok(nested_result) = render_form_xobject(
                            nested_form, &ctm, page_width, page_height,
                            &graphics_state, &current_font_name, page,
                        ) {
                            text_extractor.merge_spans(&nested_result.text_spans);
                            rendered_images.extend(nested_result.images);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    text_extractor.finalize_segment();

    Ok(FormRenderResult {
        text_spans: text_extractor.get_spans(),
        images: rendered_images,
        background_color,
    })
}

pub fn extract_fonts_from_document(
    document: &PdfDocument,
    output_bundle: &mut OutputBundle,
    pdf_data: &[u8],
) -> Result<(), OdeError> {
    let mut font_processor = FontProcessor::new();

    if let Some(xref) = &document.xref {
        for page in &document.pages {
            let resolver = crate::parser::PdfRefResolver::new(pdf_data, xref);
            for font_ref in &page.fonts {
                if let Ok(font_info) = font_processor.extract_font_from_pdf(*font_ref, &resolver) {
                    if font_info.embedded {
                        if let Some(extracted_font) = font_processor.get_font(font_info.id) {
                            output_bundle.add_font(
                                font_info.id,
                                font_info.name.clone(),
                                extracted_font.data.clone(),
                                extracted_font.format,
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn parse_content_stream(content_stream: &[u8]) -> Result<Vec<ParsedOp>, OdeError> {
    // Detect zlib compression by checking for the zlib header magic bytes.
    // Common zlib headers: 0x78 0x01 (low), 0x78 0x9C (default), 0x78 0xDA (best).
    let compression = if content_stream.len() >= 2 && content_stream[0] == 0x78 {
        Some("FlateDecode")
    } else {
        None
    };
    let mut parser = ContentStreamParser::new(content_stream, compression)?;
    parser.parse()
}

fn generate_page_html_with_text(
    _page_number: usize,
    width: f64,
    height: f64,
    _config: &ConversionConfig,
    text_spans: &Vec<TextSpan>,
) -> String {
    let spans_html: Vec<String> = text_spans
        .iter()
        .map(|span| {
            format!(
                "<span style=\"left:{}px;top:{}px;font-size:{}px;color:{};\">{}</span>",
                span.x,
                span.y,
                span.font_size,
                span.color,
                escape_html(&span.text)
            )
        })
        .collect();

    format!(
        "<div style=\"width:{}px;height:{}px;position:relative;\">{}</div>",
        width,
        height,
        spans_html.join("")
    )
}

fn generate_page_html_with_images_and_text(
    _page_number: usize,
    width: f64,
    height: f64,
    _config: &ConversionConfig,
    text_spans: &[TextSpan],
    images: &[PageImageRef],
) -> String {
    let mut inner_html = String::new();

    // Render images first (they go behind text)
    for img in images {
        inner_html.push_str(&format!(
            "<img style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;\" src=\"{}\">",
            img.x, img.y, img.width, img.height, img.data_uri
        ));
    }

    // Render text spans on top
    for span in text_spans {
        inner_html.push_str(&format!(
            "<span style=\"left:{}px;top:{}px;font-size:{}px;color:{};\">{}</span>",
            span.x, span.y, span.font_size, span.color, escape_html(&span.text)
        ));
    }

    format!(
        "<div style=\"width:{}px;height:{}px;position:relative;\">{}</div>",
        width, height, inner_html
    )
}

fn generate_page_css(_config: &ConversionConfig) -> String {
    String::from(".page span { position:absolute; white-space: nowrap; }")
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_addressed_file_naming() {
        let font_data = b"test font data";
        let hash = ContentHasher::hash_bytes(font_data);
        let filename = ContentHasher::generate_content_addressed_filename(font_data, "woff2");

        assert!(filename.starts_with(&hash));
        assert!(filename.ends_with(".woff2"));
    }

    #[test]
    fn test_output_bundle_font_storage() {
        let mut bundle = OutputBundle::default();

        let font_data = vec![1, 2, 3, 4, 5];
        bundle.add_font(
            0,
            "TestFont".to_string(),
            font_data.clone(),
            crate::config::FontFormat::Woff2,
        );

        assert_eq!(bundle.fonts.len(), 1);

        let font = bundle.get_font_by_id(0).unwrap();
        assert_eq!(font.font_id, 0);
        assert_eq!(font.font_name, "TestFont");
        assert_eq!(font.data, font_data);
        assert!(font.filename.ends_with(".woff2"));
        assert_eq!(font.filename.len(), 64 + 1 + 5); // 64 char SHA256 + "." + "woff2"
    }
}
