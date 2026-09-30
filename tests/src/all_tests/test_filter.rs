use linked_data_sparql::reexport::oxrdf::NamedNode;
use linked_data_sparql::sparql_graph_store::{
  OxigraphSparqlGraphStore, SparqlGraphStore, UpdateAction,
};
use linked_data_sparql::{Deserialize, Serialize, Sparql, SparqlFilter, ToConstructQuery};

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct Book {
  #[ld(id)]
  id: NamedNode,

  #[ld("ex:title")]
  title: String,

  #[ld("ex:published")]
  published: xsd_types::DateTime,

  #[ld("ex:pages")]
  pages: u32,

  #[ld("ex:deletedAt")]
  deleted_at: Option<xsd_types::DateTime>,
}

#[derive(SparqlFilter, Default)]
#[ld(prefix("ex" = "http://ex/"))]
struct BookFilter {
  #[ld("ex:title")]
  #[filter(str_starts)]
  title_prefix: Option<String>,

  #[ld("ex:published")]
  #[filter(year)]
  published_year: Option<i32>,

  #[ld("ex:pages")]
  pages: Option<u32>,

  #[ld("ex:pages")]
  #[filter(ne)]
  pages_not: Option<u32>,

  #[ld("ex:pages")]
  #[filter(>=)]
  min_pages: Option<u32>,

  #[ld("ex:pages")]
  #[filter(lt)]
  max_pages_exclusive: Option<u32>,

  #[ld("ex:pages")]
  #[filter(>)]
  more_pages_than: Option<u32>,

  #[ld("ex:pages")]
  #[filter(<=)]
  max_pages: Option<u32>,

  #[ld("ex:title")]
  #[filter(regex)]
  title_regex: Option<String>,

  #[ld("ex:title")]
  #[filter(regex, case_insensitive)]
  title_iregex: Option<String>,

  #[ld("ex:deletedAt")]
  #[filter(not_exists)]
  exclude_deleted: bool,

  #[ld("ex:deletedAt")]
  #[filter(exists)]
  only_deleted: bool,

  #[ld("ex:deletedAt")]
  #[filter(<)]
  deleted_before: Option<xsd_types::DateTime>,

  #[ld("ex:deletedAt")]
  #[filter(<, optional)]
  not_deleted_or_deleted_before: Option<xsd_types::DateTime>,
}

fn book(id: &str, title: &str, published: &str, pages: u32, deleted_at: Option<&str>) -> Book {
  Book {
    id: NamedNode::new(format!("http://ex/book/{id}")).unwrap(),
    title: title.to_owned(),
    published: published.parse().unwrap(),
    pages,
    deleted_at: deleted_at.map(|date| date.parse().unwrap()),
  }
}

async fn query_titles(store: &OxigraphSparqlGraphStore, filter: &BookFilter) -> Vec<String> {
  let query = Book::to_filtered_query(filter).into();

  let dataset = store
    .query(query)
    .await
    .unwrap()
    .get_query_result_dataset()
    .unwrap();

  let mut titles: Vec<String> = dataset
    .resource_ids()
    .iter()
    .filter_map(|resource_id| dataset.deserialize_subject_with_resource_id::<Book>(resource_id))
    .map(|book| book.title)
    .collect();

  titles.sort();

  titles
}

