pub mod config;
pub mod error;
pub mod fonts;
pub mod parser;
pub mod render;
pub mod renderer;
pub mod types;
pub mod util;

#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod us_acceptance_tests;
#[cfg(test)]
mod coordinate_tests;
#[cfg(test)]
mod memory_tests;
#[cfg(test)]
mod safety_audit;
#[cfg(test)]
mod benchmarks;

pub use config::ConversionConfig;
pub use error::{OdeError, OdeResult};
pub use parser::PdfDocument;
pub use renderer::OutputBundle;
pub use render::CoveredTextDetector;
pub use util::{LigatureMapper, TimeoutWrapper, ZipBombDetector};

use crate::fonts::FontProcessor;
use crate::renderer::extract_fonts_from_document;

pub fn convert_pdf(data: &[u8], config: &ConversionConfig) -> OdeResult<OutputBundle> {
    // Zip bomb detection happens per-stream during decompression,
    // not on the raw PDF file (which is not itself a compressed blob).

    let document = parser::parse_pdf(data)?;

    let mut output_bundle = OutputBundle::default();

    if let Some(xref) = &document.xref {
        let mut font_processor = FontProcessor::new();
        
        for page in &document.pages {
            for font_ref in &page.fonts {
                let resolver = parser::PdfRefResolver::new(data, xref);
                let _ = font_processor.extract_font_from_pdf(*font_ref, &resolver);
            }
        }
    }

    let start_page = config.page_range.0.saturating_sub(1);
    let end_page = (config.page_range.1.min(document.num_pages()))
        .min(start_page + 1000);

    for page_id in start_page..end_page {
        let page_number = page_id + 1;
        let rendered_page = renderer::render_pdf_page(
            &document,
            page_id,
            page_number,
            config,
        )?;

        output_bundle.add_page(rendered_page);
    }

    let _ = extract_fonts_from_document(&document, &mut output_bundle, data);

    Ok(output_bundle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_pdf_basic() {
        let pdf_data = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF";
        let config = ConversionConfig::default();
        let result = convert_pdf(pdf_data, &config);
        assert!(result.is_ok());
    }
}