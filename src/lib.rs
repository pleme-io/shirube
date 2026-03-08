//! Shirube (標) — TODO/FIXME/HACK comment highlighter and navigator for Neovim
//!
//! Part of the blnvim-ng distribution — a Rust-native Neovim plugin suite.
//! Built with [`nvim-oxi`](https://github.com/noib3/nvim-oxi) for zero-cost
//! Neovim API bindings.

pub mod highlight;
pub mod keywords;
pub mod scanner;

use nvim_oxi as oxi;
use nvim_oxi::api;
use tane::prelude::*;

/// Convert a `tane::Error` into an `oxi::Error`.
fn tane_err(e: tane::Error) -> oxi::Error {
    oxi::Error::from(oxi::api::Error::Other(e.to_string()))
}

/// Highlight the current buffer's TODO keywords.
fn highlight_current_buffer(ns_id: u32) -> oxi::Result<()> {
    let mut buf = api::get_current_buf();
    highlight::highlight_buffer(&mut buf, ns_id)
}

/// Format and display the `:ShirubeList` output.
fn list_todos() -> oxi::Result<()> {
    let todos = highlight::collect_all_todos()?;

    if todos.is_empty() {
        api::out_write("shirube: no TODO/FIXME keywords found\n");
        return Ok(());
    }

    let mut output = String::new();
    for (file, line, kw, text) in &todos {
        output.push_str(&format!(
            "{file}:{line} [{kw}] {}\n",
            text.trim()
        ));
    }
    api::out_write(output.as_str());

    Ok(())
}

#[oxi::plugin]
fn shirube() -> oxi::Result<()> {
    // 1. Define highlight groups.
    highlight::define_highlight_groups()?;

    // 2. Create a namespace for our extmarks.
    let ns = Namespace::create("shirube").map_err(tane_err)?;
    let ns_id = ns.id();

    // 3. Register autocommands to highlight on buffer enter and after write.
    Autocmd::on(&["BufEnter", "BufWritePost"])
        .pattern("*")
        .group("shirube")
        .desc("Highlight TODO/FIXME keywords")
        .register(move |_args| {
            let _ = highlight_current_buffer(ns_id);
            Ok(false) // keep the autocommand
        })
        .map_err(tane_err)?;

    // Also highlight after text changes (in normal mode, after InsertLeave).
    Autocmd::on(&["InsertLeave", "TextChanged"])
        .pattern("*")
        .group("shirube")
        .desc("Re-highlight TODO/FIXME keywords after edits")
        .register(move |_args| {
            let _ = highlight_current_buffer(ns_id);
            Ok(false)
        })
        .map_err(tane_err)?;

    // 4. Register :ShirubeList command.
    UserCommand::new("ShirubeList")
        .desc("List all TODO/FIXME keywords across open buffers")
        .bar()
        .register(move |_args| {
            let _ = list_todos();
            Ok(())
        })
        .map_err(tane_err)?;

    Ok(())
}
