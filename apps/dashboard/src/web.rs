use crate::model::{CaseItem, ReviewRequest, ReviewView, SavedReview, ViewMode};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use gennie_memory_retrieval::model::{QueryMode, SearchRequest};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct AppState {
    conn: Arc<Mutex<Connection>>,
    cases: Arc<Vec<CaseItem>>,
    dist: PathBuf,
    port: u16,
    bind_ip: Ipv4Addr,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
    #[serde(skip_serializing)]
    status: StatusCode,
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self)).into_response()
    }
}
fn error(message: impl Into<String>) -> ApiError {
    ApiError {
        error: message.into(),
        status: StatusCode::BAD_REQUEST,
    }
}

fn forbidden(message: &str) -> ApiError {
    ApiError {
        error: message.into(),
        status: StatusCode::FORBIDDEN,
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewRequest {
    case_id: Option<String>,
    search: SearchRequest,
    view_mode: ViewMode,
}

fn check_case(
    state: &AppState,
    case_id: Option<&str>,
    search: &SearchRequest,
) -> Result<(), ApiError> {
    let Some(case_id) = case_id else {
        return Ok(());
    };
    let case = state
        .cases
        .iter()
        .find(|item| item.id == case_id)
        .ok_or_else(|| error("caso de revisión desconocido"))?;
    let original = &case.search;
    if original.query != search.query
        || original.scope_kind != search.scope_kind
        || original.scope_id != search.scope_id
        || original.entity_id != search.entity_id
        || original.as_of != search.as_of
        || original.limit.unwrap_or(20) != search.limit.unwrap_or(20)
        || original.query_mode.unwrap_or(QueryMode::Keywords)
            != search.query_mode.unwrap_or(QueryMode::Keywords)
    {
        return Err(error(
            "la pregunta del caso cambió; crea una revisión nueva sin ID de caso",
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
struct CaseRecord {
    id: String,
    segment: String,
    request: CaseSearch,
}
#[derive(Deserialize)]
struct CaseSearch {
    search: SearchRequest,
}

fn read_cases(path: Option<&Path>) -> Result<Vec<CaseItem>, String> {
    let Some(path) = path else {
        return Ok(vec![]);
    };
    let content =
        fs::read_to_string(path).map_err(|e| format!("no se pudieron leer casos: {e}"))?;
    if content.len() > 10_000_000 {
        return Err("el archivo de casos supera 10 MB".into());
    }
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            let record: CaseRecord =
                serde_json::from_str(line).map_err(|e| format!("caso línea {}: {e}", index + 1))?;
            Ok(CaseItem {
                id: record.id,
                segment: record.segment,
                search: record.request.search,
            })
        })
        .collect()
}

fn check_host(headers: &HeaderMap, state: &AppState) -> Result<(), ApiError> {
    let port = state.port;
    let host = headers
        .get(header::HOST)
        .and_then(|item| item.to_str().ok())
        .unwrap_or("");
    let expected_host = format!("{}:{port}", state.bind_ip);
    if host != expected_host
        && !(state.bind_ip.is_loopback() && host == format!("localhost:{port}"))
    {
        return Err(forbidden("host no autorizado"));
    }
    if let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|item| item.to_str().ok())
    {
        if origin != format!("http://{expected_host}")
            && !(state.bind_ip.is_loopback() && origin == format!("http://localhost:{port}"))
            && !(state.bind_ip.is_loopback() && origin == "http://127.0.0.1:5173")
        {
            return Err(forbidden("origen no autorizado"));
        }
    }
    Ok(())
}

async fn list_cases(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    check_host(&headers, &state)?;
    let conn = state
        .conn
        .lock()
        .map_err(|_| error("SQLite no disponible"))?;
    let reviewed = crate::repo::reviewed_case_ids(&conn).map_err(error)?;
    Ok(Json(
        serde_json::json!({ "cases": state.cases.as_ref(), "reviewed_ids": reviewed }),
    ))
}

async fn view(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ViewRequest>,
) -> Result<Json<ReviewView>, ApiError> {
    check_host(&headers, &state)?;
    check_case(&state, request.case_id.as_deref(), &request.search)?;
    let conn = state
        .conn
        .lock()
        .map_err(|_| error("SQLite no disponible"))?;
    let mut result =
        crate::service::view(&conn, &request.search, request.view_mode).map_err(error)?;
    if request.case_id.is_some() {
        for candidate in &mut result.candidates {
            candidate.source = None;
        }
    }
    Ok(Json(result))
}

async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ReviewRequest>,
) -> Result<Json<SavedReview>, ApiError> {
    check_host(&headers, &state)?;
    check_case(&state, request.case_id.as_deref(), &request.search)?;
    let mut conn = state
        .conn
        .lock()
        .map_err(|_| error("SQLite no disponible"))?;
    crate::service::save(&mut conn, request)
        .map(Json)
        .map_err(error)
}

async fn export(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    check_host(&headers, &state)?;
    let conn = state
        .conn
        .lock()
        .map_err(|_| error("SQLite no disponible"))?;
    let reviews = crate::repo::export_rows(&conn).map_err(error)?;
    Ok(Json(
        serde_json::json!({"schema": "gennie-memory-dashboard-review-v1", "reviews": reviews}),
    ))
}

async fn index(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(e) = check_host(&headers, &state) {
        return e.into_response();
    }
    match fs::read(state.dist.join("index.html")) {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "no-store"),
                (header::REFERRER_POLICY, "no-referrer"),
                (header::CONTENT_SECURITY_POLICY, "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Falta compilar la UI: cd apps/dashboard/web && npm ci && npm run build",
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(bind_ip: Ipv4Addr) -> AppState {
        AppState {
            conn: Arc::new(Mutex::new(Connection::open_in_memory().unwrap())),
            cases: Arc::new(vec![]),
            dist: PathBuf::new(),
            port: 8787,
            bind_ip,
        }
    }

    #[test]
    fn lan_allows_direct_access_but_rejects_other_hosts_and_origins() {
        let state = state(Ipv4Addr::new(192, 168, 1, 215));
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "192.168.1.215:8787".parse().unwrap());
        assert!(check_host(&headers, &state).is_ok());
        headers.insert(header::ORIGIN, "http://evil.example".parse().unwrap());
        assert_eq!(
            check_host(&headers, &state).unwrap_err().status,
            StatusCode::FORBIDDEN
        );
        headers.remove(header::ORIGIN);
        headers.insert(header::HOST, "evil.example:8787".parse().unwrap());
        assert_eq!(
            check_host(&headers, &state).unwrap_err().status,
            StatusCode::FORBIDDEN
        );
    }

    #[test]
    fn loopback_preserves_local_access() {
        let state = state(Ipv4Addr::LOCALHOST);
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "localhost:8787".parse().unwrap());
        assert!(check_host(&headers, &state).is_ok());
    }
}

async fn asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Response {
    if let Err(e) = check_host(&headers, &state) {
        return e.into_response();
    }
    if name.contains('/') || name.contains('\\') || name.starts_with('.') {
        return StatusCode::NOT_FOUND.into_response();
    }
    let path = state.dist.join("assets").join(&name);
    let mime = if name.ends_with(".js") {
        "text/javascript"
    } else if name.ends_with(".css") {
        "text/css"
    } else if name.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    };
    match fs::read(path) {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, mime),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn run(
    conn: Connection,
    port: u16,
    case_path: Option<&Path>,
    dist: PathBuf,
    bind_ip: Ipv4Addr,
) -> Result<(), String> {
    if port == 0 {
        return Err("elige un puerto entre 1 y 65535".into());
    }
    if !bind_ip.is_loopback() && !bind_ip.is_private() {
        return Err("la IP del dashboard debe ser loopback o privada de la red local".into());
    }
    let cases = read_cases(case_path)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(SocketAddr::new(IpAddr::V4(bind_ip), port))
            .await
            .map_err(|e| format!("no se pudo abrir el puerto {port}: {e}"))?;
        let state = AppState {
            conn: Arc::new(Mutex::new(conn)),
            cases: Arc::new(cases),
            dist,
            port,
            bind_ip,
        };
        let router = Router::new()
            .route("/", get(index))
            .route("/assets/{name}", get(asset))
            .route("/api/cases", get(list_cases))
            .route("/api/view", post(view))
            .route("/api/reviews", post(save))
            .route("/api/export", get(export))
            .with_state(state);
        println!("Dashboard: http://{bind_ip}:{port}/ (Ctrl+C para cerrar)");
        axum::serve(listener, router)
            .await
            .map_err(|e| e.to_string())
    })
}
