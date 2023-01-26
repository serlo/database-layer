use std::collections::{BTreeSet, HashSet};

use sophia::graph::{inmem::FastGraph, *};
use sophia::iri::Iri;
use sophia::ns::Namespace;
use sophia::prefix::Prefix;
use sophia::serializer::turtle::TurtleConfig;
use sophia::serializer::*;
use sophia::term::literal::Literal;
use thiserror::Error;

use crate::database::Executor;
use crate::instance::Instance;
use crate::uuid::TaxonomyType;

pub struct Vocabulary;

const BASE: &str = "https://vocabulary.serlo.org/";

impl Vocabulary {
    pub async fn fetch_taxonomy_vocabulary<'a, E>(
        instance: Instance,
        executor: E,
    ) -> Result<String, VocabularyError>
    where
        E: Executor<'a>,
    {
        let base = format!("{BASE}{instance}/taxonomy/");
        let title = match instance {
            Instance::De => "de.serlo.org Taxonomie",
            Instance::En => "en.serlo.org Taxonomy",
            Instance::Es => "es.serlo.org Taxonomía",
            Instance::Fr => "fr.serlo.org Taxonomie",
            Instance::Hi => "hi.serlo.org वर्गीकरण",
            Instance::Ta => "ta.serlo.org வகைப்பாடு",
        };
        let creator = "Serlo Education e.V.";
        let types_allowlist = vec![
            TaxonomyType::Subject,
            TaxonomyType::Topic,
            TaxonomyType::TopicFolder,
        ];
        let terms_blocklist: Vec<i64> = vec![
            87993,  // de.serlo.org Community
            93176,  // de.serlo.org Testbereich
            181883, // de.serlo.org Lerntipps
        ];
        Self::fetch_taxonomy_terms_vocabulary(
            &base,
            title,
            creator,
            instance,
            &types_allowlist,
            &terms_blocklist,
            executor,
        )
        .await
    }

    async fn fetch_taxonomy_terms_vocabulary<'a, E>(
        base: &str,
        title: &str,
        creator: &str,
        instance: Instance,
        types_allowlist: &[TaxonomyType],
        terms_blocklist: &[i64],
        executor: E,
    ) -> Result<String, VocabularyError>
    where
        E: Executor<'a>,
    {
        const DCT: &str = "http://purl.org/dc/terms/";
        const SKOS: &str = "http://www.w3.org/2004/02/skos/core#";

        let types_allowlist: HashSet<TaxonomyType> = types_allowlist.iter().cloned().collect();
        let mut terms_blocklist: BTreeSet<i64> = terms_blocklist.iter().cloned().collect();

        let a_token = Iri::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?;
        let lang = format!("{instance}");

        let dct = Namespace::new(DCT)?;
        let skos = Namespace::new(SKOS)?;

        let terms = sqlx::query_as!(
            TaxonomyTerm,
            r#"
                SELECT tt.id, t.name, tt.parent_id, u.trashed, type.name AS typename
                FROM term t
                        JOIN term_taxonomy tt ON t.id = tt.term_id
                        JOIN taxonomy ON tt.taxonomy_id = taxonomy.id
                        JOIN type ON taxonomy.type_id = type.id
                        JOIN uuid u on tt.id = u.id
                        JOIN instance i ON t.instance_id = i.id
                WHERE i.subdomain = ?
            "#,
            instance
        )
        .fetch_all(executor)
        .await?;

        let (root_id, terms) = Self::sort_terms_by_depth(&terms)?;

        let mut graph = FastGraph::new();

        for term in terms {
            if term.trashed != 0 {
                continue;
            }

            if terms_blocklist.contains(&term.id) {
                continue;
            }

            let typename = term
                .typename
                .parse()
                .map_err(|_| VocabularyError::InvalidTaxonomyType)?;
            if typename != TaxonomyType::Root && !types_allowlist.contains(&typename) {
                continue;
            }

            if term.id == root_id {
                graph.insert(
                    &Iri::new(format!("{}{}", base, term.id).as_str())?,
                    &a_token,
                    &skos.get("ConceptScheme")?,
                )?;
                graph.insert(
                    &Iri::new(format!("{}{}", base, term.id).as_str())?,
                    &dct.get("title")?,
                    &Literal::<String>::new_lang_unchecked(title, lang.clone()),
                )?;
                graph.insert(
                    &Iri::new(format!("{}{}", base, term.id).as_str())?,
                    &dct.get("creator")?,
                    &Literal::<String>::new_lang_unchecked(creator, lang.clone()),
                )?;
            } else if let Some(parent_id) = term.parent_id {
                if terms_blocklist.contains(&parent_id) {
                    terms_blocklist.insert(term.id);
                    continue;
                }

                graph.insert(
                    &Iri::new(format!("{}{}", base, term.id).as_str())?,
                    &a_token,
                    &skos.get("Concept")?,
                )?;

                graph.insert(
                    &Iri::new(format!("{}{}", base, term.id).as_str())?,
                    &skos.get("prefLabel")?,
                    &Literal::<String>::new_lang_unchecked(term.name.clone(), lang.clone()),
                )?;

                if parent_id == root_id {
                    graph.insert(
                        &Iri::new(format!("{base}{root_id}").as_str())?,
                        &skos.get("hasTopConcept")?,
                        &Iri::new(format!("{}{}", base, term.id).as_str())?,
                    )?;
                    graph.insert(
                        &Iri::new(format!("{}{}", base, term.id).as_str())?,
                        &skos.get("topConceptOf")?,
                        &Iri::new(format!("{base}{root_id}").as_str())?,
                    )?;
                } else {
                    graph.insert(
                        &Iri::new(format!("{base}{parent_id}").as_str())?,
                        &skos.get("narrower")?,
                        &Iri::new(format!("{}{}", base, term.id).as_str())?,
                    )?;
                    graph.insert(
                        &Iri::new(format!("{}{}", base, term.id).as_str())?,
                        &skos.get("broader")?,
                        &Iri::new(format!("{base}{parent_id}").as_str())?,
                    )?;
                    graph.insert(
                        &Iri::new(format!("{}{}", base, term.id).as_str())?,
                        &skos.get("inScheme")?,
                        &Iri::new(format!("{}{}", base, 3).as_str())?,
                    )?;
                }
            }
        }

        let prefixes = [
            (Prefix::new_unchecked("dct"), Iri::new_unchecked(DCT)),
            (Prefix::new_unchecked("skos"), Iri::new_unchecked(SKOS)),
        ];
        let config = TurtleConfig::new()
            .with_pretty(true)
            .with_prefix_map(&prefixes[..]);

        let mut output = vec![];
        let mut serializer =
            sophia::serializer::turtle::TurtleSerializer::new_with_config(&mut output, config);
        serializer.serialize_graph(&graph)?;

        let output = format!(
            "BASE <{}>\n{}",
            base,
            String::from_utf8(output)?.replace(base, "")
        );
        Ok(output)
    }

    fn sort_terms_by_depth(
        unsorted_terms: &[TaxonomyTerm],
    ) -> Result<(i64, Vec<&TaxonomyTerm>), VocabularyError> {
        let mut sorted_terms = Vec::with_capacity(unsorted_terms.len());

        let (mut roots, mut remaining_terms): (Vec<_>, Vec<_>) = unsorted_terms
            .iter()
            .partition(|term| term.parent_id.is_none());
        if roots.len() != 1 {
            return Err(VocabularyError::InvalidTree);
        }
        let root_id = roots.first().unwrap().id;
        sorted_terms.append(&mut roots);

        let mut previous_parents: BTreeSet<Option<i64>> = [Some(root_id)].iter().cloned().collect();

        while !remaining_terms.is_empty() {
            let (mut terms_in_current_depth, terms_in_deeper_depths): (Vec<_>, Vec<_>) =
                remaining_terms
                    .into_iter()
                    .partition(|term| previous_parents.contains(&term.parent_id));

            if terms_in_current_depth.is_empty() {
                return Err(VocabularyError::InvalidTree);
            }

            remaining_terms = terms_in_deeper_depths;
            previous_parents = terms_in_current_depth
                .iter()
                .map(|term| Some(term.id))
                .collect();
            sorted_terms.append(&mut terms_in_current_depth);
        }

        Ok((root_id, sorted_terms))
    }
}

struct TaxonomyTerm {
    id: i64,
    name: String,
    parent_id: Option<i64>,
    trashed: i8,
    typename: String,
}

#[derive(Error, Debug)]
pub enum VocabularyError {
    #[error("Database error: {0:?}.")]
    DatabaseError(#[from] sqlx::Error),
    #[error("From UTF-8 error: {0:?}.")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Infallible error: {0:?}.")]
    Infallible(#[from] std::convert::Infallible),
    #[error("Invalid IRI error: {0:?}.")]
    InvalidIri(#[from] sophia::iri::error::InvalidIri),
    #[error("Invalid taxonomy type.")]
    InvalidTaxonomyType,
    #[error("Invalid tree.")]
    InvalidTree,
    #[error("Stream error: {0:?}.")]
    StreamError(
        #[from] sophia::quad::stream::StreamError<std::convert::Infallible, std::io::Error>,
    ),
}
