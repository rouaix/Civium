//! Serveur MCP (Model Context Protocol) intégré.
//!
//! Expose les données Civium en lecture seule via JSON-RPC 2.0 sur HTTP.
//! Chaque requête doit présenter un jeton Bearer.
//! Le CIL est appliqué à chaque accès à une ressource.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use civium_core::{
    cil::{CilAction, check_cil},
    NetworkKind,
    GroupKey,
    plugin::{PluginManifest, PluginPermission, PluginRecord, PluginState},
};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::store;

// ── MCP plugin record (read-only, all read permissions) ───────────────────────

fn mcp_plugin() -> PluginRecord {
    PluginRecord {
        state: PluginState::Enabled,
        installed_at: 0,
        manifest: PluginManifest {
            id: "mcp.client".to_string(),
            name: "Client MCP".to_string(),
            version: "1.0.0".to_string(),
            description: "Accès lecture via le protocole MCP (Model Context Protocol)".to_string(),
            author: "External".to_string(),
            is_system: false,
            permissions: vec![
                PluginPermission::ReadMembers,
                PluginPermission::ReadMessages,
                PluginPermission::ReadGovernance,
                PluginPermission::ReadDirectory,
                PluginPermission::ReadAgenda,
                PluginPermission::ReadDocuments,
                PluginPermission::ReadConnections,
            ],
        },
    }
}

// ── Token generation ──────────────────────────────────────────────────────────

pub fn generate_token() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut h);
    std::thread::current().id().hash(&mut h);
    let a = h.finish();
    a.hash(&mut h);
    let b = h.finish();
    format!("{a:016x}{b:016x}")
}

// ── Server entry point ────────────────────────────────────────────────────────

/// Run the MCP HTTP server until the shutdown signal fires.
pub async fn run_mcp_server(
    data_dir: PathBuf,
    port: u16,
    token: String,
    mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let listener = match TcpListener::bind(format!("127.0.0.1:{port}")).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[MCP] Impossible d'écouter sur le port {port}: {e}");
            return;
        }
    };

    loop {
        tokio::select! {
            res = listener.accept() => {
                match res {
                    Ok((stream, _)) => {
                        let dd = data_dir.clone();
                        let t = token.clone();
                        tokio::spawn(async move { handle_conn(stream, dd, t).await; });
                    }
                    Err(e) => eprintln!("[MCP] Erreur accept: {e}"),
                }
            }
            _ = &mut shutdown_rx => break,
        }
    }
}

// ── Connection handler ────────────────────────────────────────────────────────

