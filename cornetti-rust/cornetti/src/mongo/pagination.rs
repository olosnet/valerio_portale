use crate::core::models::CornettiResult;
use crate::core::pagination::{
    FilterNode, FilterOperator, FilterValue, GroupOperator, JoinEntry, LoadOptions,
    PaginationResult, SortDescriptor, SortDirection,
};
use bson::doc;
use futures_util::{StreamExt, TryStreamExt};
use mongodb::bson::{self, Document};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// MongoDB query builder for pagination.
///
/// Translates `FilterNode` into BSON filters, `SortDescriptor` into sort
/// documents, and `JoinEntry` into `$lookup` stages of the aggregation pipeline.
pub struct MongoPagination;

impl MongoPagination {
    /// Executes a full paginated query on MongoDB.
    ///
    /// When no fields require JOINs, uses `find()` + `count_documents()`.
    /// When JOINs are present, uses aggregation pipeline with `$lookup` + `$facet`.
    ///
    /// # Type parameters
    ///
    /// * `T` — return type (DTO).
    /// * `M` — MongoDB model type (must be `DeserializeOwned`).
    ///
    /// `map_fn` converts each `M` into `T`.
    pub async fn paginate<T, M>(
        collection: &mongodb::Collection<M>,
        load_options: &LoadOptions,
        join_dict: &HashMap<String, JoinEntry>,
        map_fn: impl Fn(M) -> T + Send,
    ) -> CornettiResult<PaginationResult<T>>
    where
        T: Send,
        M: DeserializeOwned + Send + Sync + Unpin,
    {
        let combined = load_options.combined_filter();
        let (main_filter, join_entries) =
            Self::split_filters(combined.as_ref(), join_dict);

        // Verifica se ci sono joins sia da filtri che da sort
        let has_filter_joins = !join_entries.is_empty();
        let has_sort_joins = load_options
            .sort
            .iter()
            .any(|s| join_dict.contains_key(&s.field));
        let has_joins = has_filter_joins || has_sort_joins;

        if has_joins {
            Self::paginate_with_joins(
                collection,
                load_options,
                &main_filter,
                &join_entries,
                join_dict,
                map_fn,
            )
            .await
        } else {
            Self::paginate_simple(collection, load_options, &main_filter, map_fn).await
        }
    }

    /// Builds a BSON filter from a `FilterNode`.
    ///
    /// Returns the `Document` for `$match`.
    pub fn build_filter(
        filter: &FilterNode,
        join_dict: &HashMap<String, JoinEntry>,
    ) -> CornettiResult<Document> {
        Ok(filter_to_bson(filter, join_dict))
    }

    /// Builds a BSON sort document.
    ///
    /// Returns the `Document` for `$sort`.
    pub fn build_sort(
        sort: &[SortDescriptor],
        join_dict: &HashMap<String, JoinEntry>,
    ) -> CornettiResult<Document> {
        Ok(Self::build_sort_inner(sort, join_dict))
    }

    // ─── Interni ────────────────────────────────────────────────────

