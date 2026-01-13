use linked_data_next::LinkedDataDeserializeSubject;
use rdf_types::dataset::IndexedBTreeDataset;

pub struct QueryResultDataset {
  dataset: IndexedBTreeDataset,
}

impl QueryResultDataset {
  pub fn new(dataset: IndexedBTreeDataset) -> Self {
    Self { dataset }
  }

  pub fn deserialize_subject_with_resource_id<T: LinkedDataDeserializeSubject>(
    &self,
    resource_id: &rdf_types::Term,
  ) -> Option<T> {
    T::deserialize_subject(&(), &(), &self.dataset, None, resource_id).ok()
  }

  pub fn deserialize_subject<T: LinkedDataDeserializeSubject>(&self) -> Option<T> {
    self.dataset.resources().find_map(|resource_id| {
      T::deserialize_subject(&(), &(), &self.dataset, None, resource_id).ok()
    })
  }

  pub fn deserialize_subject_with_resource_ids<'a, T: LinkedDataDeserializeSubject>(
    &self,
    resource_ids: impl Iterator<Item = &'a rdf_types::Term>,
  ) -> Vec<T> {
    resource_ids
      .filter_map(|resource_id| {
        T::deserialize_subject(&(), &(), &self.dataset, None, resource_id).ok()
      })
      .collect()
  }

  pub fn deserialize_subjects<T: LinkedDataDeserializeSubject>(&self) -> Vec<T> {
    self
      .dataset
      .resources()
      .filter_map(|resource_id| {
        T::deserialize_subject(&(), &(), &self.dataset, None, resource_id).ok()
      })
      .collect()
  }

  pub fn resource_ids(&self) -> impl Iterator<Item = &rdf_types::Term> {
    self.dataset.resources()
  }
}
