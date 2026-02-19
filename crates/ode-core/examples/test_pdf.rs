use std::fs;

fn main() {
    let paths = [
        "/tmp/modernpath-deck.pdf",
        "/Users/pasivuorio/modernpath/pdf-new/2026 02 16 ModernPath deck s.pdf",
    ];

    let mut data = None;
    for path in &paths {
        if let Ok(d) = fs::read(path) {
            eprintln!("Read PDF from: {} ({} bytes)", path, d.len());
            data = Some(d);
            break;
        }
    }

    let data = data.expect("Could not find PDF file");

    let config = ode_core::ConversionConfig::default();
    match ode_core::convert_pdf(&data, &config) {
        Ok(bundle) => {
            eprintln!("\nSuccess! {} pages\n", bundle.pages.len());
            for (i, page) in bundle.pages.iter().enumerate() {
                eprintln!(
                    "Page {}: {} spans, {} bytes HTML",
                    i + 1,
                    page.text_spans.len(),
                    page.html.len()
                );
                for (j, span) in page.text_spans.iter().enumerate().take(5) {
                    eprintln!(
                        "  Span {}: ({:.1}, {:.1}) size={:.1} color={} text_len={}",
                        j, span.x, span.y, span.font_size, span.color, span.text.len()
                    );
                }
                if i >= 3 {
                    break;
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}
