use crate::error::OdeError;

use super::{Dictionary, ObjectReference, PdfObject, PdfPage, PdfRefResolver};

/// Inherited properties from parent Pages nodes in the page tree.
/// Per the PDF spec, Resources, MediaBox, CropBox, and Rotate are inheritable.
#[derive(Clone, Default)]
struct InheritedProps {
    resources: Option<Dictionary>,
    mediabox: Option<Vec<PdfObject>>,
    cropbox: Option<Vec<PdfObject>>,
    rotate: Option<i32>,
}

pub struct PageTreeParser<'a> {
    resolver: &'a PdfRefResolver<'a>,
}

impl<'a> PageTreeParser<'a> {
    pub fn new(resolver: &'a PdfRefResolver<'a>) -> Self {
        Self { resolver }
    }

    pub fn parse_all_pages(&self, root_ref: ObjectReference) -> Result<Vec<PdfPage>, OdeError> {
        let mut pages = Vec::new();
        let inherited = InheritedProps::default();
        self.traverse_page_tree(root_ref, &mut pages, 0, &inherited)?;
        Ok(pages)
    }

    fn traverse_page_tree(
        &self,
        node_ref: ObjectReference,
        pages: &mut Vec<PdfPage>,
        depth: u32,
        inherited: &InheritedProps,
    ) -> Result<(), OdeError> {
        if depth > 100 {
            return Err(OdeError::PdfParseError("Page tree too deep".to_string()));
        }

        let node_obj = self.resolver.dereference(node_ref).ok_or_else(|| {
            OdeError::PdfParseError(format!("Cannot dereference node {:?}", node_ref))
        })?;

        match &node_obj {
            PdfObject::Dictionary(dict) => {
                let type_name = dict.get("Type").and_then(|v| v.as_name());

                match type_name {
                    Some("Page") => {
                        let page = self.parse_page(dict, pages.len(), inherited)?;
                        pages.push(page);
                    }
                    Some("Pages") => {
                        self.parse_pages_node(dict, pages, depth, inherited)?;
                    }
                    _ => {
                        return Err(OdeError::PdfParseError(format!(
                            "Invalid page tree node type: {:?}",
                            type_name
                        )))
                    }
                }
            }
            _ => {
                return Err(OdeError::PdfParseError(
                    "Page tree node is not a dictionary".to_string(),
                ))
            }
        }

        Ok(())
    }

