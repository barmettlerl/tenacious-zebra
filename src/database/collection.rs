use crate::{
    common::store::{Field, EmptyField},
    database::{CollectionResponse, CollectionSender, CollectionTransaction, Table},
};

use std::{collections::HashSet, hash::Hash as StdHash, sync::Arc, fmt::Display};

use talk::crypto::primitives::hash::Hash;


pub struct Collection<Item: Field + Display>(pub(crate) Arc<Table<Item, EmptyField>>);

impl<Item> Collection<Item>
where
    Item: Field + Display,
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
        lho: &mut Collection<Item>,
        rho: &mut Collection<Item>,
    ) -> (HashSet<Item>, HashSet<Item>)
    where
        Item: Clone + Eq + StdHash ,
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
    Item: Field + Display,
{
    fn clone(&self) -> Self {
        Collection(self.0.clone())
    }
}