async fn handle_conn(mut stream: TcpStream, data_dir: PathBuf, token: String) {
    let Some((headers, body)) = read_request(&mut stream).await else { return };

    // CORS preflight
    if headers.get("method").map(|s| s.as_str()) == Some("options") {
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\n\
              Access-Control-Allow-Origin: *\r\n\
              Access-Control-Allow-Headers: Authorization,Content-Type\r\n\
              Access-Control-Allow-Methods: POST,OPTIONS\r\n\
              Content-Length: 0\r\n\r\n",
        ).await;
        return;
    }

    // Bearer token authentication
    let auth = headers.get("authorization").map(|s| s.as_str()).unwrap_or("");
    if auth != format!("Bearer {token}") {
        let body = r#"{"error":"non autorisé — jeton Bearer invalide ou manquant"}"#;
        let resp = format!(
            "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes()).await;
        return;
    }

    // Parse JSON-RPC 2.0
    let req: Value = match serde_json::from_str(body.trim()) {
        Ok(v) => v,
        Err(_) => {
            let r = json!({"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null});
            write_json(&mut stream, &r).await;
            return;
        }
    };

    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);

    let response = match dispatch(method, params, &data_dir).await {
        Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
        Err(msg)   => json!({"jsonrpc":"2.0","id":id,"error":{"code":-32603,"message":msg}}),
    };
    write_json(&mut stream, &response).await;
}

// ── JSON-RPC dispatch ─────────────────────────────────────────────────────────

async fn dispatch(method: &str, params: Value, data_dir: &PathBuf) -> Result<Value, String> {
    match method {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "resources": { "subscribe": false, "listChanged": false }
            },
            "serverInfo": { "name": "Civium MCP", "version": "0.1.0" },
            "instructions": "Accès en lecture seule aux données d'un nœud Civium. \
                             Utilisez resources/list pour découvrir les ressources disponibles, \
                             resources/read pour lire une ressource par URI."
        })),

        "resources/list" => {
            check_cil(&mcp_plugin(), &CilAction::QueryMembers).map_err(|e| e)?;
            let conn = store::open_db(data_dir).map_err(|e| e.to_string())?;
            let networks = store::list_networks(&conn).map_err(|e| e.to_string())?;
            let mut resources = vec![json!({
                "uri": "civium://networks",
                "name": "Réseaux",
                "description": "Liste de tous les réseaux Civium sur ce nœud",
                "mimeType": "application/json"
            })];
            for net in &networks {
                let cid = net.cid_short();
                let name = net.name();
                let base = format!("civium://network/{cid}");
                resources.push(json!({"uri":format!("{base}/members"),"name":format!("{name} — Membres"),"mimeType":"application/json"}));
                resources.push(json!({"uri":format!("{base}/messages"),"name":format!("{name} — Messages"),"mimeType":"application/json"}));
                resources.push(json!({"uri":format!("{base}/proposals"),"name":format!("{name} — Propositions"),"mimeType":"application/json"}));
                resources.push(json!({"uri":format!("{base}/agenda"),"name":format!("{name} — Agenda"),"mimeType":"application/json"}));
                resources.push(json!({"uri":format!("{base}/documents"),"name":format!("{name} — Documents"),"mimeType":"application/json"}));
            }
            Ok(json!({"resources": resources}))
        }

        "resources/read" => {
            let uri = params.get("uri")
                .and_then(|u| u.as_str())
                .ok_or_else(|| "paramètre 'uri' manquant".to_string())?;
            let text = read_resource(uri, data_dir).await?;
            Ok(json!({"contents":[{"uri":uri,"mimeType":"application/json","text":text}]}))
        }

        _ => Err(format!("Méthode MCP inconnue : '{method}'")),
    }
}

// ── Resource readers ──────────────────────────────────────────────────────────

