use anyhow::Result;

#[derive(Debug, Clone)]
pub struct TxPlan {
    pub instructions: Vec<String>,
}

impl TxPlan {
    pub fn empty() -> Self {
        Self { instructions: vec![] }
    }

    pub fn describe(&self) -> String {
        if self.instructions.is_empty() {
            "no instructions".to_string()
        } else {
            format!("{} instruction(s)", self.instructions.len())
        }
    }
}

pub fn build_publish_plan(_object_id: &str) -> Result<TxPlan> {
    // Placeholder. When signia-program is integrated, this will create real instructions.
    Ok(TxPlan::empty())
}