    /// Separa il filtro in: (filtri su collezione principale, filtri su joined).
    fn split_filters<'a>(
        filter: Option<&FilterNode>,
        join_dict: &'a HashMap<String, JoinEntry>,
    ) -> (Document, Vec<(&'a JoinEntry, Document)>) {
        let mut main_doc = Document::new();
        let mut join_filters: Vec<(&JoinEntry, Document)> = Vec::new();

        if let Some(f) = filter {
            let (main_filters, join_filter_docs) = partition_filter(f, join_dict);
            if !main_filters.is_empty() {
                if main_filters.len() == 1 {
                    main_doc = main_filters.into_iter().next().unwrap();
                } else {
                    main_doc.insert("$and", main_filters);
                }
            }
            join_filters = join_filter_docs;
        }

        (main_doc, join_filters)
    }

    /// Paginazione semplice: `find()` + `count_documents()`.
    async fn paginate_simple<T, M>(
        collection: &mongodb::Collection<M>,
        load_options: &LoadOptions,
        filter: &Document,
        map_fn: impl Fn(M) -> T + Send,
    ) -> CornettiResult<PaginationResult<T>>
    where
        T: Send,
        M: DeserializeOwned + Send + Sync + Unpin,
    {
        let sort_doc = Self::build_sort_inner(&load_options.sort, &HashMap::new());

        // Skip count se non richiesto
        let total_count = if load_options.require_total_count {
            collection.count_documents(filter.clone()).await? as i64
        } else {
            -1
        };

        let mut find_options = mongodb::options::FindOptions::default();
        if !sort_doc.is_empty() {
            find_options.sort = Some(sort_doc);
        }
        find_options.skip = Some(load_options.skip as u64);
        find_options.limit = Some(load_options.take);

        let cursor = collection.find(filter.clone()).with_options(find_options).await?;
        let models: Vec<M> = cursor.try_collect().await?;
        let data: Vec<T> = models.into_iter().map(map_fn).collect();

        Ok(PaginationResult {
            data,
            total_count,
        })
    }

    /// Paginazione con JOIN: aggregation pipeline con `$lookup`.
    ///
    /// Strategia: due pipeline separate.
    /// - Count pipeline: $match + $lookup + $match (NO $unwind) con $count.
    ///   Usa $filter + $size > 0 per verificare che il lookup abbia match,
    ///   senza duplicare documenti.
    /// - Data pipeline: $match + $lookup + $unwind + $match + $sort + $skip/$limit.
    ///   Qui $unwind è necessario per accedere ai campi joined nell'ordinamento.
    ///   Deduplicazione via $group alla fine per relazioni 1-to-many.
    async fn paginate_with_joins<T, M>(
        collection: &mongodb::Collection<M>,
        load_options: &LoadOptions,
        main_filter: &Document,
        join_entries: &[(&JoinEntry, Document)],
        join_dict: &HashMap<String, JoinEntry>,
        map_fn: impl Fn(M) -> T + Send,
    ) -> CornettiResult<PaginationResult<T>>
    where
        T: Send,
        M: DeserializeOwned + Send + Sync + Unpin,
    {
        // ─── Raggruppa join per target_entity (deduplica) ───────────
        let mut join_entities: HashMap<String, (&JoinEntry, Vec<&Document>)> = HashMap::new();
        for (entry, filter_doc) in join_entries {
            join_entities
                .entry(entry.target_entity.clone())
                .and_modify(|(_, filters)| filters.push(filter_doc))
                .or_insert((entry, vec![filter_doc]));
        }

        // Aggiungi join richieste solo dal sort (non presenti nei filtri)
        for s in &load_options.sort {
            if let Some(entry) = join_dict.get(&s.field) {
                join_entities
                    .entry(entry.target_entity.clone())
                    .or_insert((entry, Vec::new()));
            }
        }

        // ─── Count pipeline (senza $unwind per evitare inflazione) ──
        let total_count = if load_options.require_total_count {
            let mut count_pipeline: Vec<Document> = Vec::new();

            if !main_filter.is_empty() {
                count_pipeline.push(doc! { "$match": main_filter.clone() });
            }

            // Per ogni join, $lookup + $match con $expr/$size per filtrare
            // documenti che hanno almeno un match, senza duplicare
            for (entity_name, (entry, filters)) in &join_entities {
                count_pipeline.push(doc! {
                    "$lookup": {
                        "from": entity_name.clone(),
                        "localField": entry.foreign_key.clone(),
                        "foreignField": entry.target_pk.clone(),
                        "as": entry.virtual_field.clone(),
                    }
                });

                // Applica filtri sui campi joined usando array notation
                for filter_doc in filters {
                    if !filter_doc.is_empty() {
                        count_pipeline.push(doc! { "$match": (*filter_doc).clone() });
                    }
                }

                // Se inner join: richiedi almeno un risultato nel lookup
                if !entry.outer_join {
                    count_pipeline.push(doc! {
                        "$match": {
                            entry.virtual_field.clone(): { "$ne": [] }
                        }
                    });
                }
            }

            count_pipeline.push(doc! { "$count": "count" });

            let mut cursor = collection.aggregate(count_pipeline).await?;
            if let Some(result) = cursor.next().await {
                let doc = result?;
                doc.get_i64("count").unwrap_or(0)
            } else {
                0i64
            }
        } else {
            -1i64
        };

        // ─── Data pipeline (con $unwind per accesso campi joined) ───
        let mut pipeline: Vec<Document> = Vec::new();

        if !main_filter.is_empty() {
            pipeline.push(doc! { "$match": main_filter.clone() });
        }

        for (entity_name, (entry, filters)) in &join_entities {
            let has_outer = entry.outer_join;

            pipeline.push(doc! {
                "$lookup": {
                    "from": entity_name.clone(),
                    "localField": entry.foreign_key.clone(),
                    "foreignField": entry.target_pk.clone(),
                    "as": entry.virtual_field.clone(),
                }
            });

            if !has_outer {
                pipeline.push(doc! {
                    "$unwind": {
                        "path": format!("${}", entry.virtual_field),
                    }
                });
            } else {
                pipeline.push(doc! {
                    "$unwind": {
                        "path": format!("${}", entry.virtual_field),
                        "preserveNullAndEmptyArrays": true,
                    }
                });
            }

            for filter_doc in filters {
                if !filter_doc.is_empty() {
                    pipeline.push(doc! { "$match": (*filter_doc).clone() });
                }
            }
        }

        // Deduplicazione: $group per _id per evitare righe duplicate
        // da relazioni 1-to-many dopo $unwind.
        // Usa $$ROOT per mantenere il documento originale.
        pipeline.push(doc! {
            "$group": {
                "_id": "$_id",
                "_doc": { "$first": "$$ROOT" },
            }
        });
        pipeline.push(doc! { "$replaceRoot": { "newRoot": "$_doc" } });

        // Sort
        let sort_doc = Self::build_sort_inner(&load_options.sort, join_dict);
        if !sort_doc.is_empty() {
            pipeline.push(doc! { "$sort": sort_doc });
        }

        // Paginazione
        pipeline.push(doc! { "$skip": load_options.skip });
        pipeline.push(doc! { "$limit": load_options.take });

        let cursor = collection.aggregate(pipeline).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;

        let mut data: Vec<T> = Vec::new();
        for item_doc in docs {
            if let Ok(model) = bson::deserialize_from_document::<M>(item_doc) {
                data.push(map_fn(model));
            }
        }

        Ok(PaginationResult {
            data,
            total_count,
        })
    }

    fn build_sort_inner(
        sort: &[SortDescriptor],
        join_dict: &HashMap<String, JoinEntry>,
    ) -> Document {
        let mut doc = Document::new();
        for s in sort {
            let field_name = resolve_mongo_field(&s.field, join_dict);
            let val: i32 = match s.direction {
                SortDirection::Asc => 1,
                SortDirection::Desc => -1,
            };
            doc.insert(field_name, val);
        }
        doc
    }
}

