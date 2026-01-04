use naga::{Module, Statement};
use std::collections::HashSet;

/// Generate the JS stub for shader debugging
pub struct JsStubGenerator<'a> {
    source: &'a str,
    module: &'a Module,
    name: Option<&'a str>,
    call_lines: HashSet<usize>,
}

impl<'a> JsStubGenerator<'a> {
    pub fn new(source: &'a str, module: &'a Module, name: Option<&'a str>) -> Self {
        let mut generator = Self {
            source,
            module,
            name,
            call_lines: HashSet::new(),
        };
        generator.analyze_calls();
        generator
    }

    fn analyze_calls(&mut self) {
        for (_, func) in self.module.functions.iter() {
            self.scan_block(&func.body);
        }
        for ep in self.module.entry_points.iter() {
            self.scan_block(&ep.function.body);
        }
    }

    fn scan_block(&mut self, block: &naga::Block) {
        for (stmt, span) in block.span_iter() {
            match stmt {
                Statement::Block(b) => self.scan_block(b),
                Statement::If { accept, reject, .. } => {
                    self.scan_block(accept);
                    self.scan_block(reject);
                }
                Statement::Switch { cases, .. } => {
                    for case in cases {
                        self.scan_block(&case.body);
                    }
                }
                Statement::Loop {
                    body, continuing, ..
                } => {
                    self.scan_block(body);
                    self.scan_block(continuing);
                }
                Statement::Call { .. } => {
                    if let Some(line) = self.get_line_from_span(span) {
                        self.call_lines.insert(line);
                    }
                }
                _ => {}
            }
        }
    }

    fn get_line_from_span(&self, span: &naga::Span) -> Option<usize> {
        Some(span.location(self.source).line_number as usize)
    }

    pub fn generate(&self) -> String {
        let mut js = String::from("[");
        let lines: Vec<&str> = self.source.lines().collect();
        let mut mappings = String::new();

        // Initial state for VLQ delta encoding
        let mut prev_source_idx = 0;
        let mut prev_source_line = 0;
        let mut prev_source_col = 0;

        // We generate one entry per line of source code
        for (i, _line_content) in lines.iter().enumerate() {
            let line_num = i + 1;
            let is_call = self.call_lines.contains(&line_num);

            let params = "";

            // JS Line: "  (...) => ...,"
            // We map the start of this line (col 0) to the start of the source line (col 0)

            // Generated column
            // Line 0 starts after "[", so col 1. Others start at 0.
            let gen_col = if i == 0 { 1 } else { 0 };
            let source_idx = 0;
            let source_line = i as i32; // 0-based
            let source_col = 0;

            // VLQ fields:
            // 1. Column in generated file (relative to previous in this line)
            // 2. Source file index (relative to previous)
            // 3. Source line index (relative to previous)
            // 4. Source column index (relative to previous)
            // 5. Name index (optional)

            // Since we start a new line in generated code (except first), the column resets to 0.
            // For the first line, we start at col 1.
            let col_delta = gen_col;

            let seg = vec![
                col_delta,
                source_idx - prev_source_idx,
                source_line - prev_source_line,
                source_col - prev_source_col,
            ];

            let mut encoded_seg = String::new();
            for val in seg {
                encoded_seg.push_str(&encode_vlq(val));
            }

            mappings.push_str(&encoded_seg);
            mappings.push(';');

            // Update state
            prev_source_idx = source_idx;
            prev_source_line = source_line;
            prev_source_col = source_col;

            if is_call {
                js.push_str(&format!("({}) => this?.go?.(),", params));
            } else {
                js.push_str(&format!("({}) => {{}},", params));
            }
            js.push('\n');
        }

        js.push(']');

        // Construct Source Map JSON
        // We need to escape the source content for JSON string
        let source_content_json =
            serde_json::to_string(self.source).unwrap_or_else(|_| "\"\"".to_string());
        let filename = self.name.unwrap_or("shader.glsl");

        let map_json = format!(
            r#"{{"version":3,"file":"generated.js","sourceRoot":"","sources":["{}"],"names":[],"mappings":"{}","sourcesContent":[{}]}}"#,
            filename, mappings, source_content_json
        );

        let b64_map = base64_encode(map_json.as_bytes());
        js.push_str(&format!(
            "\n//# sourceMappingURL=data:application/json;base64,{}",
            b64_map
        ));

        js
    }
}

fn encode_vlq(value: i32) -> String {
    let mut vlq = String::new();
    let mut val = value as i64;

    // Zigzag encode
    val = if val < 0 { ((-val) << 1) | 1 } else { val << 1 };

    loop {
        let mut digit = (val & 0x1F) as u8;
        val >>= 5;
        if val > 0 {
            digit |= 0x20;
        }
        // Base64 encode digit
        vlq.push(BASE64_CHARS[digit as usize] as char);
        if val == 0 {
            break;
        }
    }
    vlq
}

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let idx0 = b0 >> 2;
        let idx1 = ((b0 & 0x03) << 4) | (b1 >> 4);
        let idx2 = ((b1 & 0x0F) << 2) | (b2 >> 6);
        let idx3 = b2 & 0x3F;

        out.push(BASE64_CHARS[idx0 as usize] as char);
        out.push(BASE64_CHARS[idx1 as usize] as char);

        if i + 1 < data.len() {
            out.push(BASE64_CHARS[idx2 as usize] as char);
        } else {
            out.push('=');
        }

        if i + 2 < data.len() {
            out.push(BASE64_CHARS[idx3 as usize] as char);
        } else {
            out.push('=');
        }

        i += 3;
    }
    out
}

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
