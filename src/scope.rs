//! Language-aware scope resolution via tree-sitter. Given a file and a changed line,
//! walk the syntax tree from the enclosing node up to the root and collect the named
//! definitions along the way — a precise breadcrumb like `class Camera > setZoom()`
//! that git's line-based hunk headers cannot produce (they only see top-level defs).
//!
//! Supported languages fall back to git's `-U0` scope when a file's extension isn't
//! covered here (see caller in `main`).

use std::path::Path;
use tree_sitter::{Language, Node, Parser, Point};

#[derive(Clone, Copy)]
enum Lang {
    Rust,
    Ts,
    Tsx,
    Js,
    Py,
    Go,
}

impl Lang {
    fn from_path(p: &Path) -> Option<Lang> {
        match p.extension()?.to_str()? {
            "rs" => Some(Lang::Rust),
            "ts" | "mts" | "cts" => Some(Lang::Ts),
            "tsx" => Some(Lang::Tsx),
            "js" | "jsx" | "mjs" | "cjs" => Some(Lang::Js),
            "py" | "pyi" => Some(Lang::Py),
            "go" => Some(Lang::Go),
            _ => None,
        }
    }

    fn language(self) -> Language {
        match self {
            Lang::Rust => tree_sitter_rust::LANGUAGE.into(),
            Lang::Ts => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Lang::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Lang::Js => tree_sitter_javascript::LANGUAGE.into(),
            Lang::Py => tree_sitter_python::LANGUAGE.into(),
            Lang::Go => tree_sitter_go::LANGUAGE.into(),
        }
    }
}

/// Resolve the enclosing-definition breadcrumb for `line` (1-based) in `source`.
/// Returns `None` for unsupported languages or when no named definition encloses the
/// line, so the caller can fall back to git's hunk scope.
pub fn breadcrumb(path: &Path, source: &str, line: u32) -> Option<String> {
    let lang = Lang::from_path(path)?;
    let mut parser = Parser::new();
    parser.set_language(&lang.language()).ok()?;
    let tree = parser.parse(source, None)?;

    let row = line.saturating_sub(1) as usize;
    let pt = Point { row, column: 0 };
    let mut node = tree.root_node().descendant_for_point_range(pt, pt)?;

    let mut crumbs: Vec<String> = Vec::new();
    loop {
        if let Some(label) = def_label(node, source, lang) {
            crumbs.push(label);
        }
        match node.parent() {
            Some(p) => node = p,
            None => break,
        }
    }
    if crumbs.is_empty() {
        return None;
    }
    crumbs.reverse();
    // Keep the breadcrumb tidy: at most the 3 innermost levels.
    if crumbs.len() > 3 {
        crumbs = crumbs.split_off(crumbs.len() - 3);
    }
    Some(crumbs.join(" > "))
}

fn field_text(node: Node, field: &str, src: &str) -> Option<String> {
    node.child_by_field_name(field)?
        .utf8_text(src.as_bytes())
        .ok()
        .map(|s| s.to_string())
}

/// A short label for a definition node, or `None` if this node isn't a definition.
fn def_label(node: Node, src: &str, lang: Lang) -> Option<String> {
    let kind = node.kind();
    match lang {
        Lang::Rust => match kind {
            "function_item" => Some(format!("fn {}", field_text(node, "name", src)?)),
            "impl_item" => {
                let ty = field_text(node, "type", src)?;
                Some(match field_text(node, "trait", src) {
                    Some(tr) => format!("impl {tr} for {ty}"),
                    None => format!("impl {ty}"),
                })
            }
            "struct_item" => Some(format!("struct {}", field_text(node, "name", src)?)),
            "enum_item" => Some(format!("enum {}", field_text(node, "name", src)?)),
            "union_item" => Some(format!("union {}", field_text(node, "name", src)?)),
            "trait_item" => Some(format!("trait {}", field_text(node, "name", src)?)),
            "mod_item" => Some(format!("mod {}", field_text(node, "name", src)?)),
            "macro_definition" => Some(format!("macro {}", field_text(node, "name", src)?)),
            _ => None,
        },
        Lang::Ts | Lang::Tsx | Lang::Js => match kind {
            "function_declaration" | "generator_function_declaration" => {
                Some(format!("fn {}", field_text(node, "name", src)?))
            }
            "method_definition" => Some(format!("{}()", field_text(node, "name", src)?)),
            "class_declaration" | "class" => {
                Some(format!("class {}", field_text(node, "name", src)?))
            }
            // Catches `export const useX = create(...)` / `const handler = () => {}`.
            "variable_declarator" => field_text(node, "name", src),
            _ => None,
        },
        Lang::Py => match kind {
            "function_definition" => Some(format!("def {}", field_text(node, "name", src)?)),
            "class_definition" => Some(format!("class {}", field_text(node, "name", src)?)),
            _ => None,
        },
        Lang::Go => match kind {
            "function_declaration" => Some(format!("func {}", field_text(node, "name", src)?)),
            "method_declaration" => Some(format!("func {}", field_text(node, "name", src)?)),
            "type_spec" => Some(format!("type {}", field_text(node, "name", src)?)),
            _ => None,
        },
    }
}