async fn read_resource(uri: &str, data_dir: &PathBuf) -> Result<String, String> {
    let conn = store::open_db(data_dir).map_err(|e| e.to_string())?;

    if uri == "civium://networks" {
        check_cil(&mcp_plugin(), &CilAction::QueryMembers).map_err(|e| e)?;
        let networks = store::list_networks(&conn).map_err(|e| e.to_string())?;
        let result: Vec<Value> = networks.iter().map(|n| json!({
            "cid_short": n.cid_short(),
            "name": n.name(),
            "member_count": n.data.members.len(),
            "kind": match n.data.kind {
                NetworkKind::Directory => "annuaire",
                NetworkKind::Rrm      => "rrm",
                _                     => "standard",
            },
        })).collect();
        return serde_json::to_string_pretty(&result).map_err(|e| e.to_string());
    }

    // civium://network/{cid}/{resource}
    if let Some(rest) = uri.strip_prefix("civium://network/") {
        let (cid, resource) = rest.split_once('/')
            .ok_or_else(|| format!("URI invalide : {uri}"))?;
        let net = store::load_network(&conn, cid)
            .map_err(|_| format!("réseau '{cid}' introuvable"))?;

        return match resource {
            "members" => {
                check_cil(&mcp_plugin(), &CilAction::QueryMembers).map_err(|e| e)?;
                let members: Vec<Value> = net.data.members.iter().map(|m| json!({
                    "cid_short": m.cid_short,
                    "display_name": m.display_name,
                    "circle": m.circle as u8,
                    "role": format!("{:?}", m.role),
                    "is_minor": m.is_minor,
                })).collect();
                serde_json::to_string_pretty(&members).map_err(|e| e.to_string())
            }

            "messages" => {
                check_cil(&mcp_plugin(), &CilAction::QueryMessages).map_err(|e| e)?;
                let group_key = GroupKey::from_b58(&net.data.group_key_b58)
                    .map_err(|e| e.to_string())?;
                let messages = store::load_messages(&conn, cid).map_err(|e| e.to_string())?;
                let member_names: HashMap<String, String> = net.data.members.iter()
                    .map(|m| (m.cid_short.clone(), m.display_name.clone()))
                    .collect();
                let result: Vec<Value> = messages.iter().rev().take(100).rev().map(|msg| {
                    let body = group_key.decrypt(&msg.nonce_b58, &msg.ciphertext_b58)
                        .map(|b| String::from_utf8_lossy(&b).into_owned())
                        .unwrap_or_else(|_| "[illisible]".into());
                    let author = member_names.get(&msg.author_cid_short)
                        .cloned().unwrap_or_else(|| msg.author_cid_short.clone());
                    json!({"id":msg.id,"author":author,"body":body,"sent_at":msg.sent_at})
                }).collect();
                serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
            }

            "proposals" => {
                check_cil(&mcp_plugin(), &CilAction::QueryProposals).map_err(|e| e)?;
                let proposals = store::list_proposals(&conn, cid).map_err(|e| e.to_string())?;
                let result: Vec<Value> = proposals.iter().map(|p| json!({
                    "id": p.id,
                    "title": p.title,
                    "description": p.description,
                    "options": p.options,
                    "status": format!("{:?}", p.status),
                    "created_by": p.created_by,
                    "created_at": p.created_at,
                    "closes_at": p.closes_at,
                    "quorum_percent": p.quorum_percent,
                })).collect();
                serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
            }

            "agenda" => {
                check_cil(&mcp_plugin(), &CilAction::QueryAgenda).map_err(|e| e)?;
                let events = store::list_agenda_events(&conn, cid).map_err(|e| e.to_string())?;
                let result: Vec<Value> = events.iter().map(|e| json!({
                    "id": e.id,
                    "title": e.title,
                    "description": e.description,
                    "start_at": e.start_at,
                    "end_at": e.end_at,
                    "location": e.location,
                    "created_by": e.created_by,
                })).collect();
                serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
            }

            "documents" => {
                check_cil(&mcp_plugin(), &CilAction::QueryDocuments).map_err(|e| e)?;
                let docs = store::list_documents(&conn, cid).map_err(|e| e.to_string())?;
                let group_key = GroupKey::from_b58(&net.data.group_key_b58)
                    .map_err(|e| e.to_string())?;
                let result: Vec<Value> = docs.iter().map(|d| {
                    let body = group_key.decrypt(&d.nonce_b58, &d.body_ciphertext)
                        .map(|b| String::from_utf8_lossy(&b).into_owned())
                        .unwrap_or_else(|_| "[illisible]".into());
                    json!({"id":d.id,"title":d.title,"body":body,"version":d.version,"created_by":d.created_by,"created_at":d.created_at})
                }).collect();
                serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
            }

            _ => Err(format!("Ressource inconnue : '{resource}'")),
        };
    }

    Err(format!("URI non reconnue : '{uri}'"))
}

// ── HTTP helpers ──────────────────────────────────────────────────────────────

async fn read_request(stream: &mut TcpStream) -> Option<(HashMap<String, String>, String)> {
    let mut buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 4096];

    // Read until we have all headers (\r\n\r\n)
    loop {
        let n = stream.read(&mut tmp).await.ok()?;
        if n == 0 { return None; }
        buf.extend_from_slice(&tmp[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") { break; }
        if buf.len() > 131_072 { return None; }
    }

    let header_end = buf.windows(4).position(|w| w == b"\r\n\r\n")?;
    let headers_raw = String::from_utf8_lossy(&buf[..header_end]).to_string();

    let content_length: usize = headers_raw.lines()
        .find(|l| l.to_lowercase().starts_with("content-length:"))
        .and_then(|l| l.splitn(2, ':').nth(1))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0);

    let body_start = header_end + 4;
    while buf.len() < body_start + content_length {
        let n = stream.read(&mut tmp).await.ok()?;
        if n == 0 { break; }
        buf.extend_from_slice(&tmp[..n]);
    }

    let body = String::from_utf8_lossy(
        buf.get(body_start..body_start + content_length).unwrap_or(&[])
    ).to_string();

    // Parse headers into a map (lowercase keys)
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut lines = headers_raw.lines();
    if let Some(first) = lines.next() {
        // e.g. "POST / HTTP/1.1" — store the method
        let method = first.split_whitespace().next().unwrap_or("").to_lowercase();
        headers.insert("method".to_string(), method);
    }
    for line in lines {
        if let Some(colon) = line.find(':') {
            headers.insert(
                line[..colon].trim().to_lowercase(),
                line[colon + 1..].trim().to_string(),
            );
        }
    }

    Some((headers, body))
}

async fn write_json(stream: &mut TcpStream, value: &Value) {
    let body = value.to_string();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;
}