/// Partiziona un `FilterNode` in:
/// - `Vec<Document>` per la collezione principale (campi non in join_dict)
/// - `Vec<(&JoinEntry, Document)>` per le collezioni joined (campi in join_dict)
fn partition_filter<'a>(
    node: &FilterNode,
    join_dict: &'a HashMap<String, JoinEntry>,
) -> (Vec<Document>, Vec<(&'a JoinEntry, Document)>) {
    match node {
        FilterNode::Leaf {
            field,
            operator,
            value,
        } => {
            let doc = leaf_to_bson(field, *operator, value);
            if let Some(entry) = join_dict.get(field) {
                // Campo joined: filtro applicato DOPO il $lookup,
                // riferito al campo nell'array risultante dal lookup.
                let join_doc = leaf_to_bson(
                    &format!("{}.{}", entry.virtual_field, entry.target_field),
                    *operator,
                    value,
                );
                (Vec::new(), vec![(entry, join_doc)])
            } else {
                (vec![doc], Vec::new())
            }
        }
        FilterNode::Group { operator, children } => {
            let mut main_docs = Vec::new();
            let mut join_docs: Vec<(&JoinEntry, Document)> = Vec::new();

            for child in children {
                let (m, j) = partition_filter(child, join_dict);
                main_docs.extend(m);
                join_docs.extend(j);
            }

            // Raggruppa i filtri main con l'operatore
            if main_docs.len() > 1 {
                let op = match operator {
                    GroupOperator::And => "$and",
                    GroupOperator::Or => "$or",
                };
                main_docs = vec![doc! { op: main_docs }];
            }

            (main_docs, join_docs)
        }
        FilterNode::Not(inner) => {
            let (mut main_docs, join_docs) = partition_filter(inner, join_dict);
            if main_docs.len() == 1 {
                let inner_doc = main_docs.remove(0);
                let mut not_doc = Document::new();
                for (k, v) in inner_doc {
                    not_doc.insert(k, doc! { "$not": { "$eq": v } });
                }
                main_docs = vec![not_doc];
            } else if !main_docs.is_empty() {
                // $nor nega un array di condizioni (equivale a NOT(cond1 AND cond2))
                main_docs = vec![doc! { "$nor": main_docs }];
            }
            (main_docs, join_docs)
        }
    }
}

/// Converte un FilterNode ricorsivamente in un Document BSON.
fn filter_to_bson(node: &FilterNode, join_dict: &HashMap<String, JoinEntry>) -> Document {
    match node {
        FilterNode::Leaf {
            field,
            operator,
            value,
        } => {
            let resolved = resolve_mongo_field(field, join_dict);
            leaf_to_bson(&resolved, *operator, value)
        }
        FilterNode::Group { operator, children } => {
            let docs: Vec<Document> = children
                .iter()
                .map(|c| filter_to_bson(c, join_dict))
                .collect();
            let op = match operator {
                GroupOperator::And => "$and",
                GroupOperator::Or => "$or",
            };
            if docs.len() == 1 {
                docs.into_iter().next().unwrap()
            } else {
                doc! { op: docs }
            }
        }
        FilterNode::Not(inner) => {
            let inner_doc = filter_to_bson(inner, join_dict);
            // $not non esiste a livello root in MongoDB.
            // Usiamo $nor che nega un array di espressioni.
            doc! { "$nor": [ inner_doc ] }
        }
    }
}

/// Costruisce il Document BSON per una foglia di filtro.
fn leaf_to_bson(field: &str, operator: FilterOperator, value: &FilterValue) -> Document {
    match operator {
        FilterOperator::Contains => {
            let s = value.as_str_repr();
            doc! { field: { "$regex": s } }
        }
        FilterOperator::NotContains => {
            let s = value.as_str_repr();
            doc! { field: { "$not": { "$regex": s } } }
        }
        FilterOperator::StartsWith => {
            let s = value.as_str_repr();
            doc! { field: { "$regex": format!("^{}", s) } }
        }
        FilterOperator::EndsWith => {
            let s = value.as_str_repr();
            doc! { field: { "$regex": format!("{}$", s) } }
        }
        FilterOperator::Eq => {
            let val = filter_value_to_bson(value);
            doc! { field: val }
        }
        FilterOperator::NotEq => {
            let val = filter_value_to_bson(value);
            doc! { field: { "$ne": val } }
        }
        FilterOperator::Gt => {
            let val = filter_value_to_bson(value);
            doc! { field: { "$gt": val } }
        }
        FilterOperator::Gte => {
            let val = filter_value_to_bson(value);
            doc! { field: { "$gte": val } }
        }
        FilterOperator::Lt => {
            let val = filter_value_to_bson(value);
            doc! { field: { "$lt": val } }
        }
        FilterOperator::Lte => {
            let val = filter_value_to_bson(value);
            doc! { field: { "$lte": val } }
        }
    }
}

/// Converte un FilterValue nel tipo BSON appropriato.
fn filter_value_to_bson(value: &FilterValue) -> bson::Bson {
    match value {
        FilterValue::String(s) => bson::Bson::String(s.clone()),
        FilterValue::Integer(n) => bson::Bson::Int64(*n),
        FilterValue::Float(n) => bson::Bson::Double(*n),
        FilterValue::Boolean(b) => bson::Bson::Boolean(*b),
        FilterValue::Null => bson::Bson::Null,
    }
}

/// Risolve il nome campo per MongoDB.
///
/// Se il campo è in join_dict, restituisce `virtual_field.target_field`
/// (percorso dopo `$lookup`). Altrimenti restituisce il campo così com'è.
fn resolve_mongo_field(field: &str, join_dict: &HashMap<String, JoinEntry>) -> String {
    if let Some(entry) = join_dict.get(field) {
        format!("{}.{}", entry.virtual_field, entry.target_field)
    } else {
        field.to_string()
    }
}
