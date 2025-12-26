use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::atomic::{AtomicI64, Ordering},
};

use doublets::{
    data::{Error, Flow, LinkType, LinksConstants, ReadHandler, WriteHandler},
    Doublets, Link, Links,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{Exclusive, Result, Sql};

/// Neo4j HTTP API client using raw TCP
pub struct Client<T: LinkType> {
    host: String,
    port: u16,
    auth: String,
    constants: LinksConstants<T>,
    next_id: AtomicI64,
}

#[derive(Debug, Serialize)]
struct CypherRequest {
    statements: Vec<Statement>,
}

#[derive(Debug, Serialize)]
struct Statement {
    statement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Value>,
}

/// Response from Neo4j Cypher query
#[derive(Debug, Deserialize)]
pub struct CypherResponse {
    pub results: Vec<QueryResult>,
    #[serde(default)]
    pub errors: Vec<CypherError>,
}

/// Result from a single Cypher query statement
#[derive(Debug, Deserialize)]
pub struct QueryResult {
    #[serde(default)]
    #[allow(dead_code)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub data: Vec<RowData>,
}

/// A single row of data from a Cypher query
#[derive(Debug, Deserialize)]
pub struct RowData {
    pub row: Vec<Value>,
}

/// Error returned from Neo4j
#[derive(Debug, Deserialize)]
pub struct CypherError {
    #[allow(dead_code)]
    pub code: String,
    #[allow(dead_code)]
    pub message: String,
}

impl<T: LinkType> Client<T> {
    /// Get the host for this client
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Get the port for this client
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the auth header for this client
    pub fn auth(&self) -> &str {
        &self.auth
    }

    /// Get the constants for this client
    pub fn constants(&self) -> &LinksConstants<T> {
        &self.constants
    }

    /// Get and increment the next_id atomically
    pub fn fetch_next_id(&self) -> i64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Reset the next_id counter to 1 (used for benchmark isolation)
    pub fn reset_next_id(&self) {
        self.next_id.store(1, Ordering::SeqCst);
    }

    pub fn new(uri: &str, user: &str, password: &str) -> Result<Self> {
        // Parse URI to extract host and port
        let uri = uri.replace("bolt://", "").replace("http://", "");
        let parts: Vec<&str> = uri.split(':').collect();
        let host = parts.get(0).unwrap_or(&"localhost").to_string();
        let bolt_port: u16 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(7687);
        // Convert bolt port to HTTP port
        let port = if bolt_port == 7687 { 7474 } else { bolt_port };

        let auth = format!("Basic {}", base64_encode(&format!("{}:{}", user, password)));

        let client = Self {
            host,
            port,
            auth,
            constants: LinksConstants::new(),
            next_id: AtomicI64::new(1),
        };

        // Create indexes (ignore errors if already exist)
        let _ = client.execute_cypher(
            "CREATE CONSTRAINT link_id IF NOT EXISTS FOR (l:Link) REQUIRE l.id IS UNIQUE",
            None,
        );
        let _ = client.execute_cypher(
            "CREATE INDEX link_source IF NOT EXISTS FOR (l:Link) ON (l.source)",
            None,
        );
        let _ = client.execute_cypher(
            "CREATE INDEX link_target IF NOT EXISTS FOR (l:Link) ON (l.target)",
            None,
        );

        // Initialize next_id from database
        if let Ok(response) = client.execute_cypher(
            "MATCH (l:Link) RETURN COALESCE(max(l.id), 0) as max_id",
            None,
        ) {
            if let Some(result) = response.results.first() {
                if let Some(row) = result.data.first() {
                    if let Some(val) = row.row.first() {
                        let max_id = val.as_i64().unwrap_or(0);
                        client.next_id.store(max_id + 1, Ordering::SeqCst);
                    }
                }
            }
        }

        Ok(client)
    }

    /// Execute a Cypher query against Neo4j
    pub fn execute_cypher(&self, query: &str, params: Option<Value>) -> Result<CypherResponse> {
        let request = CypherRequest {
            statements: vec![Statement {
                statement: query.to_string(),
                parameters: params,
            }],
        };

        let body = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        let path = "/db/neo4j/tx/commit";

        let http_request = format!(
            "POST {} HTTP/1.1\r\n\
            Host: {}:{}\r\n\
            Authorization: {}\r\n\
            Content-Type: application/json\r\n\
            Accept: application/json\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\
            \r\n\
            {}",
            path,
            self.host,
            self.port,
            self.auth,
            body.len(),
            body
        );

        let mut stream =
            TcpStream::connect((&self.host[..], self.port)).map_err(|e| e.to_string())?;

        stream
            .write_all(http_request.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| e.to_string())?;

        // Parse HTTP response - find body after empty line
        let body_start = response.find("\r\n\r\n").ok_or("Invalid HTTP response")?;
        let body = &response[body_start + 4..];

        // Handle chunked encoding if present
        let json_body = if response.contains("Transfer-Encoding: chunked") {
            // Simple chunked decoding - find the JSON object
            if let Some(start) = body.find('{') {
                if let Some(end) = body.rfind('}') {
                    &body[start..=end]
                } else {
                    body
                }
            } else {
                body
            }
        } else {
            body
        };

        let cypher_response: CypherResponse = serde_json::from_str(json_body)
            .map_err(|e| format!("JSON parse error: {} in body: {}", e, json_body))?;

        if !cypher_response.errors.is_empty() {
            return Err(cypher_response.errors[0].message.clone().into());
        }

        Ok(cypher_response)
    }
}

fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).map(|&b| b as u32).unwrap_or(0);
        let b2 = chunk.get(2).map(|&b| b as u32).unwrap_or(0);

        let combined = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[((combined >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((combined >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((combined >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[(combined & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

impl<T: LinkType> Sql for Client<T> {
    fn create_table(&mut self) -> Result<()> {
        let _ = self.execute_cypher(
            "CREATE CONSTRAINT link_id IF NOT EXISTS FOR (l:Link) REQUIRE l.id IS UNIQUE",
            None,
        );
        Ok(())
    }

    fn drop_table(&mut self) -> Result<()> {
        let _ = self.execute_cypher("MATCH (l:Link) DETACH DELETE l", None);
        self.next_id.store(1, Ordering::SeqCst);
        Ok(())
    }
}

impl<T: LinkType> Links<T> for Exclusive<Client<T>> {
    fn constants(&self) -> &LinksConstants<T> {
        &self.constants
    }

    fn count_links(&self, query: &[T]) -> T {
        let any = self.constants.any;

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

        match self.get().execute_cypher(&cypher, None) {
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
        let next_id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let _ = self.execute_cypher(
            "CREATE (l:Link {id: $id, source: 0, target: 0})",
            Some(json!({ "id": next_id })),
        );

        Ok(handler(
            Link::nothing(),
            Link::new(next_id.try_into().unwrap_or(T::ZERO), T::ZERO, T::ZERO),
        ))
    }

    fn each_links(&self, query: &[T], handler: ReadHandler<T>) -> Flow {
        let any = self.constants.any;

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

        match self.get().execute_cypher(&cypher, None) {
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
        let old_result = self.execute_cypher(
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
        let _ = self.execute_cypher(
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
        let old_result = self.execute_cypher(
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
        let _ = self.execute_cypher(
            "MATCH (l:Link {id: $id}) DELETE l",
            Some(json!({"id": id.as_i64()})),
        );

        Ok(handler(
            Link::new(id, old_source, old_target),
            Link::nothing(),
        ))
    }
}

impl<T: LinkType> Doublets<T> for Exclusive<Client<T>> {
    fn get_link(&self, index: T) -> Option<Link<T>> {
        match self.get().execute_cypher(
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
