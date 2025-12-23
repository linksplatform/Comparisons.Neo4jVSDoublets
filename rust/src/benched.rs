use {
    crate::{map_file, Client, Exclusive, Fork, Result, Sql, Transaction},
    doublets::{
        data::LinkType,
        mem::{Alloc, FileMapped},
        split::{self, DataPart, IndexPart},
        unit::{self, LinkPart},
        Doublets,
    },
    std::alloc::Global,
};

pub trait Benched: Sized {
    type Builder<'params>;

    fn setup<'a>(builder: Self::Builder<'a>) -> Result<Self>;

    fn fork(&mut self) -> Fork<Self> {
        Fork(self)
    }

    unsafe fn unfork(&mut self);
}

// Doublets implementations
impl<T: LinkType> Benched for unit::Store<T, FileMapped<LinkPart<T>>> {
    type Builder<'a> = &'a str;

    fn setup(builder: Self::Builder<'_>) -> Result<Self> {
        Self::new(map_file(builder)?).map_err(Into::into)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.delete_all();
    }
}

impl<T: LinkType> Benched for unit::Store<T, Alloc<LinkPart<T>, Global>> {
    type Builder<'a> = ();

    fn setup(_: Self::Builder<'_>) -> Result<Self> {
        Self::new(Alloc::new(Global)).map_err(Into::into)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.delete_all();
    }
}

impl<T: LinkType> Benched for split::Store<T, FileMapped<DataPart<T>>, FileMapped<IndexPart<T>>> {
    type Builder<'a> = (&'a str, &'a str);

    fn setup((data, index): Self::Builder<'_>) -> Result<Self> {
        Self::new(map_file(data)?, map_file(index)?).map_err(Into::into)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.delete_all();
    }
}

impl<T: LinkType> Benched
    for split::Store<T, Alloc<DataPart<T>, Global>, Alloc<IndexPart<T>, Global>>
{
    type Builder<'a> = ();

    fn setup(_: Self::Builder<'_>) -> Result<Self> {
        Self::new(Alloc::new(Global), Alloc::new(Global)).map_err(Into::into)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.delete_all();
    }
}

// Neo4j implementations
impl<T: LinkType> Benched for Exclusive<Client<T>> {
    type Builder<'a> = ();

    fn setup(_: Self::Builder<'_>) -> Result<Self> {
        unsafe { Ok(Exclusive::new(crate::connect()?)) }
    }

    fn fork(&mut self) -> Fork<Self> {
        let _ = self.create_table();
        Fork(self)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.drop_table();
    }
}

impl<'a, T: LinkType> Benched for Exclusive<Transaction<'a, T>> {
    type Builder<'b> = &'a Client<T>;

    fn setup(builder: Self::Builder<'_>) -> Result<Self> {
        let transaction = Transaction::new(builder)?;
        unsafe { Ok(Exclusive::new(transaction)) }
    }

    fn fork(&mut self) -> Fork<Self> {
        // Clean up any existing data before benchmark to ensure isolation
        let _ = self.drop_table();
        Fork(self)
    }

    unsafe fn unfork(&mut self) {
        // Clean up after benchmark iteration
        let _ = self.drop_table();
    }
}