#[tokio::test]
async fn test_filter() {
  let store = OxigraphSparqlGraphStore::default();

  for book in [
    book("1", "Rust in Action", "2021-08-10T00:00:00Z", 456, None),
    book(
      "2",
      "Rust for Rustaceans",
      "2021-12-01T00:00:00Z",
      280,
      None,
    ),
    book("3", "Programming Rust", "2017-12-01T00:00:00Z", 622, None),
    book(
      "4",
      "Rust Atomics",
      "2023-01-01T00:00:00Z",
      250,
      Some("2024-01-01T00:00:00Z"),
    ),
  ] {
    store
      .default_insert(&book, UpdateAction::Insert)
      .await
      .unwrap();
  }

  assert_eq!(query_titles(&store, &BookFilter::default()).await.len(), 4);

  let starts_with_rust = BookFilter {
    title_prefix: Some("Rust".to_owned()),
    ..Default::default()
  };
  assert_eq!(
    query_titles(&store, &starts_with_rust).await,
    ["Rust Atomics", "Rust for Rustaceans", "Rust in Action"]
  );

  let published_2021 = BookFilter {
    published_year: Some(2021),
    ..Default::default()
  };
  assert_eq!(
    query_titles(&store, &published_2021).await,
    ["Rust for Rustaceans", "Rust in Action"]
  );

  let not_deleted_starting_with_rust = BookFilter {
    title_prefix: Some("Rust".to_owned()),
    exclude_deleted: true,
    ..Default::default()
  };
  assert_eq!(
    query_titles(&store, &not_deleted_starting_with_rust).await,
    ["Rust for Rustaceans", "Rust in Action"]
  );

  let exact_pages = BookFilter {
    pages: Some(622),
    ..Default::default()
  };
  assert_eq!(
    query_titles(&store, &exact_pages).await,
    ["Programming Rust"]
  );

  let filtered = async |filter: BookFilter| query_titles(&store, &filter).await;

  assert_eq!(
    filtered(BookFilter {
      pages_not: Some(622),
      ..Default::default()
    })
    .await,
    ["Rust Atomics", "Rust for Rustaceans", "Rust in Action"]
  );
  assert_eq!(
    filtered(BookFilter {
      min_pages: Some(456),
      ..Default::default()
    })
    .await,
    ["Programming Rust", "Rust in Action"]
  );
  assert_eq!(
    filtered(BookFilter {
      max_pages_exclusive: Some(280),
      ..Default::default()
    })
    .await,
    ["Rust Atomics"]
  );
  assert_eq!(
    filtered(BookFilter {
      more_pages_than: Some(456),
      ..Default::default()
    })
    .await,
    ["Programming Rust"]
  );
  assert_eq!(
    filtered(BookFilter {
      max_pages: Some(280),
      ..Default::default()
    })
    .await,
    ["Rust Atomics", "Rust for Rustaceans"]
  );
  assert_eq!(
    filtered(BookFilter {
      min_pages: Some(260),
      max_pages: Some(500),
      ..Default::default()
    })
    .await,
    ["Rust for Rustaceans", "Rust in Action"]
  );

  assert_eq!(
    filtered(BookFilter {
      title_regex: Some("^Rust (in|for) ".to_owned()),
      ..Default::default()
    })
    .await,
    ["Rust for Rustaceans", "Rust in Action"]
  );
  assert!(
    filtered(BookFilter {
      title_regex: Some("rust$".to_owned()),
      ..Default::default()
    })
    .await
    .is_empty()
  );
  assert_eq!(
    filtered(BookFilter {
      title_iregex: Some("rust$".to_owned()),
      ..Default::default()
    })
    .await,
    ["Programming Rust"]
  );

  assert_eq!(
    filtered(BookFilter {
      only_deleted: true,
      ..Default::default()
    })
    .await,
    ["Rust Atomics"]
  );

  let date = |date: &str| Some(date.parse::<xsd_types::DateTime>().unwrap());

  // Without `optional`, a book lacking `ex:deletedAt` never passes a condition on it.
  assert_eq!(
    filtered(BookFilter {
      deleted_before: date("2025-01-01T00:00:00Z"),
      ..Default::default()
    })
    .await,
    ["Rust Atomics"]
  );
  // With `optional`, it does; the condition only applies to books that have the predicate.
  assert_eq!(
    filtered(BookFilter {
      not_deleted_or_deleted_before: date("2025-01-01T00:00:00Z"),
      ..Default::default()
    })
    .await
    .len(),
    4
  );
  assert_eq!(
    filtered(BookFilter {
      not_deleted_or_deleted_before: date("2023-06-01T00:00:00Z"),
      ..Default::default()
    })
    .await,
    ["Programming Rust", "Rust for Rustaceans", "Rust in Action"]
  );
}

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct Label {
  #[ld(id)]
  id: NamedNode,

  #[ld("ex:label")]
  label: String,
}

#[derive(SparqlFilter)]
#[ld(prefix("ex" = "http://ex/"))]
struct LabelFilter {
  #[ld("ex:label")]
  #[filter(lang)]
  lang: String,
}

#[tokio::test]
async fn test_filter_lang() {
  let store = OxigraphSparqlGraphStore::default();

  store
    .update(
      "INSERT DATA { <http://ex/a> <http://ex/label> \"bonjour\"@fr . <http://ex/b> <http://ex/label> \"hello\"@en-GB . }"
        .parse()
        .unwrap(),
    )
    .await
    .unwrap();

  let query = Label::to_filtered_query(&LabelFilter {
    lang: "en".to_owned(),
  })
  .into();
  let dataset = store
    .query(query)
    .await
    .unwrap()
    .get_query_result_dataset()
    .unwrap();

  let labels: Vec<String> = dataset
    .resource_ids()
    .iter()
    .filter_map(|resource_id| dataset.deserialize_subject_with_resource_id::<Label>(resource_id))
    .map(|label| label.label)
    .collect();

  assert_eq!(labels, ["hello"]);
}

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct Person {
  #[ld(id)]
  id: NamedNode,

  #[ld("ex:name")]
  name: String,

  #[ld("ex:email")]
  email: String,

  #[ld("ex:birth")]
  birth: xsd_types::DateTime,
}

