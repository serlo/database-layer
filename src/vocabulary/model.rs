use sophia::graph::{inmem::FastGraph, *};
use sophia::iri::Iri;
use sophia::ns::Namespace;
use sophia::prefix::Prefix;
use sophia::serializer::turtle::TurtleConfig;
use sophia::serializer::*;
use sophia::term::literal::Literal;

use crate::database::Executor;
use crate::instance::Instance;

pub struct Vocabulary;

impl Vocabulary {
    pub async fn fetch_taxonomy_vocabulary<'a, E>(
        instance: Instance,
        executor: E,
        // TODO: better error handling
    ) -> Result<String, anyhow::Error>
    where
        E: Executor<'a>,
    {
        pub const BASE: &str = "https://vocabs.serlo.org/terms/";

        let a_token = Iri::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?;
        let lang = String::from(format!("{}", instance));

        let dct = Namespace::new("http://purl.org/dc/terms/")?;
        let skos = Namespace::new("http://www.w3.org/2004/02/skos/core#")?;

        let terms = sqlx::query!(
            r#"
                SELECT tt.id, t.name, tt.parent_id, u.trashed
                FROM term t
                         JOIN term_taxonomy tt ON t.id = tt.term_id
                         JOIN uuid u on tt.id = u.id
                         JOIN instance i ON t.instance_id = i.id
                WHERE i.subdomain = ?
            "#,
            instance
        )
        .fetch_all(executor)
        .await?;

        // TODO: error handling
        let root = terms.iter().find(|term| term.parent_id.is_none()).unwrap();
        let root_id = root.id;

        let mut graph = FastGraph::new();

        for term in terms {
            if term.trashed != 0 {
                continue;
            }

            if term.id == root_id {
                graph.insert(
                    &Iri::new(format!("{}{}", BASE, term.id).as_str())?,
                    &a_token,
                    &skos.get("ConceptScheme")?,
                )?;

                graph.insert(
                    &Iri::new(format!("{}{}", BASE, term.id).as_str())?,
                    &dct.get("title")?,
                    // TODO: i18n
                    &Literal::<String>::new_lang_unchecked("Taxonomie", lang.clone()),
                )?;

                graph.insert(
                    &Iri::new(format!("{}{}", BASE, term.id).as_str())?,
                    &dct.get("creator")?,
                    &Literal::<String>::new_lang_unchecked(
                        format!("{}.serlo.org", lang),
                        lang.clone(),
                    ),
                )?;
            } else if let Some(parent_id) = term.parent_id {
                graph.insert(
                    &Iri::new(format!("{}{}", BASE, term.id).as_str())?,
                    &a_token,
                    &skos.get("Concept")?,
                )?;

                graph.insert(
                    &Iri::new(format!("{}{}", BASE, term.id).as_str())?,
                    &skos.get("prefLabel")?,
                    &Literal::<String>::new_lang_unchecked(term.name, lang.clone()),
                )?;

                if parent_id == root_id {
                    graph.insert(
                        &Iri::new(format!("{}{}", BASE, root_id).as_str())?,
                        &skos.get("hasTopConcept")?,
                        &Iri::new(format!("{}{}", BASE, term.id).as_str())?,
                    )?;
                    graph.insert(
                        &Iri::new(format!("{}{}", BASE, term.id).as_str())?,
                        &skos.get("topConceptOf")?,
                        &Iri::new(format!("{}{}", BASE, root_id).as_str())?,
                    )?;
                } else {
                    graph.insert(
                        &Iri::new(format!("{}{}", BASE, parent_id).as_str())?,
                        &skos.get("narrower")?,
                        &Iri::new(format!("{}{}", BASE, term.id).as_str())?,
                    )?;
                    graph.insert(
                        &Iri::new(format!("{}{}", BASE, term.id).as_str())?,
                        &skos.get("broader")?,
                        &Iri::new(format!("{}{}", BASE, parent_id).as_str())?,
                    )?;
                    graph.insert(
                        &Iri::new(format!("{}{}", BASE, term.id).as_str())?,
                        &skos.get("inScheme")?,
                        &Iri::new(format!("{}{}", BASE, 3).as_str())?,
                    )?;
                }
            }
        }

        let prefixes = [
            (
                Prefix::new_unchecked("dct"),
                Iri::new_unchecked("http://purl.org/dc/terms/"),
            ),
            (
                Prefix::new_unchecked("skos"),
                Iri::new_unchecked("http://www.w3.org/2004/02/skos/core#"),
            ),
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
            BASE,
            String::from_utf8(output)?.replace(BASE, "")
        );
        Ok(output)
    }
}
