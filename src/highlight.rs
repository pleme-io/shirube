//! Apply highlights, extmarks, and signs to a Neovim buffer.

use nvim_oxi::api;
use nvim_oxi::api::opts::SetExtmarkOpts;
use nvim_oxi::api::Buffer;

use crate::keywords::Keyword;
use crate::scanner;

/// Register all `Shirube*` highlight groups via the tane SDK.
pub fn define_highlight_groups() -> nvim_oxi::Result<()> {
    for kw in Keyword::ALL {
        tane::highlight::Highlight::new(kw.hl_group())
            .fg(kw.fg_color())
            .bg(kw.bg_color())
            .bold()
            .apply()
            .map_err(|e| {
                nvim_oxi::Error::from(nvim_oxi::api::Error::Other(e.to_string()))
            })?;
    }
    Ok(())
}

/// Clear all shirube extmarks from `buf` in the given namespace, then
/// re-scan every line and place fresh highlights + signs.
pub fn highlight_buffer(buf: &mut Buffer, ns_id: u32) -> nvim_oxi::Result<()> {
    // Clear previous marks.
    buf.clear_namespace(ns_id, ..)?;

    let line_count = buf.line_count()?;
    if line_count == 0 {
        return Ok(());
    }

    let lines: Vec<_> = buf.get_lines(0..line_count, false)?.collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let line_str = line.to_string_lossy();
        let matches = scanner::scan_line(&line_str);

        for m in matches {
            // Inline highlight on the keyword text.
            let mut opts = SetExtmarkOpts::builder();
            opts.end_col(m.byte_offset + m.byte_len);
            opts.hl_group(m.keyword.hl_group());
            opts.sign_text(m.keyword.sign_text());
            opts.sign_hl_group(m.keyword.hl_group());
            opts.priority(200); // above treesitter (100)
            let opts = opts.build();

            buf.set_extmark(ns_id, line_idx, m.byte_offset, &opts)?;
        }
    }

    Ok(())
}

/// Collect every keyword occurrence across all loaded buffers.
///
/// Returns a list of `(file_path, line_number_1_indexed, keyword, line_text)`.
pub fn collect_all_todos() -> nvim_oxi::Result<Vec<(String, usize, Keyword, String)>> {
    let mut results = Vec::new();

    for buf in api::list_bufs() {
        // Skip unlisted / scratch buffers.
        let name = buf.get_name().map_err(nvim_oxi::Error::from)?;
        let name_str = name.to_string_lossy().to_string();
        if name_str.is_empty() {
            continue;
        }

        let line_count = buf.line_count().map_err(nvim_oxi::Error::from)?;
        if line_count == 0 {
            continue;
        }

        let lines: Vec<_> = buf
            .get_lines(0..line_count, false)
            .map_err(nvim_oxi::Error::from)?
            .collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_str = line.to_string_lossy();
            for m in scanner::scan_line(&line_str) {
                results.push((
                    name_str.clone(),
                    line_idx + 1,
                    m.keyword,
                    line_str.to_string(),
                ));
            }
        }
    }

    Ok(results)
}
