pub trait LeftJoin {
  fn left_join(self, other: Self) -> Self;
}
