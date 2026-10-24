use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorRef {
    pub code: String,
    pub name: String,
}

impl DoctorRef {
    pub fn new(code: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
        }
    }

    pub fn label(&self) -> String {
        if self.name.trim().is_empty() {
            self.code.clone()
        } else {
            format!("{} ({})", self.name, self.code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_falls_back_to_code() {
        assert_eq!(DoctorRef::new("D01", "สมชาย").label(), "สมชาย (D01)");
        assert_eq!(DoctorRef::new("D01", "").label(), "D01");
    }
}
