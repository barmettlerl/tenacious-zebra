use crate::{
    common::store::Field,
    database::{CollectionResponse, CollectionSender, CollectionTransaction, Table},
};

use std::{collections::HashSet, hash::Hash as StdHash, sync::Arc};

use talk::crypto::primitives::hash::Hash;

pub struct Collection<Item: Field>(pub(crate) Arc<Table<Item, ()>>);

impl<Item> Collection<Item>
where
    Item: Field,
{
    pub fn commit(&self) -> Hash {
        self.0.commit()
    }

    pub fn execute(
        &mut self,
        transaction: CollectionTransaction<Item>,
    ) -> CollectionResponse<Item> {
        CollectionResponse(self.0.execute(transaction.0))
    }

    pub fn send(self) -> CollectionSender<Item> {
        CollectionSender(self.0.send())
    }

    pub fn diff(
        lho: &Collection<Item>,
        rho: &Collection<Item>,
    ) -> (HashSet<Item>, HashSet<Item>)
    where
        Item: Clone + Eq + StdHash,
    {
        let mut lho_minus_rho = HashSet::new();
        let mut rho_minus_lho = HashSet::new();

        for (key, (in_lho, _)) in Table::diff(&lho.0, &rho.0) {
            if in_lho.is_some() {
                lho_minus_rho.insert(key);
            } else {
                rho_minus_lho.insert(key);
            }
        }

        (lho_minus_rho, rho_minus_lho)
    }
}

impl<Item> Clone for Collection<Item>
where
    Item: Field,
{
    fn clone(&self) -> Self {
        Collection(self.0.clone())
    }
}
