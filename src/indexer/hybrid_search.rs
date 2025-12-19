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

use crate::store::{bm25::BM25Result, CodeBlock, DocumentBlock, TextBlock};
use std::collections::HashMap;

pub struct QueryStrategy {
    pub bm25_weight: f32,
    pub vector_weight: f32,
}

impl QueryStrategy {
    pub fn from_query(query: &str) -> Self {
        let word_count = query.split_whitespace().count();
        let is_identifier = query.chars().all(|c| c.is_alphanumeric() || c == '_');
        let has_camel_case = query.chars().any(|c| c.is_uppercase())
            && query.chars().any(|c| c.is_lowercase());

        if word_count <= 2 || is_identifier || has_camel_case {
            Self {
                bm25_weight: 0.8,
                vector_weight: 0.2,
            }
        } else if word_count > 5 {
            Self {
                bm25_weight: 0.3,
                vector_weight: 0.7,
            }
        } else {
            Self {
                bm25_weight: 0.5,
                vector_weight: 0.5,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct HybridResult {
    pub hash: String,
    pub rrf_score: f32,
    pub bm25_rank: Option<usize>,
    pub vector_rank: Option<usize>,
}

pub fn reciprocal_rank_fusion(
    bm25_results: &[BM25Result],
    vector_results: &[CodeBlock],
    strategy: &QueryStrategy,
    k: f32,
) -> Vec<HybridResult> {
    let mut score_map: HashMap<String, HybridResult> = HashMap::new();

    for (rank, result) in bm25_results.iter().enumerate() {
        let rrf_score = strategy.bm25_weight / (k + rank as f32 + 1.0);
        score_map
            .entry(result.hash.clone())
            .and_modify(|e| {
                e.rrf_score += rrf_score;
                e.bm25_rank = Some(rank);
            })
            .or_insert(HybridResult {
                hash: result.hash.clone(),
                rrf_score,
                bm25_rank: Some(rank),
                vector_rank: None,
            });
    }

    for (rank, result) in vector_results.iter().enumerate() {
        let rrf_score = strategy.vector_weight / (k + rank as f32 + 1.0);
        score_map
            .entry(result.hash.clone())
            .and_modify(|e| {
                e.rrf_score += rrf_score;
                e.vector_rank = Some(rank);
            })
            .or_insert(HybridResult {
                hash: result.hash.clone(),
                rrf_score,
                bm25_rank: None,
                vector_rank: Some(rank),
            });
    }

    let mut results: Vec<_> = score_map.into_values().collect();
    results.sort_by(|a, b| b.rrf_score.partial_cmp(&a.rrf_score).unwrap());

    results
}

pub fn reciprocal_rank_fusion_docs(
    bm25_results: &[BM25Result],
    vector_results: &[DocumentBlock],
    strategy: &QueryStrategy,
    k: f32,
) -> Vec<HybridResult> {
    let mut score_map: HashMap<String, HybridResult> = HashMap::new();

    for (rank, result) in bm25_results.iter().enumerate() {
        let rrf_score = strategy.bm25_weight / (k + rank as f32 + 1.0);
        score_map
            .entry(result.hash.clone())
            .and_modify(|e| {
                e.rrf_score += rrf_score;
                e.bm25_rank = Some(rank);
            })
            .or_insert(HybridResult {
                hash: result.hash.clone(),
                rrf_score,
                bm25_rank: Some(rank),
                vector_rank: None,
            });
    }

    for (rank, result) in vector_results.iter().enumerate() {
        let rrf_score = strategy.vector_weight / (k + rank as f32 + 1.0);
        score_map
            .entry(result.hash.clone())
            .and_modify(|e| {
                e.rrf_score += rrf_score;
                e.vector_rank = Some(rank);
            })
            .or_insert(HybridResult {
                hash: result.hash.clone(),
                rrf_score,
                bm25_rank: None,
                vector_rank: Some(rank),
            });
    }

    let mut results: Vec<_> = score_map.into_values().collect();
    results.sort_by(|a, b| b.rrf_score.partial_cmp(&a.rrf_score).unwrap());

    results
}

pub fn reciprocal_rank_fusion_text(
    bm25_results: &[BM25Result],
    vector_results: &[TextBlock],
    strategy: &QueryStrategy,
    k: f32,
) -> Vec<HybridResult> {
    let mut score_map: HashMap<String, HybridResult> = HashMap::new();

    for (rank, result) in bm25_results.iter().enumerate() {
        let rrf_score = strategy.bm25_weight / (k + rank as f32 + 1.0);
        score_map
            .entry(result.hash.clone())
            .and_modify(|e| {
                e.rrf_score += rrf_score;
                e.bm25_rank = Some(rank);
            })
            .or_insert(HybridResult {
                hash: result.hash.clone(),
                rrf_score,
                bm25_rank: Some(rank),
                vector_rank: None,
            });
    }

    for (rank, result) in vector_results.iter().enumerate() {
        let rrf_score = strategy.vector_weight / (k + rank as f32 + 1.0);
        score_map
            .entry(result.hash.clone())
            .and_modify(|e| {
                e.rrf_score += rrf_score;
                e.vector_rank = Some(rank);
            })
            .or_insert(HybridResult {
                hash: result.hash.clone(),
                rrf_score,
                bm25_rank: None,
                vector_rank: Some(rank),
            });
    }

    let mut results: Vec<_> = score_map.into_values().collect();
    results.sort_by(|a, b| b.rrf_score.partial_cmp(&a.rrf_score).unwrap());

    results
}
