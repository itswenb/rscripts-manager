#[derive(Debug, Clone)]
pub struct ScriptMeta {
    pub title: Option<String>,
    pub description: Option<String>,
    pub inputs: Vec<PortDef>,
    pub outputs: Vec<PortDef>,
}

#[derive(Debug, Clone)]
pub struct PortDef {
    pub name: String,
    pub r#type: String,
    pub default: Option<String>,
    pub description: Option<String>,
}

pub fn parse_script(content: &str) -> ScriptMeta {
    let mut meta = ScriptMeta {
        title: None,
        description: None,
        inputs: vec![],
        outputs: vec![],
    };
    let mut first_doc_line: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("#'") {
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
            continue;
        }
        let rest = trimmed[2..].trim();
        if let Some(val) = rest.strip_prefix("@title") {
            meta.title = Some(val.trim().to_string());
        } else if let Some(val) = rest.strip_prefix("@description") {
            meta.description = Some(val.trim().to_string());
        } else if let Some(val) = rest.strip_prefix("@input") {
            if let Some(port) = parse_port(val) {
                meta.inputs.push(port);
            }
        } else if let Some(val) = rest.strip_prefix("@output") {
            if let Some(port) = parse_port(val) {
                meta.outputs.push(port);
            }
        } else if !rest.is_empty() && !rest.starts_with('@') && first_doc_line.is_none() {
            first_doc_line = Some(rest.to_string());
        }
    }
    if meta.title.is_none() {
        meta.title = first_doc_line;
    }
    meta
}

fn parse_port(val: &str) -> Option<PortDef> {
    let parts: Vec<&str> = val.trim().splitn(3, ' ').collect();
    if parts.is_empty() { return None; }
    let name = parts[0].to_string();
    let type_str = parts.get(1).unwrap_or(&"file");
    let (r#type, default) = if let Some((t, d)) = type_str.split_once(':') {
        (t.to_string(), Some(d.to_string()))
    } else {
        (type_str.to_string(), None)
    };
    let description = parts.get(2).map(|s| s.to_string());
    Some(PortDef { name, r#type, default, description })
}
