// Transaction wraps Client and delegates all operations to it.
// In the HTTP-based approach using /db/neo4j/tx/commit endpoint,
// all requests are auto-committed transactions.
//
// This wrapper exists for API compatibility to benchmark "transactional"
// Neo4j operations, which in this implementation are semantically
// equivalent to non-transactional operations.

use doublets::{
    data::{Error, Flow, LinkType, LinksConstants, ReadHandler, WriteHandler},
    Doublets, Link, Links,
};
use serde_json::json;

use crate::{Client, Exclusive, Result, Sql};

pub struct Transaction<'a, T: LinkType> {
    client: &'a Client<T>,
}

impl<'a, T: LinkType> Transaction<'a, T> {
    pub fn new(client: &'a Client<T>) -> Result<Self> {
        Ok(Self { client })
    }
}

impl<T: LinkType> Sql for Transaction<'_, T> {
    fn create_table(&mut self) -> Result<()> {
        // Already created by client during initialization
        Ok(())
    }

    fn drop_table(&mut self) -> Result<()> {
        // Delete all nodes - delegated to client
        let _ = self
            .client
            .execute_cypher("MATCH (l:Link) DETACH DELETE l", None);
        // Reset the ID counter to ensure isolation between benchmark iterations
        self.client.reset_next_id();
        Ok(())
    }
}

// Transaction delegates all Links operations to the underlying Client
impl<'a, T: LinkType> Links<T> for Exclusive<Transaction<'a, T>> {
    fn constants(&self) -> &LinksConstants<T> {
        self.client.constants()
    }

    fn count_links(&self, query: &[T]) -> T {
        let any = self.constants().any;

        let cypher = if query.is_empty() {
            "MATCH (l:Link) RETURN count(l) as count".to_string()
        } else if query.len() == 1 {
            if query[0] == any {
                "MATCH (l:Link) RETURN count(l) as count".to_string()
            } else {
                format!(
                    "MATCH (l:Link {{id: {}}}) RETURN count(l) as count",
                    query[0]
                )
            }
        } else if query.len() == 3 {
            let mut conditions = Vec::new();

            if query[0] != any {
                conditions.push(format!("l.id = {}", query[0]));
            }
            if query[1] != any {
                conditions.push(format!("l.source = {}", query[1]));
            }
            if query[2] != any {
                conditions.push(format!("l.target = {}", query[2]));
            }

            if conditions.is_empty() {
                "MATCH (l:Link) RETURN count(l) as count".to_string()
            } else {
                format!(
                    "MATCH (l:Link) WHERE {} RETURN count(l) as count",
                    conditions.join(" AND ")
                )
            }
        } else {
            panic!("Constraints violation: size of query neither 1 nor 3")
        };

        match self.client.execute_cypher(&cypher, None) {
            Ok(response) => {
                if let Some(result) = response.results.first() {
                    if let Some(row) = result.data.first() {
                        if let Some(val) = row.row.first() {
                            let count = val.as_i64().unwrap_or(0);
                            return count.try_into().unwrap_or(T::ZERO);
                        }
                    }
                }
                T::ZERO
            }
            Err(_) => T::ZERO,
        }
    }

    fn create_links(
        &mut self,
        _query: &[T],
        handler: WriteHandler<T>,
    ) -> std::result::Result<Flow, Error<T>> {
        let next_id = self.client.fetch_next_id();

        let _ = self.client.execute_cypher(
            "CREATE (l:Link {id: $id, source: 0, target: 0})",
            Some(json!({ "id": next_id })),
        );

        Ok(handler(
            Link::nothing(),
            Link::new(next_id.try_into().unwrap_or(T::ZERO), T::ZERO, T::ZERO),
        ))
    }

