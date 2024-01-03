use std::{cmp::Ordering, hash};

pub use grouped_ordering_proc_macros::grouped_ordering;

pub trait GroupedOrdering {
    type Group: hash::Hash + Eq;

    fn compare(&self, a: &Self::Group, b: &Self::Group) -> Ordering;
}

pub trait GroupedOrderable<TGroupedOrdering: GroupedOrdering> {
    fn map_to_grouped_ordering(&self) -> TGroupedOrdering::Group;
}

pub trait VecExt {
    type Item;

    fn sort_by_grouped_ordering<TGroupedOrdering>(&mut self, grouped_ordering: &TGroupedOrdering)
    where
        TGroupedOrdering: GroupedOrdering,
        Self::Item: GroupedOrderable<TGroupedOrdering>;
}

impl<T> VecExt for Vec<T> {
    type Item = T;

    fn sort_by_grouped_ordering<TGroupedOrdering>(&mut self, grouped_ordering: &TGroupedOrdering)
    where
        TGroupedOrdering: GroupedOrdering,
        Self::Item: GroupedOrderable<TGroupedOrdering>,
    {
        self.sort_by(|a, b| {
            grouped_ordering.compare(&a.map_to_grouped_ordering(), &b.map_to_grouped_ordering())
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use speculoos::prelude::*;

    use super::*;
    use grouped_ordering_proc_macros::grouped_ordering_crate_internal as grouped_ordering;

    #[test]
    fn test_manual_impl_vec_sort() {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        enum GroupedOrderingFooGroup {
            A,
            B,
            C,
        }

        struct GroupedOrderingFoo {
            #[allow(dead_code)]
            groups: [GroupedOrderingFooGroup; 3],
            index_lookup: HashMap<GroupedOrderingFooGroup, usize>,
        }

        impl GroupedOrdering for GroupedOrderingFoo {
            type Group = GroupedOrderingFooGroup;

            fn compare(
                &self,
                a: &GroupedOrderingFooGroup,
                b: &GroupedOrderingFooGroup,
            ) -> Ordering {
                self.index_lookup[a].cmp(&self.index_lookup[b])
            }
        }

        impl TryFrom<[GroupedOrderingFooGroup; 3]> for GroupedOrderingFoo {
            type Error = String;

            fn try_from(groups: [GroupedOrderingFooGroup; 3]) -> Result<Self, Self::Error> {
                let index_lookup: HashMap<GroupedOrderingFooGroup, usize> = groups
                    .into_iter()
                    .enumerate()
                    .map(|(index, group)| (group, index))
                    .collect();
                if index_lookup.len() < 3 {
                    return Err("Found duplicate groups".to_owned());
                }
                Ok(Self {
                    groups,
                    index_lookup,
                })
            }
        }

        impl GroupedOrderable<GroupedOrderingFoo> for u32 {
            fn map_to_grouped_ordering(&self) -> GroupedOrderingFooGroup {
                match self % 3 {
                    0 => GroupedOrderingFooGroup::A,
                    1 => GroupedOrderingFooGroup::B,
                    2 => GroupedOrderingFooGroup::C,
                    _ => unreachable!(),
                }
            }
        }

        let grouped_ordering: GroupedOrderingFoo = [
            GroupedOrderingFooGroup::A,
            GroupedOrderingFooGroup::B,
            GroupedOrderingFooGroup::C,
        ]
        .try_into()
        .unwrap();

        let mut nums: Vec<u32> = vec![0, 1, 2, 3, 4, 5];
        nums.sort_by_grouped_ordering(&grouped_ordering);
        assert_that!(&nums).is_equal_to(vec![0, 3, 1, 4, 2, 5]);
    }

    #[test]
    fn test_vec_sort() {
        grouped_ordering!(GroupedOrderingFoo, [A, B, C]);

        impl GroupedOrderable<GroupedOrderingFoo> for u32 {
            fn map_to_grouped_ordering(&self) -> GroupedOrderingFooGroup {
                match self % 3 {
                    0 => GroupedOrderingFooGroup::A,
                    1 => GroupedOrderingFooGroup::B,
                    2 => GroupedOrderingFooGroup::C,
                    _ => unreachable!(),
                }
            }
        }

        let grouped_ordering: GroupedOrderingFoo = [
            GroupedOrderingFooGroup::A,
            GroupedOrderingFooGroup::B,
            GroupedOrderingFooGroup::C,
        ]
        .try_into()
        .unwrap();

        let mut nums: Vec<u32> = vec![0, 1, 2, 3, 4, 5];
        nums.sort_by_grouped_ordering(&grouped_ordering);
        assert_that!(&nums).is_equal_to(vec![0, 3, 1, 4, 2, 5]);
    }

    #[test]
    fn test_default() {
        grouped_ordering!(GroupedOrderingFoo, [A, B, C]);

        assert_that!(GroupedOrderingFoo::default().groups).is_equal_to([
            GroupedOrderingFooGroup::A,
            GroupedOrderingFooGroup::B,
            GroupedOrderingFooGroup::C,
        ]);
    }

    #[test]
    fn test_try_into_from_list_with_duplicates_fails() {
        grouped_ordering!(GroupedOrderingFoo, [A, B, C]);

        assert_that!(GroupedOrderingFoo::try_from([
            GroupedOrderingFooGroup::A,
            GroupedOrderingFooGroup::B,
            GroupedOrderingFooGroup::B,
        ]))
        .is_err();
    }

    #[test]
    fn test_deserialize() {
        grouped_ordering!(GroupedOrderingFoo, [A, B, C]);

        let grouped_ordering: GroupedOrderingFoo =
            serde_json::from_str(r#"["b", "a", "c"]"#).unwrap();

        impl GroupedOrderable<GroupedOrderingFoo> for u32 {
            fn map_to_grouped_ordering(&self) -> GroupedOrderingFooGroup {
                match self % 3 {
                    0 => GroupedOrderingFooGroup::A,
                    1 => GroupedOrderingFooGroup::B,
                    2 => GroupedOrderingFooGroup::C,
                    _ => unreachable!(),
                }
            }
        }

        let mut nums: Vec<u32> = vec![0, 1, 2, 3, 4, 5];
        nums.sort_by_grouped_ordering(&grouped_ordering);
        assert_that!(&nums).is_equal_to(vec![1, 4, 0, 3, 2, 5]);
    }

    #[test]
    fn test_deserialize_from_incomplete_list_fails() {
        grouped_ordering!(GroupedOrderingFoo, [A, B, C]);

        assert_that!(&serde_json::from_str::<GroupedOrderingFoo>(r#"["a", "b"]"#)).is_err();
    }

    #[test]
    fn test_deserialize_from_list_with_duplicates_fails() {
        grouped_ordering!(GroupedOrderingFoo, [A, B, C]);

        assert_that!(&serde_json::from_str::<GroupedOrderingFoo>(
            r#"["a", "b", "b"]"#
        ))
        .is_err();
    }

    #[test]
    fn test_instantiate_macro() {
        grouped_ordering!(GroupedOrderingFoo, [A, B, C]);

        impl GroupedOrderable<GroupedOrderingFoo> for u32 {
            fn map_to_grouped_ordering(&self) -> GroupedOrderingFooGroup {
                match self % 3 {
                    0 => GroupedOrderingFooGroup::A,
                    1 => GroupedOrderingFooGroup::B,
                    2 => GroupedOrderingFooGroup::C,
                    _ => unreachable!(),
                }
            }
        }

        let grouped_ordering = grouped_ordering_foo![A, B, C];

        let mut nums: Vec<u32> = vec![0, 1, 2, 3, 4, 5];
        nums.sort_by_grouped_ordering(&grouped_ordering);
        assert_that!(&nums).is_equal_to(vec![0, 3, 1, 4, 2, 5]);
    }

    #[test]
    fn test_index_by_group() {
        grouped_ordering!(GroupedOrderingFoo, [A, B, C]);

        let grouped_ordering = grouped_ordering_foo![C, A, B];

        assert_that!(&grouped_ordering[GroupedOrderingFooGroup::A]).is_equal_to(1);
    }

    #[test]
    fn test_index_by_usize() {
        grouped_ordering!(GroupedOrderingFoo, [A, B, C]);

        let grouped_ordering = grouped_ordering_foo![C, A, B];

        assert_that!(&grouped_ordering[0]).is_equal_to(GroupedOrderingFooGroup::C);
    }
}