#[derive(SparqlFilter, Default)]
#[ld(prefix("ex" = "http://ex/"))]
struct PersonFilter {
  #[ld("ex:email")]
  #[filter(str_ends)]
  email_domain_suffix: Option<String>,

  #[ld("ex:name")]
  #[filter(contains)]
  name_contains: Option<String>,

  #[ld("ex:email")]
  #[filter(str_before("@"))]
  email_user: Option<String>,

  #[ld("ex:email")]
  #[filter(str_after("@"), !=)]
  email_domain_not: Option<String>,

  #[ld("ex:name")]
  #[filter(str_length, >=)]
  name_min_length: Option<u32>,

  #[ld("ex:name")]
  #[filter(uppercase)]
  name_upper: Option<String>,

  #[ld("ex:name")]
  #[filter(lowercase, contains)]
  name_lower_contains: Option<String>,

  #[ld("ex:email")]
  #[filter(str_after("@"), lowercase, str_starts)]
  email_domain_lower_starts: Option<String>,

  #[ld("ex:birth")]
  #[filter(year, <)]
  born_before_year: Option<i32>,

  #[ld("ex:birth")]
  #[filter(>=, datatype = "xsd:dateTime")]
  born_from: Option<String>,

  #[ld("ex:birth")]
  #[filter(<, datatype = "http://www.w3.org/2001/XMLSchema#dateTime")]
  born_until: Option<String>,
}

#[tokio::test]
async fn test_filter_string_functions() {
  let store = OxigraphSparqlGraphStore::default();

  for (id, name, email, birth) in [
    (
      "1",
      "Alice Martin",
      "alice@Example.org",
      "1990-05-01T00:00:00Z",
    ),
    ("2", "Bob", "bob@test.net", "1985-02-11T00:00:00Z"),
    (
      "3",
      "Charlie Dupont",
      "charlie@example.org",
      "2001-09-30T00:00:00Z",
    ),
  ] {
    let person = Person {
      id: NamedNode::new(format!("http://ex/person/{id}")).unwrap(),
      name: name.to_owned(),
      email: email.to_owned(),
      birth: birth.parse().unwrap(),
    };
    store
      .default_insert(&person, UpdateAction::Insert)
      .await
      .unwrap();
  }

  let names = async |filter: PersonFilter| {
    let query = Person::to_filtered_query(&filter).into();
    let dataset = store
      .query(query)
      .await
      .unwrap()
      .get_query_result_dataset()
      .unwrap();

    let mut names: Vec<String> = dataset
      .resource_ids()
      .iter()
      .filter_map(|resource_id| dataset.deserialize_subject_with_resource_id::<Person>(resource_id))
      .map(|person| person.name)
      .collect();
    names.sort();
    names
  };
  let some = |value: &str| Some(value.to_owned());

  assert_eq!(names(PersonFilter::default()).await.len(), 3);

  assert_eq!(
    names(PersonFilter {
      email_domain_suffix: some("example.org"),
      ..Default::default()
    })
    .await,
    ["Charlie Dupont"]
  );
  assert_eq!(
    names(PersonFilter {
      name_contains: some("li"),
      ..Default::default()
    })
    .await,
    ["Alice Martin", "Charlie Dupont"]
  );
  assert_eq!(
    names(PersonFilter {
      email_user: some("bob"),
      ..Default::default()
    })
    .await,
    ["Bob"]
  );
  assert_eq!(
    names(PersonFilter {
      email_domain_not: some("test.net"),
      ..Default::default()
    })
    .await,
    ["Alice Martin", "Charlie Dupont"]
  );
  assert_eq!(
    names(PersonFilter {
      name_min_length: Some(13),
      ..Default::default()
    })
    .await,
    ["Charlie Dupont"]
  );
  assert_eq!(
    names(PersonFilter {
      name_upper: some("BOB"),
      ..Default::default()
    })
    .await,
    ["Bob"]
  );
  assert_eq!(
    names(PersonFilter {
      name_lower_contains: some("dupont"),
      ..Default::default()
    })
    .await,
    ["Charlie Dupont"]
  );
  assert_eq!(
    names(PersonFilter {
      email_domain_lower_starts: some("example"),
      ..Default::default()
    })
    .await,
    ["Alice Martin", "Charlie Dupont"]
  );
  assert_eq!(
    names(PersonFilter {
      born_before_year: Some(1991),
      ..Default::default()
    })
    .await,
    ["Alice Martin", "Bob"]
  );

  // FILTER("1990-01-01T00:00:00Z"^^xsd:dateTime <= ?birth && ?birth < "2002-01-01T00:00:00Z"^^xsd:dateTime)
  assert_eq!(
    names(PersonFilter {
      born_from: some("1990-01-01T00:00:00Z"),
      born_until: some("2002-01-01T00:00:00Z"),
      ..Default::default()
    })
    .await,
    ["Alice Martin", "Charlie Dupont"]
  );
  assert_eq!(
    names(PersonFilter {
      born_until: some("1990-01-01T00:00:00Z"),
      ..Default::default()
    })
    .await,
    ["Bob"]
  );
}

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct Event {
  #[ld(id)]
  id: NamedNode,

  #[ld("ex:name")]
  name: String,

  #[ld("ex:start")]
  start: xsd_types::DateTime,
}