    fn parse_pages_node(
        &self,
        dict: &Dictionary,
        pages: &mut Vec<PdfPage>,
        depth: u32,
        parent_inherited: &InheritedProps,
    ) -> Result<(), OdeError> {
        // Build inherited props for children: this node's properties override parent's
        let mut child_inherited = parent_inherited.clone();

        if let Some(resources_obj) = dict.get("Resources") {
            if let Some(resolved) = self.resolve_dict(resources_obj) {
                child_inherited.resources = Some(resolved);
            }
        }

        if let Some(mediabox) = dict.get("MediaBox") {
            if let Some(arr) = mediabox.as_array() {
                child_inherited.mediabox = Some(arr.clone());
            }
        }

        if let Some(cropbox) = dict.get("CropBox") {
            if let Some(arr) = cropbox.as_array() {
                child_inherited.cropbox = Some(arr.clone());
            }
        }

        if let Some(rot) = dict.get("Rotate") {
            if let Some(r) = rot.as_number() {
                child_inherited.rotate = Some(r as i32);
            }
        }

        let kids_value = dict.get("Kids");

        let kids_array = match kids_value {
            Some(PdfObject::Array(arr)) => Some(arr.clone()),
            Some(obj) => {
                if let Some(kids_ref) = obj.as_reference() {
                    if let Some(PdfObject::Array(arr)) = self.resolver.dereference(kids_ref) {
                        Some(arr)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => None,
        };

        if let Some(kids) = kids_array {
            for kid in &kids {
                if let Some(kid_ref) = kid.as_reference() {
                    self.traverse_page_tree(kid_ref, pages, depth + 1, &child_inherited)?;
                }
            }
        }

        Ok(())
    }

    /// Resolve a PdfObject to a Dictionary, dereferencing indirect references if needed
    fn resolve_dict(&self, obj: &PdfObject) -> Option<Dictionary> {
        match obj {
            PdfObject::Dictionary(d) => Some(d.clone()),
            _ => {
                if let Some(r) = obj.as_reference() {
                    if let Some(PdfObject::Dictionary(d)) = self.resolver.dereference(r) {
                        Some(d)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn parse_page(
        &self,
        dict: &Dictionary,
        page_number: usize,
        inherited: &InheritedProps,
    ) -> Result<PdfPage, OdeError> {
        let mut width = 612.0;
        let mut height = 792.0;
        let mut contents = Vec::new();
        let mut fonts = Vec::new();
        let mut rotation = inherited.rotate.unwrap_or(0);

        // MediaBox: page's own or inherited
        let mediabox = dict.get("MediaBox")
            .and_then(|v| v.as_array().cloned())
            .or_else(|| inherited.mediabox.clone());

        if let Some(box_array) = mediabox {
            if box_array.len() >= 4 {
                if let (Some(x0), Some(y0), Some(x1), Some(y1)) = (
                    box_array.get(0).and_then(|v| v.as_number()),
                    box_array.get(1).and_then(|v| v.as_number()),
                    box_array.get(2).and_then(|v| v.as_number()),
                    box_array.get(3).and_then(|v| v.as_number()),
                ) {
                    width = (x1 - x0).max(1.0);
                    height = (y1 - y0).max(1.0);
                }
            }
        }

        // CropBox: page's own or inherited (overrides MediaBox dimensions)
        let cropbox = dict.get("CropBox")
            .and_then(|v| v.as_array().cloned())
            .or_else(|| inherited.cropbox.clone());

        if let Some(box_array) = cropbox {
            if box_array.len() >= 4 {
                if let (Some(x0), Some(y0), Some(x1), Some(y1)) = (
                    box_array.get(0).and_then(|v| v.as_number()),
                    box_array.get(1).and_then(|v| v.as_number()),
                    box_array.get(2).and_then(|v| v.as_number()),
                    box_array.get(3).and_then(|v| v.as_number()),
                ) {
                    width = (x1 - x0).max(1.0);
                    height = (y1 - y0).max(1.0);
                }
            }
        }

        if let Some(rot) = dict.get("Rotate") {
            if let Some(r) = rot.as_number() {
                rotation = r as i32;
            }
        }

        if let Some(contents_obj) = dict.get("Contents") {
            contents = self.extract_content_stream(contents_obj)?;
        }

        // Resources: page's own or inherited
        let resources_dict = dict.get("Resources")
            .and_then(|obj| self.resolve_dict(obj))
            .or_else(|| inherited.resources.clone());

        // Extract fonts from resources
        if let Some(ref res_dict) = resources_dict {
            let font_dict_obj = res_dict.get("Font");
            let font_dict = font_dict_obj.and_then(|obj| self.resolve_dict(obj));
            if let Some(fd) = font_dict {
                for (_key, value) in &fd.entries {
                    if let Some(font_ref) = value.as_reference() {
                        fonts.push(font_ref);
                    }
                }
            }
        }

        // Extract ToUnicode CMaps for each font
        let mut font_cmaps = std::collections::HashMap::new();
        if let Some(ref res_dict) = resources_dict {
            let font_dict = res_dict.get("Font").and_then(|obj| self.resolve_dict(obj));
            if let Some(fd) = font_dict {
                for (font_name, font_ref_obj) in &fd.entries {
                    if let Some(font_ref) = font_ref_obj.as_reference() {
                        if let Some(font_obj) = self.resolver.dereference(font_ref) {
                            if let PdfObject::Dictionary(ref font_d) = font_obj {
                                if let Some(tounicode_ref_obj) = font_d.get("ToUnicode") {
                                    if let Some(tounicode_ref) = tounicode_ref_obj.as_reference() {
                                        if let Some(PdfObject::Stream(cmap_data, _)) = self.resolver.dereference(tounicode_ref) {
                                            let cmap = super::ToUnicodeCMap::parse(&cmap_data);
                                            font_cmaps.insert(font_name.clone(), cmap);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Extract XObjects (images and forms) from Resources
        let (images, form_xobjects) = self.extract_xobjects(&resources_dict);

        Ok(PdfPage {
            page_number,
            width,
            height,
            contents,
            fonts,
            rotation,
            dict: Some(dict.clone()),
            font_cmaps,
            images,
            form_xobjects,
        })
    }

    fn extract_xobjects(
        &self,
        resources_dict: &Option<Dictionary>,
    ) -> (
        std::collections::HashMap<String, super::PageImage>,
        std::collections::HashMap<String, super::FormXObject>,
    ) {
        let mut images = std::collections::HashMap::new();
        let mut form_xobjects = std::collections::HashMap::new();

        let res_dict = match resources_dict {
            Some(d) => d,
            None => return (images, form_xobjects),
        };

        let xobj_dict = match res_dict.get("XObject").and_then(|obj| self.resolve_dict(obj)) {
            Some(d) => d,
            None => return (images, form_xobjects),
        };

        for (xobj_name, xobj_ref_obj) in &xobj_dict.entries {
            let xobj_ref = match xobj_ref_obj.as_reference() {
                Some(r) => r,
                None => continue,
            };
            let xobj_obj = match self.resolver.dereference(xobj_ref) {
                Some(o) => o,
                None => continue,
            };
            if let PdfObject::Stream(data, stream_dict) = &xobj_obj {
                let subtype = stream_dict.get("Subtype").and_then(|v| v.as_name());

                if subtype == Some("Image") {
                    let img_w = stream_dict.get("Width").and_then(|v| v.as_number()).unwrap_or(0.0) as u32;
                    let img_h = stream_dict.get("Height").and_then(|v| v.as_number()).unwrap_or(0.0) as u32;
                    let filter = stream_dict.get("Filter").and_then(|v| v.as_name()).unwrap_or("");

                    let (img_data, mime) = match filter {
                        "DCTDecode" => (data.clone(), "image/jpeg"),
                        "JPXDecode" => (data.clone(), "image/jp2"),
                        "FlateDecode" => {
                            let colorspace = stream_dict.get("ColorSpace").and_then(|v| v.as_name()).unwrap_or("DeviceRGB");
                            let channels: u8 = match colorspace {
                                "DeviceGray" => 1,
                                "DeviceRGB" => 3,
                                "DeviceCMYK" => 4,
                                _ => 3,
                            };
                            let png_data = encode_raw_pixels_as_png(data, img_w, img_h, channels);
                            (png_data, "image/png")
                        }
                        _ => continue,
                    };

                    images.insert(xobj_name.clone(), super::PageImage {
                        name: xobj_name.clone(),
                        data: img_data,
                        width: img_w,
                        height: img_h,
                        mime_type: mime.to_string(),
                    });
                } else if subtype == Some("Form") {
                    // Extract BBox
                    let bbox = stream_dict.get("BBox")
                        .and_then(|v| if let PdfObject::Array(arr) = v {
                            if arr.len() >= 4 {
                                Some([
                                    arr[0].as_number().unwrap_or(0.0),
                                    arr[1].as_number().unwrap_or(0.0),
                                    arr[2].as_number().unwrap_or(0.0),
                                    arr[3].as_number().unwrap_or(0.0),
                                ])
                            } else { None }
                        } else { None })
                        .unwrap_or([0.0, 0.0, 0.0, 0.0]);

                    // Extract Matrix (optional)
                    let matrix = stream_dict.get("Matrix")
                        .and_then(|v| if let PdfObject::Array(arr) = v {
                            if arr.len() >= 6 {
                                Some([
                                    arr[0].as_number().unwrap_or(1.0),
                                    arr[1].as_number().unwrap_or(0.0),
                                    arr[2].as_number().unwrap_or(0.0),
                                    arr[3].as_number().unwrap_or(1.0),
                                    arr[4].as_number().unwrap_or(0.0),
                                    arr[5].as_number().unwrap_or(0.0),
                                ])
                            } else { None }
                        } else { None });

                    // Extract form's own Resources
                    let form_resources = stream_dict.get("Resources")
                        .and_then(|obj| self.resolve_dict(obj));

                    // Extract font CMaps from form's resources
                    let mut form_font_cmaps = std::collections::HashMap::new();
                    if let Some(ref form_res) = form_resources {
                        let font_dict = form_res.get("Font").and_then(|obj| self.resolve_dict(obj));
                        if let Some(fd) = font_dict {
                            for (font_name, font_ref_obj) in &fd.entries {
                                if let Some(font_ref) = font_ref_obj.as_reference() {
                                    if let Some(font_obj) = self.resolver.dereference(font_ref) {
                                        if let PdfObject::Dictionary(ref font_d) = font_obj {
                                            if let Some(tounicode_ref_obj) = font_d.get("ToUnicode") {
                                                if let Some(tounicode_ref) = tounicode_ref_obj.as_reference() {
                                                    if let Some(PdfObject::Stream(cmap_data, _)) = self.resolver.dereference(tounicode_ref) {
                                                        let cmap = super::ToUnicodeCMap::parse(&cmap_data);
                                                        form_font_cmaps.insert(font_name.clone(), cmap);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Recursively extract nested XObjects from form's resources
                    let (form_images, nested_forms) = self.extract_xobjects(&form_resources);

                    form_xobjects.insert(xobj_name.clone(), super::FormXObject {
                        name: xobj_name.clone(),
                        content_stream: data.clone(),
                        resources: form_resources,
                        bbox,
                        matrix,
                        font_cmaps: form_font_cmaps,
                        images: form_images,
                        form_xobjects: nested_forms,
                    });
                }
            }
        }

        (images, form_xobjects)
    }

    fn extract_content_stream(&self, contents: &PdfObject) -> Result<Vec<u8>, OdeError> {
        match contents {
            PdfObject::Stream(ref data, _dict) => Ok(data.clone()),
            PdfObject::IndirectReference { obj_id, gen } => {
                let obj = self.resolver.dereference(super::ObjectReference(*obj_id, *gen))
                    .ok_or_else(|| OdeError::PdfParseError(
                        format!("Cannot dereference /Contents {} {} R", obj_id, gen)
                    ))?;
                match obj {
                    PdfObject::Stream(data, _dict) => Ok(data),
                    _ => Ok(Vec::new()),
                }
            }
            PdfObject::Array(refs) => {
                let mut combined = Vec::new();
                for ref_item in refs {
                    if let Some(ref_obj_ref) = ref_item.as_reference() {
                        if let Some(ref_obj) = self.resolver.dereference(ref_obj_ref) {
                            if let PdfObject::Stream(ref data, _dict) = ref_obj {
                                combined.extend(data.clone());
                                combined.extend_from_slice(b"\n");
                            }
                        }
                    }
                }
                Ok(combined)
            }
            PdfObject::String(data) => Ok(data.as_bytes().to_vec()),
            _ => Ok(Vec::new()),
        }
    }
}

/// Encode raw pixel data as a valid PNG file.
fn encode_raw_pixels_as_png(pixels: &[u8], width: u32, height: u32, channels: u8) -> Vec<u8> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut png = Vec::new();

    // PNG signature
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    // IHDR chunk
    let color_type: u8 = match channels {
        1 => 0, // Grayscale
        3 => 2, // RGB
        4 => 6, // RGBA
        _ => 2,
    };
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(color_type);
    ihdr.push(0); // compression method
    ihdr.push(0); // filter method
    ihdr.push(0); // interlace method
    write_png_chunk(&mut png, b"IHDR", &ihdr);

    // IDAT chunk: add filter byte (0=None) before each row, then deflate
    let row_bytes = (width as usize) * (channels as usize);
    let mut raw_with_filter = Vec::with_capacity(pixels.len() + height as usize);
    for row in 0..height as usize {
        raw_with_filter.push(0); // filter type = None
        let start = row * row_bytes;
        let end = start + row_bytes;
        if end <= pixels.len() {
            raw_with_filter.extend_from_slice(&pixels[start..end]);
        } else {
            let available = pixels.len().saturating_sub(start);
            if available > 0 {
                raw_with_filter.extend_from_slice(&pixels[start..start + available]);
            }
            raw_with_filter.resize(raw_with_filter.len() + row_bytes - available, 0);
        }
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(&raw_with_filter);
    let compressed = encoder.finish().unwrap_or_default();
    write_png_chunk(&mut png, b"IDAT", &compressed);

    // IEND chunk
    write_png_chunk(&mut png, b"IEND", &[]);

    png
}

fn write_png_chunk(output: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(chunk_type);
    output.extend_from_slice(data);
    let crc = png_crc32(chunk_type, data);
    output.extend_from_slice(&crc.to_be_bytes());
}

fn png_crc32(chunk_type: &[u8], data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in chunk_type.iter().chain(data.iter()) {
        let index = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = PNG_CRC_TABLE[index] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

const PNG_CRC_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut n = 0usize;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            if c & 1 != 0 {
                c = 0xEDB88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
            k += 1;
        }
        table[n] = c;
        n += 1;
    }
    table
};
