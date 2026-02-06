/// REST API + Web UI server via EspHttpServer
///
/// Serves on port 80.
/// REST endpoints for task CRUD, time sync, and WiFi management.
extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use std::sync::{Arc, Mutex};

use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};

use chrono::NaiveDate;
use serde_json::json;

use crate::config;
use crate::models::RecurrenceType;
use crate::storage::Storage;
use crate::wifi::{self, WiFiMode};

/// Shared state between HTTP server and main thread
pub type SharedStorage = Arc<Mutex<Storage>>;

/// Shared time source - seconds since epoch, set by phone
pub type SharedTime = Arc<Mutex<Option<i64>>>;

/// Shared WiFi instance for scanning (AP mode only)
pub type SharedWifi = Arc<Mutex<BlockingWifi<EspWifi<'static>>>>;

/// Start the HTTP server
pub fn start_server(
    storage: SharedStorage,
    time_source: SharedTime,
    ip: [u8; 4],
    wifi_mode: WiFiMode,
    shared_wifi: Option<SharedWifi>,
    nvs_partition: Option<EspDefaultNvsPartition>,
) -> Result<EspHttpServer<'static>, Box<dyn std::error::Error>> {
    let server_config = HttpConfig {
        http_port: config::HTTP_PORT,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&server_config)?;

    // GET / -> serve index.html
    {
        server.fn_handler("/", Method::Get, |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let html = include_str!("../static/index.html");
            req.into_ok_response()?
                .write(html.as_bytes())?;
            Ok(())
        })?;
    }

    // Captive portal detection handlers
    // Redirect connectivity checks to our web UI so phones auto-open it
    {
        let url = format!("http://{}.{}.{}.{}/", ip[0], ip[1], ip[2], ip[3]);

        // Android connectivity check
        let redirect = url.clone();
        server.fn_handler("/generate_204", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let mut resp = req.into_response(302, None, &[("Location", &redirect)])?;
            resp.write(&[])?;
            Ok(())
        })?;

        // Android alternate
        let redirect = url.clone();
        server.fn_handler("/gen_204", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let mut resp = req.into_response(302, None, &[("Location", &redirect)])?;
            resp.write(&[])?;
            Ok(())
        })?;

        // iOS / macOS captive portal detection
        let redirect = url.clone();
        server.fn_handler("/hotspot-detect.html", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let mut resp = req.into_response(302, None, &[("Location", &redirect)])?;
            resp.write(&[])?;
            Ok(())
        })?;

        // Windows connectivity check
        let redirect = url.clone();
        server.fn_handler("/connecttest.txt", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let mut resp = req.into_response(302, None, &[("Location", &redirect)])?;
            resp.write(&[])?;
            Ok(())
        })?;

        // Windows NCSI
        let redirect = url.clone();
        server.fn_handler("/ncsi.txt", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let mut resp = req.into_response(302, None, &[("Location", &redirect)])?;
            resp.write(&[])?;
            Ok(())
        })?;

        // Firefox captive portal detection
        let redirect = url.clone();
        server.fn_handler("/canonical.html", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let mut resp = req.into_response(302, None, &[("Location", &redirect)])?;
            resp.write(&[])?;
            Ok(())
        })?;
    }

    // GET /health
    {
        let time = time_source.clone();
        server.fn_handler("/health", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let timestamp = time.lock().unwrap().unwrap_or(0);
            let body = json!({
                "status": "ok",
                "timestamp": timestamp
            })
            .to_string();
            let mut resp = req.into_ok_response()?;
            resp.write(body.as_bytes())?;
            Ok(())
        })?;
    }

    // GET /api/tasks
    {
        let store = storage.clone();
        let time = time_source.clone();
        server.fn_handler("/api/tasks", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let s = store.lock().unwrap();
            let today = get_today(&time);
            let tasks = s.get_all_tasks(true);
            let json_tasks: Vec<serde_json::Value> = tasks
                .iter()
                .map(|t| {
                    json!({
                        "id": t.id,
                        "name": t.name,
                        "recurrenceType": t.recurrence_type.as_str(),
                        "recurrenceValue": t.recurrence_value,
                        "nextDueDate": t.next_due_date,
                        "daysUntilDue": t.days_until_due(today),
                        "urgency": t.urgency(today).as_str(),
                        "createdAt": t.created_at,
                        "updatedAt": t.updated_at,
                    })
                })
                .collect();
            let body = serde_json::to_string(&json_tasks).unwrap_or_else(|_| "[]".into());
            let mut resp = req.into_ok_response()?;
            resp.write(body.as_bytes())?;
            Ok(())
        })?;
    }

    // POST /api/tasks
    {
        let store = storage.clone();
        let time = time_source.clone();
        server.fn_handler("/api/tasks", Method::Post, move |mut req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let mut buf = [0u8; 1024];
            let len = req.read(&mut buf).unwrap_or(0);
            let body_str = core::str::from_utf8(&buf[..len]).unwrap_or("");

            let parsed: Result<serde_json::Value, _> = serde_json::from_str(body_str);
            match parsed {
                Ok(data) => {
                    let name = data["name"].as_str().unwrap_or("").to_string();
                    let rec_type_str = data["recurrenceType"].as_str().unwrap_or("daily");
                    let rec_value = data["recurrenceValue"].as_u64().unwrap_or(1) as u32;
                    let next_due = data["nextDueDate"].as_str().unwrap_or("").to_string();

                    let recurrence_type = match rec_type_str {
                        "weekly" => RecurrenceType::Weekly,
                        "monthly" => RecurrenceType::Monthly,
                        "yearly" => RecurrenceType::Yearly,
                        _ => RecurrenceType::Daily,
                    };

                    let now_iso = get_now_iso(&time);
                    let today = get_today(&time);
                    let mut s = store.lock().unwrap();
                    let task = s.create_task(name, recurrence_type, rec_value, next_due, &now_iso);

                    let resp_body = json!({
                        "id": task.id,
                        "name": task.name,
                        "recurrenceType": task.recurrence_type.as_str(),
                        "recurrenceValue": task.recurrence_value,
                        "nextDueDate": task.next_due_date,
                        "daysUntilDue": task.days_until_due(today),
                        "urgency": task.urgency(today).as_str(),
                    })
                    .to_string();

                    let mut resp = req.into_response(201, None, &[("Content-Type", "application/json")])?;
                    resp.write(resp_body.as_bytes())?;
                }
                Err(_) => {
                    let err = json!({"error": "Invalid JSON"}).to_string();
                    let mut resp = req.into_response(400, None, &[("Content-Type", "application/json")])?;
                    resp.write(err.as_bytes())?;
                }
            }
            Ok(())
        })?;
    }

    // POST /api/time - receive timestamp from phone JS for RTC sync
    {
        let time = time_source.clone();
        server.fn_handler("/api/time", Method::Post, move |mut req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let mut buf = [0u8; 256];
            let len = req.read(&mut buf).unwrap_or(0);
            let body_str = core::str::from_utf8(&buf[..len]).unwrap_or("");

            if let Ok(data) = serde_json::from_str::<serde_json::Value>(body_str) {
                if let Some(ts) = data["timestamp"].as_i64() {
                    *time.lock().unwrap() = Some(ts / 1000); // JS sends milliseconds
                    log::info!("Time synced from phone: {}", ts / 1000);
                }
            }

            let body = json!({"status": "ok"}).to_string();
            let mut resp = req.into_ok_response()?;
            resp.write(body.as_bytes())?;
            Ok(())
        })?;
    }

    // === WiFi management endpoints ===

    // GET /api/wifi/status
    {
        let mode = wifi_mode.clone();
        server.fn_handler("/api/wifi/status", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let ip = mode.ip();
            let body = json!({
                "mode": mode.mode_str(),
                "ssid": mode.ssid().unwrap_or(""),
                "ip": format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]),
                "connected": mode.is_station(),
                "hostname": config::MDNS_HOSTNAME,
            })
            .to_string();
            let mut resp = req.into_ok_response()?;
            resp.write(body.as_bytes())?;
            Ok(())
        })?;
    }

    // GET /api/wifi/scan
    {
        let shared_w = shared_wifi.clone();
        server.fn_handler("/api/wifi/scan", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            if let Some(ref w) = shared_w {
                let mut wifi_guard = w.lock().unwrap();
                let networks = wifi::scan_networks(&mut wifi_guard);
                let json_networks: Vec<serde_json::Value> = networks
                    .iter()
                    .map(|n| {
                        json!({
                            "ssid": n.ssid,
                            "rssi": n.rssi,
                            "auth": n.auth,
                        })
                    })
                    .collect();
                let body = serde_json::to_string(&json_networks).unwrap_or_else(|_| "[]".into());
                let mut resp = req.into_ok_response()?;
                resp.write(body.as_bytes())?;
            } else {
                let body = json!({"error": "Scan only available in AP mode"}).to_string();
                let mut resp = req.into_response(400, None, &[("Content-Type", "application/json")])?;
                resp.write(body.as_bytes())?;
            }
            Ok(())
        })?;
    }

    // POST /api/wifi/connect
    {
        let nvs = nvs_partition.clone();
        server.fn_handler("/api/wifi/connect", Method::Post, move |mut req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let mut buf = [0u8; 512];
            let len = req.read(&mut buf).unwrap_or(0);
            let body_str = core::str::from_utf8(&buf[..len]).unwrap_or("");

            if let Ok(data) = serde_json::from_str::<serde_json::Value>(body_str) {
                let ssid = data["ssid"].as_str().unwrap_or("");
                let password = data["password"].as_str().unwrap_or("");

                if ssid.is_empty() {
                    let err = json!({"error": "SSID required"}).to_string();
                    let mut resp = req.into_response(400, None, &[("Content-Type", "application/json")])?;
                    resp.write(err.as_bytes())?;
                    return Ok(());
                }

                if let Some(ref nvs_part) = nvs {
                    if let Err(e) = wifi::save_wifi_creds(nvs_part, ssid, password) {
                        log::error!("Failed to save WiFi creds: {}", e);
                        let err = json!({"error": "Failed to save credentials"}).to_string();
                        let mut resp = req.into_response(500, None, &[("Content-Type", "application/json")])?;
                        resp.write(err.as_bytes())?;
                        return Ok(());
                    }
                }

                let body = json!({"status": "ok", "message": "Credentials saved. Restarting..."}).to_string();
                let mut resp = req.into_ok_response()?;
                resp.write(body.as_bytes())?;

                // Schedule restart after response is sent
                std::thread::spawn(|| {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    log::info!("Restarting to apply WiFi credentials...");
                    unsafe { esp_idf_svc::sys::esp_restart(); }
                });
            } else {
                let err = json!({"error": "Invalid JSON"}).to_string();
                let mut resp = req.into_response(400, None, &[("Content-Type", "application/json")])?;
                resp.write(err.as_bytes())?;
            }
            Ok(())
        })?;
    }

    // DELETE /api/wifi/credentials
    {
        let nvs = nvs_partition.clone();
        server.fn_handler("/api/wifi/credentials", Method::Delete, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            if let Some(ref nvs_part) = nvs {
                let _ = wifi::clear_wifi_creds(nvs_part);
            }

            let body = json!({"status": "ok", "message": "Credentials cleared. Restarting..."}).to_string();
            let mut resp = req.into_ok_response()?;
            resp.write(body.as_bytes())?;

            // Schedule restart
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_secs(1));
                log::info!("Restarting after WiFi credential reset...");
                unsafe { esp_idf_svc::sys::esp_restart(); }
            });

            Ok(())
        })?;
    }

    // Register dynamic task routes using a catch-all pattern
    // EspHttpServer doesn't have route params, so we parse manually
    register_task_routes(&mut server, storage.clone(), time_source.clone())?;

    log::info!("HTTP server started on port {}", config::HTTP_PORT);
    Ok(server)
}

