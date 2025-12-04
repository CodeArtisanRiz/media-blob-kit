use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSettings {
    pub variants: Option<HashMap<String, VariantConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantConfig {
    pub format: Option<String>,
    pub quality: Option<u8>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub fit: Option<String>, // cover, contain, inside, fill
}
