//! Compile-time check of which `neo4j` crate types are `Send`/`Sync`.
fn send<T: Send>() {}
fn sync<T: Sync>() {}
fn main() {
    send::<neo4j::driver::Driver>();
    sync::<neo4j::driver::Driver>();
    send::<neo4j::session::Session<'static>>();
    sync::<neo4j::session::Session<'static>>();
    send::<neo4j::transaction::Transaction<'static, 'static>>();
    sync::<neo4j::transaction::Transaction<'static, 'static>>();
}
