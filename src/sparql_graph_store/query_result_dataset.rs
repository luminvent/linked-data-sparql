use crate::FromRdfSubject;
use oxrdf::{Dataset, NamedOrBlankNode};
use std::collections::HashSet;

pub struct QueryResultDataset {
  dataset: Dataset,
}

impl QueryResultDataset {
  pub fn new(dataset: Dataset) -> Self {
    Self { dataset }
  }

  fn subjects(&self) -> impl Iterator<Item = NamedOrBlankNode> {
    self
      .dataset
      .iter()
      .map(|quad| quad.subject.into_owned())
      .collect::<HashSet<_>>()
      .into_iter()
  }

  pub fn deserialize_subject_with_resource_id<T: FromRdfSubject>(
    &self,
    resource_id: &NamedOrBlankNode,
  ) -> Option<T> {
    T::deserialize_subject(&self.dataset, resource_id).ok()
  }

  pub fn deserialize_subject<T: FromRdfSubject>(&self) -> Option<T> {
    self
      .subjects()
      .find_map(|resource_id| T::deserialize_subject(&self.dataset, &resource_id).ok())
  }

  pub fn deserialize_subject_with_resource_ids<'a, T: FromRdfSubject>(
    &self,
    resource_ids: impl Iterator<Item = &'a NamedOrBlankNode>,
  ) -> Vec<T> {
    resource_ids
      .filter_map(|resource_id| T::deserialize_subject(&self.dataset, resource_id).ok())
      .collect()
  }

  pub fn deserialize_subjects<T: FromRdfSubject>(&self) -> Vec<T> {
    self
      .subjects()
      .filter_map(|resource_id| T::deserialize_subject(&self.dataset, &resource_id).ok())
      .collect()
  }

  pub fn resource_ids(&self) -> Vec<NamedOrBlankNode> {
    self.subjects().collect()
  }
}
