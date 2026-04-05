//! Agent API server — processes JSON-RPC requests.

use crate::protocol::*;
use serde_json::json;
use std::path::{Path, PathBuf};

/// The Agent API server. Stateless — each request includes source code.
pub struct AgentServer;

struct LoadedSource {
    source: String,
    file: Option<String>,
}

struct LoadedProject {
    root: PathBuf,
    display: String,
    project: aura_core::project::Project,
    entry_file: Option<String>,
    entry_source: Option<String>,
}

impl AgentServer {
    pub fn new() -> Self {
        Self
    }

    /// Process a single JSON-RPC request and return a response.
    pub fn handle_request(&self, request: &Request) -> Response {
        match request.method.as_str() {
            "ast.get" => self.handle_ast_get(&request.id, &request.params),
            "diagnostics.get" => self.handle_diagnostics(&request.id, &request.params),
            "completions.get" => self.handle_completions(&request.id, &request.params),
            "hir.get" => self.handle_hir_get(&request.id, &request.params),
            "explain" => self.handle_explain(&request.id, &request.params),
            "sketch" => self.handle_sketch(&request.id, &request.params),
            "hover" => self.handle_hover(&request.id, &request.params),
            "goto.definition" => self.handle_goto_definition(&request.id, &request.params),
            "ping" => Response::success(
                request.id.clone(),
                json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }),
            ),
            _ => Response::error(
                request.id.clone(),
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    /// Process a JSON string request and return a JSON string response.
    pub fn handle_json(&self, input: &str) -> String {
        let request: Request = match serde_json::from_str(input) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::error(json!(null), -32700, format!("Parse error: {}", e));
                return serde_json::to_string(&resp).unwrap();
            }
        };

