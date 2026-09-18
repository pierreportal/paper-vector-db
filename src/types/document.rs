use uuid::Uuid;

#[derive(Debug)]
pub struct Document {
    pub id: Uuid,
    // pub title: String,
    pub content: String,
    pub embedding: Vec<f32>,
}

pub struct DocumentInsert {
    // pub title: String,
    pub content: String,
    pub embedding: Vec<f32>,
}
