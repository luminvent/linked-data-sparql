use super::{QueryResults, SparqlGraphStore};

use oxigraph::sparql::UpdateEvaluationError;
use oxttl::NTriplesParser;
use sparesults::{QueryResultsFormat, QueryResultsParser, ReaderQueryResultsParserOutput};
use spareval::QueryEvaluationError;
use spargebra::{Query, Update};
use std::io::Cursor;

#[derive(Clone)]
pub struct SparqlClientDatabase {
  update_server_endpoint: String,
  query_server_endpoint: String,
}

impl SparqlClientDatabase {
  pub fn new(update_server_endpoint: &str, query_server_endpoint: &str) -> Self {
    Self {
      update_server_endpoint: update_server_endpoint.to_owned(),
      query_server_endpoint: query_server_endpoint.to_owned(),
    }
  }
}

impl SparqlGraphStore for SparqlClientDatabase {
  async fn update(&self, update: Update) -> Result<(), UpdateEvaluationError> {
    let client = reqwest::Client::new();
    let url = self.update_server_endpoint.clone();

    #[derive(serde::Serialize)]
    struct SparqlUpdateForm {
      update: String,
    }

    let update = SparqlUpdateForm {
      update: update.to_string(),
    };

    let response = client
      .post(&url)
      .form(&update)
      .send()
      .await
      .map_err(|error| UpdateEvaluationError::Service(Box::new(error)))?;

    if !response.status().is_success() {
      let message = response.text().await.unwrap();

      let error = std::io::Error::other(message);

      return Err(UpdateEvaluationError::Service(Box::new(error)));
    }

    Ok(())
  }

  async fn query(&self, query: Query) -> Result<QueryResults, QueryEvaluationError> {
    let client = reqwest::Client::new();
    let url = self.query_server_endpoint.clone();

    #[derive(serde::Serialize)]
    struct SparqlQueryForm {
      query: String,
    }

    let query = SparqlQueryForm {
      query: query.to_string(),
    };

    let response = client
      .post(&url)
      .form(&query)
      .send()
      .await
      .map_err(|error| QueryEvaluationError::Service(Box::new(error)))?;

    if !response.status().is_success() {
      let message = response.text().await.unwrap();

      let error = std::io::Error::other(message);

      return Err(QueryEvaluationError::Service(Box::new(error)));
    }

    let body = response.text().await.unwrap();

    let json_parser = QueryResultsParser::from_format(QueryResultsFormat::Json);

    let query_results = match json_parser.clone().for_reader(Cursor::new(body.clone())) {
      Ok(ReaderQueryResultsParserOutput::Solutions(solutions)) => {
        let variables = solutions.variables().to_vec();
        let solutions = solutions.flatten().collect();

        QueryResults::Solutions {
          variables,
          solutions,
        }
      }
      Ok(ReaderQueryResultsParserOutput::Boolean(boolean)) => QueryResults::Boolean(boolean),
      _ => {
        let content = body.as_bytes();
        let ntriple_parser = NTriplesParser::new().for_slice(content);

        let triples: Vec<_> = ntriple_parser.flatten().collect();

        QueryResults::Triples(triples)
      }
    };

    Ok(query_results)
  }
}
