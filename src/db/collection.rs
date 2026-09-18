use crate::types::document::{Document, DocumentInsert};
use fastembed::similarity::cosine_similarity;
use uuid::Uuid;

#[derive(Debug)]
pub struct SearchResult {
    pub id: Uuid,
    // pub title: String,
    pub score: f32,
    pub content: String,
}

#[derive(Debug)]
pub struct Collection {
    pub name: String,
    pub dimension: usize,
    pub documents: Vec<Document>, // Later upgrade: replace Vec → disk + index.
}

impl Collection {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            dimension: 384, // default
            documents: Vec::<Document>::new(),
        }
    }

    pub fn insert(&mut self, document: DocumentInsert) -> Uuid {
        let new_document_id = Uuid::new_v4();

        let new_document = Document {
            id: new_document_id,
            // title: document.title,
            content: document.content,
            embedding: document.embedding,
        };
        self.documents.push(new_document);

        new_document_id
    }

    pub fn delete(&mut self, document_id: Uuid) -> bool {
        if let Some(pos) = self.documents.iter().position(|d| d.id == document_id) {
            self.documents.swap_remove(pos);
            true
        } else {
            false
        }
    }

    pub fn search(&self, query: &Vec<f32>, k: usize) -> Vec<SearchResult> {
        let mut results: Vec<_> = self
            .documents
            .iter()
            .map(|doc| {
                let score = cosine_similarity(query, &doc.embedding);
                (doc, score)
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        results
            .into_iter()
            .take(k)
            .map(|(doc, score)| SearchResult {
                id: doc.id,
                // title: doc.title.clone(),
                score,
                content: doc.content.clone(),
            })
            .collect()
    }
}