    fn each_links(&self, query: &[T], handler: ReadHandler<T>) -> Flow {
        let any = self.constants().any;

        let cypher = if query.is_empty() {
            "MATCH (l:Link) RETURN l.id as id, l.source as source, l.target as target".to_string()
        } else if query.len() == 1 {
            if query[0] == any {
                "MATCH (l:Link) RETURN l.id as id, l.source as source, l.target as target"
                    .to_string()
            } else {
                format!(
                    "MATCH (l:Link {{id: {}}}) RETURN l.id as id, l.source as source, l.target as target",
                    query[0]
                )
            }
        } else if query.len() == 3 {
            let mut conditions = Vec::new();

            if query[0] != any {
                conditions.push(format!("l.id = {}", query[0]));
            }
            if query[1] != any {
                conditions.push(format!("l.source = {}", query[1]));
            }
            if query[2] != any {
                conditions.push(format!("l.target = {}", query[2]));
            }

            if conditions.is_empty() {
                "MATCH (l:Link) RETURN l.id as id, l.source as source, l.target as target"
                    .to_string()
            } else {
                format!(
                    "MATCH (l:Link) WHERE {} RETURN l.id as id, l.source as source, l.target as target",
                    conditions.join(" AND ")
                )
            }
        } else {
            panic!("Constraints violation: size of query neither 1 nor 3")
        };

        match self.client.execute_cypher(&cypher, None) {
            Ok(response) => {
                if let Some(result) = response.results.first() {
                    for row in &result.data {
                        if row.row.len() >= 3 {
                            let id = row.row[0].as_i64().unwrap_or(0);
                            let source = row.row[1].as_i64().unwrap_or(0);
                            let target = row.row[2].as_i64().unwrap_or(0);

                            if let Flow::Break = handler(Link::new(
                                id.try_into().unwrap_or(T::ZERO),
                                source.try_into().unwrap_or(T::ZERO),
                                target.try_into().unwrap_or(T::ZERO),
                            )) {
                                return Flow::Break;
                            }
                        }
                    }
                }
                Flow::Continue
            }
            Err(_) => Flow::Continue,
        }
    }

    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<T>,
    ) -> std::result::Result<Flow, Error<T>> {
        let id = query[0];
        let source = change[1];
        let target = change[2];

        // Get old values
        let old_result = self.client.execute_cypher(
            "MATCH (l:Link {id: $id}) RETURN l.source as source, l.target as target",
            Some(json!({"id": id.as_i64()})),
        );

        let (old_source, old_target) = match old_result {
            Ok(response) => {
                if let Some(result) = response.results.first() {
                    if let Some(row) = result.data.first() {
                        if row.row.len() >= 2 {
                            let s = row.row[0].as_i64().unwrap_or(0);
                            let t = row.row[1].as_i64().unwrap_or(0);
                            (
                                s.try_into().unwrap_or(T::ZERO),
                                t.try_into().unwrap_or(T::ZERO),
                            )
                        } else {
                            (T::ZERO, T::ZERO)
                        }
                    } else {
                        (T::ZERO, T::ZERO)
                    }
                } else {
                    (T::ZERO, T::ZERO)
                }
            }
            Err(_) => (T::ZERO, T::ZERO),
        };

        // Update
        let _ = self.client.execute_cypher(
            "MATCH (l:Link {id: $id}) SET l.source = $source, l.target = $target",
            Some(json!({
                "id": id.as_i64(),
                "source": source.as_i64(),
                "target": target.as_i64()
            })),
        );

        Ok(handler(
            Link::new(id, old_source, old_target),
            Link::new(id, source, target),
        ))
    }

    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<T>,
    ) -> std::result::Result<Flow, Error<T>> {
        let id = query[0];

        // Get old values before deleting
        let old_result = self.client.execute_cypher(
            "MATCH (l:Link {id: $id}) RETURN l.source as source, l.target as target",
            Some(json!({"id": id.as_i64()})),
        );

        let (old_source, old_target) = match old_result {
            Ok(response) => {
                if let Some(result) = response.results.first() {
                    if let Some(row) = result.data.first() {
                        if row.row.len() >= 2 {
                            let s = row.row[0].as_i64().unwrap_or(0);
                            let t = row.row[1].as_i64().unwrap_or(0);
                            (
                                s.try_into().unwrap_or(T::ZERO),
                                t.try_into().unwrap_or(T::ZERO),
                            )
                        } else {
                            return Err(Error::<T>::NotExists(id));
                        }
                    } else {
                        return Err(Error::<T>::NotExists(id));
                    }
                } else {
                    return Err(Error::<T>::NotExists(id));
                }
            }
            Err(_) => return Err(Error::<T>::NotExists(id)),
        };

        // Delete
        let _ = self.client.execute_cypher(
            "MATCH (l:Link {id: $id}) DELETE l",
            Some(json!({"id": id.as_i64()})),
        );

        Ok(handler(
            Link::new(id, old_source, old_target),
            Link::nothing(),
        ))
    }
}

impl<'a, T: LinkType> Doublets<T> for Exclusive<Transaction<'a, T>> {
    fn get_link(&self, index: T) -> Option<Link<T>> {
        match self.client.execute_cypher(
            "MATCH (l:Link {id: $id}) RETURN l.source as source, l.target as target",
            Some(json!({"id": index.as_i64()})),
        ) {
            Ok(response) => {
                if let Some(result) = response.results.first() {
                    if let Some(row) = result.data.first() {
                        if row.row.len() >= 2 {
                            let source = row.row[0].as_i64().unwrap_or(0);
                            let target = row.row[1].as_i64().unwrap_or(0);
                            return Some(Link::new(
                                index,
                                source.try_into().unwrap_or(T::ZERO),
                                target.try_into().unwrap_or(T::ZERO),
                            ));
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    }
}
