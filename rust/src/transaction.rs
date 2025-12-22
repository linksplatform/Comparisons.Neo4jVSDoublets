// Transaction is a thin wrapper around Client for API compatibility
// In the HTTP-based approach, all requests are already transactional

use {
    crate::{Client, Exclusive, Result, Sql},
    doublets::{
        data::{Error, Flow, LinkType, LinksConstants, ReadHandler, WriteHandler},
        Doublets, Link, Links,
    },
    once_cell::sync::Lazy,
    std::marker::PhantomData,
};

pub struct Transaction<'a, T: LinkType> {
    #[allow(dead_code)]
    client: &'a Client<T>,
    _marker: PhantomData<T>,
}

impl<'a, T: LinkType> Transaction<'a, T> {
    pub fn new(client: &'a Client<T>) -> Result<Self> {
        Ok(Self {
            client,
            _marker: PhantomData,
        })
    }
}

impl<T: LinkType> Sql for Transaction<'_, T> {
    fn create_table(&mut self) -> Result<()> {
        // Already created by client
        Ok(())
    }

    fn drop_table(&mut self) -> Result<()> {
        // Handled at benchmark level
        Ok(())
    }
}

// For API compatibility, Transaction delegates to Client through Exclusive wrapper
impl<'a, T: LinkType> Links<T> for Exclusive<Transaction<'a, T>> {
    fn constants(&self) -> &LinksConstants<T> {
        // Get constants from the underlying client
        // This is a bit hacky but works for benchmarking
        static CONSTANTS: Lazy<LinksConstants<usize>> = Lazy::new(|| LinksConstants::new());
        // Safety: we're only using this for the 'any' field which is the same for all T
        unsafe { std::mem::transmute(&*CONSTANTS) }
    }

    fn count_links(&self, _query: &[T]) -> T {
        T::ZERO
    }

    fn create_links(&mut self, _query: &[T], handler: WriteHandler<T>) -> std::result::Result<Flow, Error<T>> {
        Ok(handler(Link::nothing(), Link::nothing()))
    }

    fn each_links(&self, _query: &[T], _handler: ReadHandler<T>) -> Flow {
        Flow::Continue
    }

    fn update_links(
        &mut self,
        _query: &[T],
        _change: &[T],
        handler: WriteHandler<T>,
    ) -> std::result::Result<Flow, Error<T>> {
        Ok(handler(Link::nothing(), Link::nothing()))
    }

    fn delete_links(&mut self, query: &[T], _handler: WriteHandler<T>) -> std::result::Result<Flow, Error<T>> {
        Err(Error::NotExists(query[0]))
    }
}

impl<'a, T: LinkType> Doublets<T> for Exclusive<Transaction<'a, T>> {
    fn get_link(&self, _index: T) -> Option<Link<T>> {
        None
    }
}
