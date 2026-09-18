use crate::db::storage::Storage;
use crate::types::document::{Document, DocumentInsert};
use fastembed::similarity::cosine_similarity;
use uuid::Uuid;

#[derive(Debug)]
pub struct SearchResult {
    pub id: Uuid,
    pub chunk_index: usize,
    pub score: f32,
    pub path: String,
}

#[derive(Debug)]
pub struct Collection {
    pub name: String,
    pub dimension: usize,
    pub documents: Vec<Document>,
    storage: Storage,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CollectionData {
    name: String,
    dimension: usize,
    documents: Vec<Document>,
}

impl Collection {
    pub fn new(name: &str, storage_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            name: name.to_string(),
            dimension: 384, // default
            documents: Vec::<Document>::new(),
            storage: Storage::new(storage_path),
        }
    }

    pub fn insert(&mut self, document: DocumentInsert) -> Uuid {
        let new_document_id = Uuid::new_v4();

        let new_document = Document {
            id: new_document_id,
            path: document.path,
            chunk_index: document.chunk_index,
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

    pub fn delete_at_path(&mut self, path: String) -> bool {
        if let Some(pos) = self.documents.iter().position(|d| d.path == path) {
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
                chunk_index: doc.chunk_index,
                score,
                path: doc.path.to_owned(),
            })
            .collect()
    }

    // storage
    pub fn save(&self) -> anyhow::Result<()> {
        let data = CollectionData {
            name: self.name.clone(),
            dimension: self.dimension,
            documents: self.documents.clone(),
        };

        self.storage.save(&data)
    }
    pub fn load(storage_path: impl Into<std::path::PathBuf>) -> anyhow::Result<Self> {
        let storage = Storage::new(storage_path);
        let data: CollectionData = storage.load()?;

        Ok(Self {
            name: data.name,
            dimension: data.dimension,
            documents: data.documents,
            storage,
        })
    }
    pub fn open(name: &str, storage_path: impl Into<std::path::PathBuf>) -> anyhow::Result<Self> {
        let storage_path = storage_path.into();
        let storage = Storage::new(storage_path.clone());

        if storage.exists() {
            Self::load(storage_path)
        } else {
            Ok(Self::new(name, storage_path))
        }
    }
}
