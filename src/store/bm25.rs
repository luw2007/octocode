// Copyright 2025 Muvon Un Limited
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::Result;
use std::path::PathBuf;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy};

pub struct BM25Index {
    index: Index,
    reader: IndexReader,
    writer: Option<IndexWriter>,
    content_field: Field,
    path_field: Field,
    symbols_field: Field,
    hash_field: Field,
    block_type_field: Field,
}

impl BM25Index {
    pub fn new(index_path: &PathBuf) -> Result<Self> {
        let mut schema_builder = Schema::builder();

        let content_field = schema_builder.add_text_field("content", TEXT);
        let path_field = schema_builder.add_text_field("path", STRING | STORED);
        let symbols_field = schema_builder.add_text_field("symbols", TEXT);
        let hash_field = schema_builder.add_text_field("hash", STRING | STORED);
        let block_type_field = schema_builder.add_text_field("block_type", STRING | STORED);

        let schema = schema_builder.build();

        let bm25_path = index_path.join("bm25");
        std::fs::create_dir_all(&bm25_path)?;

        let index = Index::open_or_create(tantivy::directory::MmapDirectory::open(&bm25_path)?, schema.clone())?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        Ok(Self {
            index: index.clone(),
            reader,
            writer: None,
            content_field,
            path_field,
            symbols_field,
            hash_field,
            block_type_field,
        })
    }

    pub fn get_writer(&mut self) -> Result<&mut IndexWriter> {
        if self.writer.is_none() {
            self.writer = Some(self.index.writer(50_000_000)?);
        }
        Ok(self.writer.as_mut().unwrap())
    }

    pub fn add_code_block(
        &mut self,
        content: &str,
        path: &str,
        symbols: &[String],
        hash: &str,
    ) -> Result<()> {
        let symbols_text = symbols.join(" ");

        let content_field = self.content_field;
        let path_field = self.path_field;
        let symbols_field = self.symbols_field;
        let hash_field = self.hash_field;
        let block_type_field = self.block_type_field;

        let writer = self.get_writer()?;

        writer.add_document(doc!(
            content_field => content,
            path_field => path,
            symbols_field => symbols_text,
            hash_field => hash,
            block_type_field => "code"
        ))?;

        Ok(())
    }

    pub fn add_document_block(
        &mut self,
        content: &str,
        path: &str,
        hash: &str,
    ) -> Result<()> {
        let content_field = self.content_field;
        let path_field = self.path_field;
        let hash_field = self.hash_field;
        let block_type_field = self.block_type_field;

        let writer = self.get_writer()?;

        writer.add_document(doc!(
            content_field => content,
            path_field => path,
            hash_field => hash,
            block_type_field => "doc"
        ))?;

        Ok(())
    }

    pub fn add_text_block(
        &mut self,
        content: &str,
        path: &str,
        hash: &str,
    ) -> Result<()> {
        let content_field = self.content_field;
        let path_field = self.path_field;
        let hash_field = self.hash_field;
        let block_type_field = self.block_type_field;

        let writer = self.get_writer()?;

        writer.add_document(doc!(
            content_field => content,
            path_field => path,
            hash_field => hash,
            block_type_field => "text"
        ))?;

        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        if let Some(writer) = &mut self.writer {
            writer.commit()?;
        }
        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize, block_type: Option<&str>) -> Result<Vec<BM25Result>> {
        let searcher = self.reader.searcher();

        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.content_field, self.symbols_field, self.path_field],
        );

        let query = query_parser.parse_query(query_str)?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;

            if let Some(filter_type) = block_type {
                if let Some(doc_type) = retrieved_doc.get_first(self.block_type_field) {
                    if doc_type.as_str() != Some(filter_type) {
                        continue;
                    }
                }
            }

            let hash = retrieved_doc
                .get_first(self.hash_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let path = retrieved_doc
                .get_first(self.path_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            results.push(BM25Result {
                hash,
                path,
                score: score as f32,
            });
        }

        Ok(results)
    }

    pub fn clear(&mut self) -> Result<()> {
        if let Some(writer) = &mut self.writer {
            writer.delete_all_documents()?;
            writer.commit()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BM25Result {
    pub hash: String,
    pub path: String,
    pub score: f32,
}
