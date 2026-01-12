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
    let resource_ids = self.dataset.resources().cloned().collect::<Vec<_>>();

    resource_ids.iter().find_map(|resource_id| {
      T::deserialize_subject(&(), &(), &self.dataset, None, resource_id).ok()
    })
  }

  pub fn deserialize_subject_with_resource_ids<T: LinkedDataDeserializeSubject>(
    &self,
    resource_ids: &[rdf_types::Term],
  ) -> Vec<T> {
    resource_ids
      .iter()
      .filter_map(|resource_id| {
        T::deserialize_subject(&(), &(), &self.dataset, None, resource_id).ok()
      })
      .collect()
  }

  pub fn deserialize_subjects<T: LinkedDataDeserializeSubject>(&self) -> Vec<T> {
    let resource_ids = self.dataset.resources().cloned().collect::<Vec<_>>();

    resource_ids
      .iter()
      .filter_map(|resource_id| {
        T::deserialize_subject(&(), &(), &self.dataset, None, resource_id).ok()
      })
      .collect()
  }
}
