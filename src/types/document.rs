use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Document {
    pub id: Uuid,
    pub chunk_index: usize,
    pub path: String, // local/path/to/embedded-file
    pub embedding: Vec<f32>,
}

pub struct DocumentInsert {
    pub chunk_index: usize,
    pub path: String, // local/path/to/embedded-file
    pub embedding: Vec<f32>,
}