/// Register routes for /api/tasks/* (GET, PUT, DELETE single task + complete + history)
fn register_task_routes(
    server: &mut EspHttpServer<'static>,
    storage: SharedStorage,
    time_source: SharedTime,
) -> Result<(), Box<dyn std::error::Error>> {
    // GET /api/tasks/*  (single task, history)
    {
        let store = storage.clone();
        let time = time_source.clone();
        server.fn_handler("/api/tasks/*", Method::Get, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let uri = req.uri();
            let parts: Vec<&str> = uri.trim_start_matches("/api/tasks/").split('/').collect();

            if parts.is_empty() {
                let err = json!({"error": "Not found"}).to_string();
                let mut resp = req.into_response(404, None, &[("Content-Type", "application/json")])?;
                resp.write(err.as_bytes())?;
                return Ok(());
            }

            let task_id: u32 = match parts[0].parse() {
                Ok(id) => id,
                Err(_) => {
                    let err = json!({"error": "Invalid task ID"}).to_string();
                    let mut resp = req.into_response(400, None, &[("Content-Type", "application/json")])?;
                    resp.write(err.as_bytes())?;
                    return Ok(());
                }
            };

            let s = store.lock().unwrap();
            let today = get_today(&time);

            if parts.len() >= 2 && parts[1] == "history" {
                // GET /api/tasks/:id/history
                let task = s.get_task(task_id);
                if task.is_none() {
                    let err = json!({"error": "Task not found"}).to_string();
                    let mut resp = req.into_response(404, None, &[("Content-Type", "application/json")])?;
                    resp.write(err.as_bytes())?;
                    return Ok(());
                }

                let history = s.get_task_history(task_id);
                let json_history: Vec<serde_json::Value> = history
                    .iter()
                    .map(|h| {
                        json!({
                            "id": h.id,
                            "completedAt": h.completed_at,
                            "daysSinceLast": h.days_since_last,
                        })
                    })
                    .collect();

                let body = serde_json::to_string(&json_history).unwrap_or_else(|_| "[]".into());
                let mut resp = req.into_ok_response()?;
                resp.write(body.as_bytes())?;
            } else {
                // GET /api/tasks/:id
                match s.get_task(task_id) {
                    Some(task) => {
                        let body = json!({
                            "id": task.id,
                            "name": task.name,
                            "recurrenceType": task.recurrence_type.as_str(),
                            "recurrenceValue": task.recurrence_value,
                            "nextDueDate": task.next_due_date,
                            "daysUntilDue": task.days_until_due(today),
                            "urgency": task.urgency(today).as_str(),
                            "createdAt": task.created_at,
                            "updatedAt": task.updated_at,
                        })
                        .to_string();
                        let mut resp = req.into_ok_response()?;
                        resp.write(body.as_bytes())?;
                    }
                    None => {
                        let err = json!({"error": "Task not found"}).to_string();
                        let mut resp = req.into_response(404, None, &[("Content-Type", "application/json")])?;
                        resp.write(err.as_bytes())?;
                    }
                }
            }
            Ok(())
        })?;
    }

    // PUT /api/tasks/*
    {
        let store = storage.clone();
        let time = time_source.clone();
        server.fn_handler("/api/tasks/*", Method::Put, move |mut req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let uri = req.uri().to_string();
            let task_id_str = uri.trim_start_matches("/api/tasks/").split('/').next().unwrap_or("");
            let task_id: u32 = match task_id_str.parse() {
                Ok(id) => id,
                Err(_) => {
                    let err = json!({"error": "Invalid task ID"}).to_string();
                    let mut resp = req.into_response(400, None, &[("Content-Type", "application/json")])?;
                    resp.write(err.as_bytes())?;
                    return Ok(());
                }
            };

            let mut buf = [0u8; 1024];
            let len = req.read(&mut buf).unwrap_or(0);
            let body_str = core::str::from_utf8(&buf[..len]).unwrap_or("");

            if let Ok(data) = serde_json::from_str::<serde_json::Value>(body_str) {
                let name = data["name"].as_str().map(String::from);
                let rec_type = data["recurrenceType"].as_str().map(|s| match s {
                    "weekly" => RecurrenceType::Weekly,
                    "monthly" => RecurrenceType::Monthly,
                    "yearly" => RecurrenceType::Yearly,
                    _ => RecurrenceType::Daily,
                });
                let rec_value = data["recurrenceValue"].as_u64().map(|v| v as u32);
                let next_due = data["nextDueDate"].as_str().map(String::from);

                let now_iso = get_now_iso(&time);
                let today = get_today(&time);
                let mut s = store.lock().unwrap();

                match s.update_task(task_id, name, rec_type, rec_value, next_due, &now_iso) {
                    Some(task) => {
                        let body = json!({
                            "id": task.id,
                            "name": task.name,
                            "recurrenceType": task.recurrence_type.as_str(),
                            "recurrenceValue": task.recurrence_value,
                            "nextDueDate": task.next_due_date,
                            "daysUntilDue": task.days_until_due(today),
                            "urgency": task.urgency(today).as_str(),
                        })
                        .to_string();
                        let mut resp = req.into_ok_response()?;
                        resp.write(body.as_bytes())?;
                    }
                    None => {
                        let err = json!({"error": "Task not found"}).to_string();
                        let mut resp = req.into_response(404, None, &[("Content-Type", "application/json")])?;
                        resp.write(err.as_bytes())?;
                    }
                }
            } else {
                let err = json!({"error": "Invalid JSON"}).to_string();
                let mut resp = req.into_response(400, None, &[("Content-Type", "application/json")])?;
                resp.write(err.as_bytes())?;
            }
            Ok(())
        })?;
    }

    // DELETE /api/tasks/*
    {
        let store = storage.clone();
        server.fn_handler("/api/tasks/*", Method::Delete, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let uri = req.uri().to_string();
            let task_id_str = uri.trim_start_matches("/api/tasks/").split('/').next().unwrap_or("");
            let task_id: u32 = match task_id_str.parse() {
                Ok(id) => id,
                Err(_) => {
                    let err = json!({"error": "Invalid task ID"}).to_string();
                    let mut resp = req.into_response(400, None, &[("Content-Type", "application/json")])?;
                    resp.write(err.as_bytes())?;
                    return Ok(());
                }
            };

            let mut s = store.lock().unwrap();
            if s.delete_task(task_id) {
                let mut resp = req.into_response(204, None, &[])?;
                resp.write(&[])?;
            } else {
                let err = json!({"error": "Task not found"}).to_string();
                let mut resp = req.into_response(404, None, &[("Content-Type", "application/json")])?;
                resp.write(err.as_bytes())?;
            }
            Ok(())
        })?;
    }

    // POST /api/tasks/*/complete
    {
        let store = storage.clone();
        let time = time_source.clone();
        server.fn_handler("/api/tasks/*/complete", Method::Post, move |req| -> Result<(), esp_idf_svc::io::EspIOError> {
            let uri = req.uri().to_string();
            let task_id_str = uri
                .trim_start_matches("/api/tasks/")
                .trim_end_matches("/complete")
                .split('/')
                .next()
                .unwrap_or("");
            let task_id: u32 = match task_id_str.parse() {
                Ok(id) => id,
                Err(_) => {
                    let err = json!({"error": "Invalid task ID"}).to_string();
                    let mut resp = req.into_response(400, None, &[("Content-Type", "application/json")])?;
                    resp.write(err.as_bytes())?;
                    return Ok(());
                }
            };

            let now_iso = get_now_iso(&time);
            let today = get_today(&time);
            let mut s = store.lock().unwrap();

            if s.complete_task(task_id, &now_iso, today) {
                if let Some(task) = s.get_task(task_id) {
                    let body = json!({
                        "id": task.id,
                        "name": task.name,
                        "nextDueDate": task.next_due_date,
                        "daysUntilDue": task.days_until_due(today),
                        "urgency": task.urgency(today).as_str(),
                    })
                    .to_string();
                    let mut resp = req.into_ok_response()?;
                    resp.write(body.as_bytes())?;
                } else {
                    let err = json!({"error": "Task not found"}).to_string();
                    let mut resp = req.into_response(404, None, &[("Content-Type", "application/json")])?;
                    resp.write(err.as_bytes())?;
                }
            } else {
                let err = json!({"error": "Task not found"}).to_string();
                let mut resp = req.into_response(404, None, &[("Content-Type", "application/json")])?;
                resp.write(err.as_bytes())?;
            }
            Ok(())
        })?;
    }

    Ok(())
}

/// Get today's date from the shared time source
fn get_today(time: &SharedTime) -> NaiveDate {
    let secs = time.lock().unwrap().unwrap_or(0);
    if secs > 0 {
        chrono::DateTime::from_timestamp(secs, 0)
            .map(|dt| dt.date_naive())
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
    } else {
        // Fallback if time not yet synced
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
    }
}

/// Get current datetime as ISO string
fn get_now_iso(time: &SharedTime) -> String {
    let secs = time.lock().unwrap().unwrap_or(0);
    if secs > 0 {
        chrono::DateTime::from_timestamp(secs, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
            .unwrap_or_else(|| String::from("2025-01-01T00:00:00"))
    } else {
        String::from("2025-01-01T00:00:00")
    }
}
