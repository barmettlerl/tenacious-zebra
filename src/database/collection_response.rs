use crate::{
    common::store::{Field, EmptyField},
    database::{Query, TableResponse},
};

pub struct CollectionResponse<Item: Field>(pub(crate) TableResponse<Item, EmptyField>);

impl<Item> CollectionResponse<Item>
where
    Item: Field,
{
    pub fn contains(&self, query: &Query) -> bool {
        self.0.get(&query).is_some()
    }
}
