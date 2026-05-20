use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMeta {
    pub title: Option<String>,
    pub description: Option<String>,
    pub inputs: Vec<PortDef>,
    pub outputs: Vec<PortDef>,
    pub params: Vec<ParamDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDef {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub default: Option<String>,
    pub description: Option<String>,
}

/// Parse R script header annotations.
/// Supports:
///   #' @title ...
///   #' @description ...
///   #' @input filename.csv "description"
///   #' @output filename.csv "description"
///   #' @param name type default "description"
pub fn parse_script(content: &str) -> ScriptMeta {
    let mut meta = ScriptMeta {
        title: None,
        description: None,
        inputs: Vec::new(),
        outputs: Vec::new(),
        params: Vec::new(),
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("#'") {
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
            continue;
        }
        let body = trimmed[2..].trim();

        if let Some(rest) = body.strip_prefix("@title") {
            meta.title = Some(rest.trim().to_string());
        } else if let Some(rest) = body.strip_prefix("@description") {
            meta.description = Some(rest.trim().to_string());
        } else if let Some(rest) = body.strip_prefix("@input") {
            if let Some(port) = parse_port(rest.trim()) {
                meta.inputs.push(port);
            }
        } else if let Some(rest) = body.strip_prefix("@output") {
            if let Some(port) = parse_port(rest.trim()) {
                meta.outputs.push(port);
            }
        } else if let Some(rest) = body.strip_prefix("@param") {
            if let Some(param) = parse_param(rest.trim()) {
                meta.params.push(param);
            }
        }
    }

    meta
}

fn parse_port(s: &str) -> Option<PortDef> {
    if s.is_empty() {
        return None;
    }
    let (name, desc) = split_name_desc(s);
    Some(PortDef {
        name: name.to_string(),
        description: desc.map(|d| d.to_string()),
    })
}

fn parse_param(s: &str) -> Option<ParamDef> {
    if s.is_empty() {
        return None;
    }
    let parts: Vec<&str> = s.splitn(4, char::is_whitespace).collect();
    let name = parts.first()?.trim().to_string();
    let param_type = parts.get(1).map(|s| s.trim()).unwrap_or("character").to_string();
    let (default, description) = if let Some(rest) = parts.get(2..) {
        let rest_str = rest.join(" ");
        let rest_trimmed = rest_str.trim().to_string();
        if rest_trimmed.starts_with('"') {
            (None, extract_quoted_owned(&rest_trimmed))
        } else {
            let space = rest_trimmed.find('"');
            match space {
                Some(idx) => {
                    let def = rest_trimmed[..idx].trim().to_string();
                    let desc = extract_quoted_owned(&rest_trimmed[idx..]);
                    (Some(def), desc)
                }
                None => {
                    let def = rest_trimmed.split_whitespace().next().unwrap_or("").to_string();
                    (if def.is_empty() { None } else { Some(def) }, None)
                }
            }
        }
    } else {
        (None, None)
    };

    Some(ParamDef { name, param_type, default, description })
}

fn split_name_desc(s: &str) -> (&str, Option<&str>) {
    if let Some(idx) = s.find('"') {
        let name = s[..idx].trim();
        let desc = extract_quoted(&s[idx..]);
        (name, desc)
    } else {
        let name = s.split_whitespace().next().unwrap_or(s);
        (name, None)
    }
}

fn extract_quoted(s: &str) -> Option<&str> {
    let start = s.find('"')? + 1;
    let end = s[start..].find('"').map(|i| start + i).unwrap_or(s.len());
    Some(&s[start..end])
}

fn extract_quoted_owned(s: &str) -> Option<String> {
    extract_quoted(s).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let script = r#"#' @title DEG Analysis
#' @description Differential expression using DESeq2
#'
#' @input counts.csv "Gene expression count matrix"
#' @input metadata.csv "Sample metadata"
#'
#' @param padj_cutoff numeric 0.05 "Adjusted p-value threshold"
#' @param lfc_cutoff numeric 1.0 "Log2 fold change cutoff"
#'
#' @output deg_results.csv "DE genes table"
#' @output volcano.png "Volcano plot"

library(DESeq2)
"#;
        let meta = parse_script(script);
        assert_eq!(meta.title.as_deref(), Some("DEG Analysis"));
        assert_eq!(meta.inputs.len(), 2);
        assert_eq!(meta.inputs[0].name, "counts.csv");
        assert_eq!(meta.outputs.len(), 2);
        assert_eq!(meta.outputs[1].name, "volcano.png");
        assert_eq!(meta.params.len(), 2);
        assert_eq!(meta.params[0].name, "padj_cutoff");
        assert_eq!(meta.params[0].default.as_deref(), Some("0.05"));
    }
}