#[derive(SparqlFilter, Default)]
#[ld(prefix("ex" = "http://ex/"))]
struct EventFilter {
  #[ld("ex:start")]
  #[filter(month)]
  month: Option<i32>,

  #[ld("ex:start")]
  #[filter(day)]
  day: Option<i32>,

  #[ld("ex:start")]
  #[filter(hours, >=)]
  from_hour: Option<i32>,

  #[ld("ex:start")]
  #[filter(minutes)]
  minutes: Option<i32>,

  #[ld("ex:start")]
  #[filter(seconds, >)]
  seconds_above: Option<i32>,

  #[ld("ex:start")]
  #[filter(timezone, datatype = "xsd:dayTimeDuration")]
  timezone: Option<String>,

  #[ld("ex:start")]
  #[filter(tz)]
  tz: Option<String>,

  #[ld("ex:start")]
  #[filter(<, now)]
  past: bool,

  #[ld("ex:start")]
  #[filter(>=, now)]
  upcoming: bool,
}

#[tokio::test]
async fn test_filter_date_time_functions() {
  let store = OxigraphSparqlGraphStore::default();

  for (id, start) in [
    ("launch", "2020-03-15T10:30:45Z"),
    ("party", "2021-07-04T18:05:00-05:00"),
    ("future", "2999-12-25T08:00:30+02:00"),
  ] {
    let event = Event {
      id: NamedNode::new(format!("http://ex/event/{id}")).unwrap(),
      name: id.to_owned(),
      start: start.parse().unwrap(),
    };
    store
      .default_insert(&event, UpdateAction::Insert)
      .await
      .unwrap();
  }

  let names = async |filter: EventFilter| {
    let query = Event::to_filtered_query(&filter).into();
    let dataset = store
      .query(query)
      .await
      .unwrap()
      .get_query_result_dataset()
      .unwrap();

    let mut names: Vec<String> = dataset
      .resource_ids()
      .iter()
      .filter_map(|resource_id| dataset.deserialize_subject_with_resource_id::<Event>(resource_id))
      .map(|event| event.name)
      .collect();
    names.sort();
    names
  };

  assert_eq!(
    names(EventFilter {
      month: Some(12),
      ..Default::default()
    })
    .await,
    ["future"]
  );
  assert_eq!(
    names(EventFilter {
      day: Some(4),
      ..Default::default()
    })
    .await,
    ["party"]
  );
  assert_eq!(
    names(EventFilter {
      from_hour: Some(10),
      ..Default::default()
    })
    .await,
    ["launch", "party"]
  );
  assert_eq!(
    names(EventFilter {
      minutes: Some(30),
      ..Default::default()
    })
    .await,
    ["launch"]
  );
  assert_eq!(
    names(EventFilter {
      seconds_above: Some(0),
      ..Default::default()
    })
    .await,
    ["future", "launch"]
  );
  assert_eq!(
    names(EventFilter {
      timezone: Some("-PT5H".to_owned()),
      ..Default::default()
    })
    .await,
    ["party"]
  );
  assert_eq!(
    names(EventFilter {
      tz: Some("Z".to_owned()),
      ..Default::default()
    })
    .await,
    ["launch"]
  );
  assert_eq!(
    names(EventFilter {
      past: true,
      ..Default::default()
    })
    .await,
    ["launch", "party"]
  );
  assert_eq!(
    names(EventFilter {
      upcoming: true,
      ..Default::default()
    })
    .await,
    ["future"]
  );
}