        let response = self.handle_request(&request);
        serde_json::to_string(&response).unwrap()
    }

    // === Method handlers ===

    fn handle_ast_get(&self, id: &serde_json::Value, params: &serde_json::Value) -> Response {
        if params
            .get("path")
            .and_then(|value| value.as_str())
            .is_some()
        {
            let loaded = match load_project(params) {
                Ok(loaded) => loaded,
                Err(message) => return Response::error(id.clone(), -32602, message),
            };
            // Serialize AST summary
            let ast_json = json!({
                "app": {
                    "name": loaded.project.program.app.name,
                    "members": loaded.project.program.app.members.len(),
                },
                "imports": loaded.project.program.imports.len(),
                "parse_errors": loaded.project.errors.len(),
                "project_root": loaded.root.display().to_string(),
                "files": loaded.project.files.iter().map(|file| file.path.clone()).collect::<Vec<_>>(),
            });
            Response::success(id.clone(), ast_json)
        } else {
            let loaded = match load_source(params) {
                Ok(loaded) => loaded,
                Err(message) => return Response::error(id.clone(), -32602, message),
            };
            let result = aura_core::parser::parse(&loaded.source);

            if let Some(ref program) = result.program {
                let ast_json = json!({
                    "app": {
                        "name": program.app.name,
                        "members": program.app.members.len(),
                    },
                    "imports": program.imports.len(),
                    "parse_errors": result.errors.len(),
                });
                Response::success(id.clone(), ast_json)
            } else {
                Response::error(
                    id.clone(),
                    -32000,
                    format!("Parse failed with {} errors", result.errors.len()),
                )
            }
        }
    }

    fn handle_diagnostics(&self, id: &serde_json::Value, params: &serde_json::Value) -> Response {
        let diagnostics = if params
            .get("path")
            .and_then(|value| value.as_str())
            .is_some()
        {
            let loaded = match load_project(params) {
                Ok(loaded) => loaded,
                Err(message) => return Response::error(id.clone(), -32602, message),
            };
            let analysis =
                aura_core::semantic::SemanticAnalyzer::new().analyze(&loaded.project.program);
            let mut diagnostics = Vec::new();

            for file in &loaded.project.files {
                let source = match std::fs::read_to_string(&file.abs_path) {
                    Ok(source) => source,
                    Err(_) => continue,
                };
                let parse_result = aura_core::parser::parse(&source);
                diagnostics.extend(
                    parse_result
                        .errors
                        .iter()
                        .map(|err| diagnostic_from_error(err, &source, Some(file.path.as_str()))),
                );
            }

            diagnostics.extend(analysis.errors.iter().map(|err| {
                diagnostic_from_error(
                    err,
                    loaded.entry_source.as_deref().unwrap_or(""),
                    loaded.entry_file.as_deref(),
                )
            }));

            diagnostics.extend(
                loaded
                    .project
                    .errors
                    .iter()
                    .filter(|err| err.span.start == 0 && err.span.end == 0)
                    .map(|err| diagnostic_from_error(err, "", Some(loaded.display.as_str()))),
            );
            diagnostics
        } else {
            let loaded = match load_source(params) {
                Ok(loaded) => loaded,
                Err(message) => return Response::error(id.clone(), -32602, message),
            };

            let parse_result = aura_core::parser::parse(&loaded.source);
            let mut all_errors = parse_result.errors;

            if let Some(ref program) = parse_result.program {
                let analysis = aura_core::semantic::SemanticAnalyzer::new().analyze(program);
                all_errors.extend(analysis.errors);
            }

            all_errors
                .iter()
                .map(|err| diagnostic_from_error(err, &loaded.source, loaded.file.as_deref()))
                .collect()
        };

        let error_count = diagnostics.iter().filter(|d| d.severity == "error").count();
        let warning_count = diagnostics
            .iter()
            .filter(|d| d.severity == "warning")
            .count();

        Response::success(
            id.clone(),
            json!({
                "diagnostics": diagnostics,
                "summary": {
                    "errors": error_count,
                    "warnings": warning_count,
                    "total": diagnostics.len(),
                }
            }),
        )
    }

    fn handle_completions(&self, id: &serde_json::Value, params: &serde_json::Value) -> Response {
        let _source = params.get("source").and_then(|s| s.as_str()).unwrap_or("");
        let context = params
            .get("context")
            .and_then(|s| s.as_str())
            .unwrap_or("view");

        let mut completions: Vec<AgentCompletion> = Vec::new();

        match context {
            "design_token" | "view" => {
                // Spacing tokens
                for t in &["xs", "sm", "md", "lg", "xl", "2xl", "3xl", "4xl"] {
                    completions.push(AgentCompletion {
                        label: format!(".{}", t),
                        kind: "spacing".to_string(),
                        detail: Some(format!("Spacing: {}", t)),
                    });
                }
                // Color tokens
                for t in &[
                    "accent",
                    "primary",
                    "secondary",
                    "muted",
                    "danger",
                    "warning",
                    "success",
                    "info",
                    "surface",
                ] {
                    completions.push(AgentCompletion {
                        label: format!(".{}", t),
                        kind: "color".to_string(),
                        detail: Some(format!("Color: {}", t)),
                    });
                }
                // Typography tokens
                for t in &[
                    "bold",
                    "medium",
                    "semibold",
                    "italic",
                    "mono",
                    "center",
                    "uppercase",
                ] {
                    completions.push(AgentCompletion {
                        label: format!(".{}", t),
                        kind: "typography".to_string(),
                        detail: Some(format!("Typography: {}", t)),
                    });
                }
                // Shape tokens
                for t in &["rounded", "smooth", "pill", "circle", "sharp"] {
                    completions.push(AgentCompletion {
                        label: format!(".{}", t),
                        kind: "shape".to_string(),
                        detail: Some(format!("Shape: {}", t)),
                    });
                }
                // Compound tokens
                for prefix in &["gap", "padding", "margin", "size"] {
                    for size in &["xs", "sm", "md", "lg", "xl", "2xl"] {
                        completions.push(AgentCompletion {
                            label: format!("{}.{}", prefix, size),
                            kind: "compound".to_string(),
                            detail: Some(format!("{} {}", prefix, size)),
                        });
                    }
                }
            }
            "type" => {
                for t in &[
                    "text",
                    "int",
                    "float",
                    "bool",
                    "timestamp",
                    "duration",
                    "percent",
                    "secret",
                    "sanitized",
                    "email",
                    "url",
                    "token",
                    "list",
                    "map",
                    "set",
                    "optional",
                    "enum",
                ] {
                    completions.push(AgentCompletion {
                        label: t.to_string(),
                        kind: "type".to_string(),
                        detail: None,
                    });
                }
            }
            "view_element" => {
                for (elem, desc) in &[
                    ("column", "Vertical layout"),
                    ("row", "Horizontal layout"),
                    ("stack", "Layered stack"),
                    ("grid", "Grid layout"),
                    ("scroll", "Scrollable container"),
                    ("text", "Text display"),
                    ("heading", "Heading text"),
                    ("button", "Interactive button"),
                    ("textfield", "Text input"),
                    ("checkbox", "Checkbox"),
                    ("toggle", "Toggle switch"),
                    ("slider", "Value slider"),
                    ("image", "Image display"),
                    ("icon", "Icon display"),
                    ("spacer", "Flexible space"),
                    ("divider", "Separator line"),
                    ("if", "Conditional view"),
                    ("each", "Loop over list"),
                    ("when", "Pattern match"),
                ] {
                    completions.push(AgentCompletion {
                        label: elem.to_string(),
                        kind: "view_element".to_string(),
                        detail: Some(desc.to_string()),
                    });
                }
            }
            _ => {}
        }

        Response::success(id.clone(), json!({ "completions": completions }))
    }

    fn handle_hir_get(&self, id: &serde_json::Value, params: &serde_json::Value) -> Response {
        if params
            .get("path")
            .and_then(|value| value.as_str())
            .is_some()
        {
            let loaded = match load_project(params) {
                Ok(loaded) => loaded,
                Err(message) => return Response::error(id.clone(), -32602, message),
            };
            let hir = aura_core::hir::build_hir(&loaded.project.program);
            let hir_json = json!({
                "app": {
                    "name": hir.app.name,
                    "theme": hir.app.theme,
                    "navigation": format!("{:?}", hir.app.navigation),
                },
                "models": hir.models.iter().map(|m| json!({
                    "name": m.name,
                    "fields": m.fields.iter().map(|f| json!({
                        "name": f.name,
                        "type": f.field_type.display_name(),
                    })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
                "screens": hir.screens.iter().map(|s| json!({
                    "name": s.name,
                    "state_count": s.state.len(),
                    "action_count": s.actions.len(),
                    "function_count": s.functions.len(),
                })).collect::<Vec<_>>(),
                "components": hir.components.iter().map(|c| json!({
                    "name": c.name,
                    "props": c.props.iter().map(|p| json!({
                        "name": p.name,
                    "type": p.param_type.display_name(),
                })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
                "project_root": loaded.root.display().to_string(),
                "files": loaded.project.files.iter().map(|file| file.path.clone()).collect::<Vec<_>>(),
            });
            Response::success(id.clone(), hir_json)
        } else {
            let loaded = match load_source(params) {
                Ok(loaded) => loaded,
                Err(message) => return Response::error(id.clone(), -32602, message),
            };
            let result = aura_core::parser::parse(&loaded.source);
            if let Some(ref program) = result.program {
                let hir = aura_core::hir::build_hir(program);
                let hir_json = json!({
                    "app": {
                        "name": hir.app.name,
                        "theme": hir.app.theme,
                        "navigation": format!("{:?}", hir.app.navigation),
                    },
                    "models": hir.models.iter().map(|m| json!({
                        "name": m.name,
                        "fields": m.fields.iter().map(|f| json!({
                            "name": f.name,
                            "type": f.field_type.display_name(),
                        })).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                    "screens": hir.screens.iter().map(|s| json!({
                        "name": s.name,
                        "state_count": s.state.len(),
                        "action_count": s.actions.len(),
                        "function_count": s.functions.len(),
                    })).collect::<Vec<_>>(),
                    "components": hir.components.iter().map(|c| json!({
                        "name": c.name,
                        "props": c.props.iter().map(|p| json!({
                            "name": p.name,
                            "type": p.param_type.display_name(),
                        })).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                });
                Response::success(id.clone(), hir_json)
            } else {
                Response::error(id.clone(), -32000, "Parse failed".to_string())
            }
        }
    }

    fn handle_explain(&self, id: &serde_json::Value, params: &serde_json::Value) -> Response {
        if params
            .get("path")
            .and_then(|value| value.as_str())
            .is_some()
        {
            let loaded = match load_project(params) {
                Ok(loaded) => loaded,
                Err(message) => return Response::error(id.clone(), -32602, message),
            };
            let hir = aura_core::hir::build_hir(&loaded.project.program);
            let explanation = aura_core::explain::explain(&hir);
            Response::success(
                id.clone(),
                json!({
                    "explanation": explanation,
                    "project_root": loaded.root.display().to_string(),
                }),
            )
        } else {
            let loaded = match load_source(params) {
                Ok(loaded) => loaded,
                Err(message) => return Response::error(id.clone(), -32602, message),
            };
            let result = aura_core::parser::parse(&loaded.source);
            if let Some(ref program) = result.program {
                let hir = aura_core::hir::build_hir(program);
                let explanation = aura_core::explain::explain(&hir);
                Response::success(id.clone(), json!({ "explanation": explanation }))
            } else {
                Response::error(id.clone(), -32000, "Parse failed".to_string())
            }
        }
    }

    fn handle_sketch(&self, id: &serde_json::Value, params: &serde_json::Value) -> Response {
        let description = match params.get("description").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => {
                return Response::error(
                    id.clone(),
                    -32602,
                    "Missing 'description' parameter".to_string(),
                );
            }
        };

        let code = aura_core::sketch::sketch(description);
        Response::success(id.clone(), json!({ "code": code }))
    }

    fn handle_hover(&self, id: &serde_json::Value, params: &serde_json::Value) -> Response {
        let loaded = match load_source(params) {
            Ok(loaded) => loaded,
            Err(message) => return Response::error(id.clone(), -32602, message),
        };
        let line = params.get("line").and_then(|l| l.as_u64()).unwrap_or(1) as usize;
        let column = params.get("column").and_then(|c| c.as_u64()).unwrap_or(1) as usize;

        // Find the byte offset for the given line:column
        let byte_offset = line_col_to_byte(&loaded.source, line, column);

        // Parse and analyze
        let result = aura_core::parser::parse(&loaded.source);
        if let Some(ref program) = result.program {
            let analysis = aura_core::semantic::SemanticAnalyzer::new().analyze(program);
            let file_label = loaded.file.as_deref();

            // Find the token/identifier at this position
            let lex_result = aura_core::lexer::lex(&loaded.source);
            let mut hover_info = None;

            for token in &lex_result.tokens {
                if token.span.start <= byte_offset && byte_offset < token.span.end {
                    match &token.value {
                        aura_core::lexer::Token::Ident(name)
                        | aura_core::lexer::Token::TypeIdent(name) => {
                            // Look up in symbol table
                            if let Some(sym) = analysis.symbols.lookup(0, name) {
                                hover_info = Some(json!({
                                    "name": name,
                                    "kind": format!("{:?}", sym.kind),
                                    "type": sym.resolved_type.display_name(),
                                    "file": file_label,
                                    "line": line,
                                    "column": column,
                                }));
                            } else {
                                hover_info = Some(json!({
                                    "name": name,
                                    "kind": "unknown",
                                    "type": null,
                                    "file": file_label,
                                    "line": line,
                                    "column": column,
                                }));
                            }
                        }
                        // Design tokens
                        aura_core::lexer::Token::Dot => {
                            hover_info = Some(json!({
                                "name": ".",
                                "kind": "design_token",
                                "type": "design token prefix",
                            }));
                        }
                        other => {
                            hover_info = Some(json!({
                                "name": format!("{}", other),
                                "kind": "keyword",
                                "type": null,
                            }));
                        }
                    }
                    break;
                }
            }

            match hover_info {
                Some(info) => Response::success(id.clone(), json!({ "hover": info })),
                None => Response::success(id.clone(), json!({ "hover": null })),
            }
        } else {
            Response::error(id.clone(), -32000, "Parse failed".to_string())
        }
    }

    fn handle_goto_definition(
        &self,
        id: &serde_json::Value,
        params: &serde_json::Value,
    ) -> Response {
        let loaded = match load_source(params) {
            Ok(loaded) => loaded,
            Err(message) => return Response::error(id.clone(), -32602, message),
        };
        let line = params.get("line").and_then(|l| l.as_u64()).unwrap_or(1) as usize;
        let column = params.get("column").and_then(|c| c.as_u64()).unwrap_or(1) as usize;

        let byte_offset = line_col_to_byte(&loaded.source, line, column);

        let result = aura_core::parser::parse(&loaded.source);
        if let Some(ref program) = result.program {
            let analysis = aura_core::semantic::SemanticAnalyzer::new().analyze(program);
            let file_label = loaded.file.as_deref();

            let lex_result = aura_core::lexer::lex(&loaded.source);
            for token in &lex_result.tokens {
                if token.span.start <= byte_offset && byte_offset < token.span.end {
                    if let aura_core::lexer::Token::Ident(name)
                    | aura_core::lexer::Token::TypeIdent(name) = &token.value
                    {
                        if let Some(sym) = analysis.symbols.lookup(0, name) {
                            let (def_line, def_col) =
                                byte_to_line_col(&loaded.source, sym.span.start);
                            return Response::success(
                                id.clone(),
                                json!({
                                    "definition": {
                                        "name": name,
                                        "file": file_label,
                                        "line": def_line,
                                        "column": def_col,
                                        "kind": format!("{:?}", sym.kind),
                                    }
                                }),
                            );
                        }
                    }
                    break;
                }
            }

            Response::success(id.clone(), json!({ "definition": null }))
        } else {
            Response::error(id.clone(), -32000, "Parse failed".to_string())
        }
    }
}

fn diagnostic_from_error(
    err: &aura_core::AuraError,
    source: &str,
    file: Option<&str>,
) -> AgentDiagnostic {
    let (line, column) = if source.is_empty() {
        (1, 1)
    } else {
        byte_to_line_col(source, err.span.start)
    };

    AgentDiagnostic {
        code: format!("{}", err.code),
        severity: match err.severity {
            aura_core::Severity::Error => "error".to_string(),
            aura_core::Severity::Warning => "warning".to_string(),
            aura_core::Severity::Info => "info".to_string(),
        },
        message: err.message.clone(),
        location: AgentLocation {
            file: file.map(ToOwned::to_owned),
            start: err.span.start,
            end: err.span.end,
            line,
            column,
        },
        fix: err.fix.as_ref().map(|fix| AgentFix {
            action: format!("{:?}", fix.action),
            replacement: fix.replacement.clone(),
            confidence: fix.confidence,
        }),
        suppressed: err.suppressed,
    }
}

fn load_source(params: &serde_json::Value) -> Result<LoadedSource, String> {
    if let Some(source) = params.get("source").and_then(|value| value.as_str()) {
        return Ok(LoadedSource {
            source: source.to_string(),
            file: params
                .get("file")
                .and_then(|value| value.as_str())
                .map(str::to_string),
        });
    }

    let path = params
        .get("path")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "Missing 'source' or 'path' parameter".to_string())?;
    let path_buf = PathBuf::from(path);
    if path_buf.is_dir() {
        let project = load_project(params)?;
        return Ok(LoadedSource {
            source: project.entry_source.ok_or_else(|| {
                format!("Could not find an Aura entry file in {}", project.display)
            })?,
            file: project.entry_file,
        });
    }

    let source = std::fs::read_to_string(&path_buf)
        .map_err(|err| format!("Cannot read '{}': {}", path_buf.display(), err))?;
    Ok(LoadedSource {
        source,
        file: Some(path_buf.display().to_string()),
    })
}

fn load_project(params: &serde_json::Value) -> Result<LoadedProject, String> {
    let path = params
        .get("path")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "Missing 'path' parameter".to_string())?;
    let requested = PathBuf::from(path);
    if !requested.exists() {
        return Err(format!("Path '{}' does not exist", requested.display()));
    }

    let root = if requested.is_file() {
        find_project_root(&requested)
            .unwrap_or_else(|| requested.parent().unwrap_or(Path::new(".")).to_path_buf())
    } else {
        find_project_root(&requested).unwrap_or_else(|| requested.clone())
    };
    let project = aura_core::project::load_project(&root);

    let entry_path = if requested.is_file() {
        Some(requested.clone())
    } else {
        project.files.first().map(|file| file.abs_path.clone())
    };
    let entry_file = entry_path.as_ref().map(|path| path.display().to_string());
    let entry_source = entry_path
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok());

    Ok(LoadedProject {
        display: root.display().to_string(),
        root,
        project,
        entry_file,
        entry_source,
    })
}

fn find_project_root(start: &Path) -> Option<PathBuf> {
    let start_dir = if start.is_file() {
        start.parent().unwrap_or(start)
    } else {
        start
    };

    for dir in start_dir.ancestors() {
        if dir.join("aura.toml").exists() {
            return Some(dir.to_path_buf());
        }

        if dir.join("src").join("main.aura").exists() {
            return Some(dir.to_path_buf());
        }
    }

    None
}

fn line_col_to_byte(source: &str, target_line: usize, target_col: usize) -> usize {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if line == target_line && col == target_col {
            return i;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    source.len()
}

fn byte_to_line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= byte_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn call(method: &str, params: serde_json::Value) -> serde_json::Value {
        let server = AgentServer::new();
        let request = Request {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: method.to_string(),
            params,
        };
        let response = server.handle_request(&request);
        assert!(response.error.is_none(), "RPC error: {:?}", response.error);
        response.result.unwrap()
    }

    #[test]
    fn test_ping() {
        let result = call("ping", json!({}));
        assert_eq!(result["status"], "ok");
    }

    #[test]
    fn test_ast_get() {
        let result = call(
            "ast.get",
            json!({
                "source": "app Hello\n  screen Main\n    view\n      text \"Hi\""
            }),
        );
        assert_eq!(result["app"]["name"], "Hello");
        assert_eq!(result["parse_errors"], 0);
    }

    #[test]
    fn test_diagnostics_clean() {
        let result = call(
            "diagnostics.get",
            json!({
                "source": "app Test\n  screen Main\n    view\n      text \"Hi\""
            }),
        );
        assert_eq!(result["summary"]["errors"], 0);
    }

    #[test]
    fn test_diagnostics_with_errors() {
        let result = call(
            "diagnostics.get",
            json!({
                "source": "app Test\n  screen Main\n    state x: int = 0\n    view\n      text \"Hi\"\n    action test\n      unknownVar = 1"
            }),
        );
        let diagnostics = result["diagnostics"].as_array().unwrap();
        assert!(!diagnostics.is_empty());
        // Should have fix suggestion with confidence
        let err = &diagnostics[0];
        assert_eq!(err["severity"], "error");
    }

    #[test]
    fn test_diagnostics_with_fix_confidence() {
        let result = call(
            "diagnostics.get",
            json!({
                "source": "app Test\n  screen Main\n    state todos: list[text] = []\n    view\n      text \"Hi\"\n    action test\n      todoos = []"
            }),
        );
        let diagnostics = result["diagnostics"].as_array().unwrap();
        let type_errs: Vec<_> = diagnostics
            .iter()
            .filter(|d| d["code"] == "E0103")
            .collect();
        assert!(!type_errs.is_empty(), "Expected E0103");
        let fix = &type_errs[0]["fix"];
        assert!(fix.is_object(), "Expected fix suggestion");
        assert_eq!(fix["replacement"], "todos");
        assert!(fix["confidence"].as_f64().unwrap() > 0.7);
    }

    #[test]
    fn test_completions_design_tokens() {
        let result = call(
            "completions.get",
            json!({
                "context": "design_token"
            }),
        );
        let completions = result["completions"].as_array().unwrap();
        assert!(completions.len() > 20);
        let labels: Vec<&str> = completions
            .iter()
            .map(|c| c["label"].as_str().unwrap())
            .collect();
        assert!(labels.contains(&".accent"));
        assert!(labels.contains(&".bold"));
        assert!(labels.contains(&"gap.md"));
    }

    #[test]
    fn test_completions_view_elements() {
        let result = call(
            "completions.get",
            json!({
                "context": "view_element"
            }),
        );
        let completions = result["completions"].as_array().unwrap();
        let labels: Vec<&str> = completions
            .iter()
            .map(|c| c["label"].as_str().unwrap())
            .collect();
        assert!(labels.contains(&"column"));
        assert!(labels.contains(&"button"));
        assert!(labels.contains(&"textfield"));
    }

    #[test]
    fn test_hir_get() {
        let result = call(
            "hir.get",
            json!({
                "source": "app Test\n  model Todo\n    title: text\n  screen Main\n    view\n      text \"Hi\""
            }),
        );
        assert_eq!(result["app"]["name"], "Test");
        assert_eq!(result["models"][0]["name"], "Todo");
        assert_eq!(result["screens"][0]["name"], "Main");
    }

    #[test]
    fn test_explain() {
        let result = call(
            "explain",
            json!({
                "source": "app Hello\n  screen Main\n    view\n      text \"Hello, Aura!\""
            }),
        );
        let explanation = result["explanation"].as_str().unwrap();
        assert!(explanation.contains("Hello"));
    }

    #[test]
    fn test_sketch() {
        let result = call(
            "sketch",
            json!({
                "description": "counter app"
            }),
        );
        let code = result["code"].as_str().unwrap();
        assert!(code.contains("state count"));
    }

    #[test]
    fn test_json_roundtrip() {
        let server = AgentServer::new();
        let input = r#"{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}"#;
        let output = server.handle_json(input);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["result"]["status"], "ok");
    }

    #[test]
    fn test_unknown_method() {
        let server = AgentServer::new();
        let request = Request {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "nonexistent".to_string(),
            params: json!({}),
        };
        let response = server.handle_request(&request);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }

    fn temp_project() -> (std::path::PathBuf, std::path::PathBuf) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("aura-agent-test-{}", unique));
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            root.join("aura.toml"),
            "[app]\nname = \"AgentTest\"\nversion = \"0.1.0\"\naura-version = \"0.1.0\"\n",
        )
        .unwrap();
        (root, src)
    }

    #[test]
    fn test_hir_get_supports_project_path() {
        let (root, src) = temp_project();
        std::fs::write(
            src.join("main.aura"),
            "app AgentTest\n  model Todo\n    title: text\n  screen Main\n    view\n      text \"Hi\"",
        )
        .unwrap();

        let result = call("hir.get", json!({ "path": root.display().to_string() }));
        assert_eq!(result["app"]["name"], "AgentTest");
        assert_eq!(result["project_root"], root.display().to_string());
        assert_eq!(result["files"].as_array().unwrap().len(), 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn test_diagnostics_supports_file_path() {
        let (root, src) = temp_project();
        let file = src.join("main.aura");
        std::fs::write(
            &file,
            "app AgentTest\n  screen Main\n    state todos: list[text] = []\n    view\n      text \"Hi\"\n    action test\n      todoos = []",
        )
        .unwrap();

        let result = call(
            "diagnostics.get",
            json!({ "path": file.display().to_string() }),
        );
        assert!(result["summary"]["errors"].as_u64().unwrap() >= 1);
        let diagnostics = result["diagnostics"].as_array().unwrap();
        assert!(
            diagnostics
                .iter()
                .any(|diag| diag["location"]["file"] == file.display().to_string()),
            "Expected file path in diagnostics"
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
