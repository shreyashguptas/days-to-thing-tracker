/// Days Tracker Kiosk - ESP32-C6 Firmware
///
/// Standalone embedded firmware for XIAO ESP32-C6 with:
/// - ST7735 160x128 TFT display via SPI
/// - KY-040 rotary encoder for navigation
/// - WiFi Station mode (joins home WiFi) with SoftAP provisioning fallback
/// - JSON storage on LittleFS
extern crate alloc;

use alloc::format;
use alloc::string::String;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriverConfig};
use esp_idf_hal::units::FromValueType;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use display_interface_spi::SPIInterface;
use mipidsi::models::ST7735s;
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::Builder;

mod config;
mod display;
mod dns;
mod encoder;
mod fonts;
mod http_server;
mod models;
mod renderer;
mod storage;
mod theme;
mod views;
mod wifi;

use display::FrameBuffer;
use encoder::{Encoder, EncoderEvent};
use http_server::{get_now_iso, get_today, SharedStorage, SharedTime, SharedWifi};
use models::{HistoryDisplayEntry, TaskDisplayData};
use renderer::Renderer;
use storage::Storage;
use views::{RenderCommand, ViewNavigator, ViewState};
use wifi::WiFiMode;

fn main() {
    // Initialize ESP-IDF
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    log::info!("Days Tracker Kiosk Starting...");

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().ok();

    // === Initialize SPI Display ===
    log::info!("Initializing display...");

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio19;  // D8 - SPI clock
    let mosi = peripherals.pins.gpio18;  // D10 - SPI MOSI
    let cs = peripherals.pins.gpio21;    // D3 - Chip select
    let dc = PinDriver::output(peripherals.pins.gpio22).unwrap();  // D4 - Data/command
    let rst = PinDriver::output(peripherals.pins.gpio23).unwrap(); // D5 - Reset

    let spi_driver = SpiDeviceDriver::new_single(
        spi,
        sclk,
        mosi,
        Option::<esp_idf_hal::gpio::AnyIOPin>::None,
        Some(cs),
        &SpiDriverConfig::default(),
        &SpiConfig::new().baudrate(config::SPI_FREQ_HZ.Hz()),
    )
    .unwrap();

    let spi_iface = SPIInterface::new(spi_driver, dc);

    let mut hw_display = Builder::new(ST7735s, spi_iface)
        .reset_pin(rst)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut FreeRtos)
        .unwrap();

    log::info!("Display initialized");

    // === Initialize Encoder + Backlight ===
    log::info!("Initializing encoder...");
    let backlight_on = Arc::new(AtomicBool::new(true));
    let mut enc = Encoder::new(
        peripherals.pins.gpio0,   // D0 - Encoder CLK (A)
        peripherals.pins.gpio1,   // D1 - Encoder DT (B)
        peripherals.pins.gpio2,   // D2 - Encoder switch
        peripherals.pins.gpio20,  // D9 - Backlight
        backlight_on.clone(),
    )
    .unwrap();
    log::info!("Encoder initialized");

    // === Create framebuffer ===
    let mut fb = FrameBuffer::new();

    // === Determine WiFi mode: Station (saved creds) or AP (provisioning) ===
    Renderer::render_connecting(&mut fb, "Starting WiFi...");
    flush_to_display(&mut hw_display, &fb);

    // Clone NVS partition for credential access (separate from WiFi driver)
    let nvs_for_creds = nvs.clone();

    // Extract modem before branching (consumed by whichever WiFi mode initializes)
    let modem = peripherals.modem;

    // Check for saved WiFi credentials
    let saved_creds = nvs_for_creds
        .as_ref()
        .and_then(|nvs_part| wifi::load_wifi_creds(nvs_part));

    let (wifi_mode, _wifi, shared_wifi, _dns_handle): (
        WiFiMode,
        Option<wifi::BlockingWifiHandle>,
        Option<SharedWifi>,
        bool,
    ) = if let Some(ref creds) = saved_creds {
        // === Station Mode: Connect to saved WiFi ===
        log::info!("Found saved WiFi credentials, trying Station mode...");
        Renderer::render_connecting(&mut fb, &format!("Connecting to {}...", creds.ssid));
        flush_to_display(&mut hw_display, &fb);

        // Try connecting (single attempt — on failure, clear creds and restart into AP)
        log::info!("Connecting to '{}'...", creds.ssid);

        let result = wifi::init_station(
            modem,
            sysloop.clone(),
            nvs.clone(),
            creds,
        );

        if let Ok((wifi_inst, ip)) = result {
            let ssid = creds.ssid.clone();
            let mode = WiFiMode::Station { ssid: ssid.clone(), ip };

            let url = wifi::web_url_from_ip(ip);
            Renderer::render_connected(&mut fb, &ssid, &url);
            flush_to_display(&mut hw_display, &fb);
            FreeRtos::delay_ms(2000);

            log::info!("WiFi Station mode ready: {}", url);

            // No shared_wifi needed in STA mode (no scanning)
            // No DNS captive portal needed in STA mode
            (mode, Some(wifi_inst), None::<SharedWifi>, false)
        } else {
            // Connection failed — clear bad credentials and restart into AP mode
            log::error!("Station connection failed, clearing credentials and restarting...");
            Renderer::render_wifi_failed(&mut fb, &creds.ssid);
            flush_to_display(&mut hw_display, &fb);

            if let Some(ref nvs_part) = nvs_for_creds {
                let _ = wifi::clear_wifi_creds(nvs_part);
            }

            FreeRtos::delay_ms(3000);
            unsafe { esp_idf_svc::sys::esp_restart(); }
            // esp_restart() never returns, but we need to satisfy the type checker
            loop { FreeRtos::delay_ms(1000); }
        }
    } else {
        // === AP Mode: Provisioning ===
        log::info!("No saved WiFi credentials, starting SoftAP provisioning...");
        Renderer::render_connecting(&mut fb, "Starting setup...");
        flush_to_display(&mut hw_display, &fb);

        let wifi_inst = wifi::init_softap(modem, sysloop, nvs.clone()).unwrap();
        log::info!("WiFi SoftAP ready");

        // Configure captive portal
        let ap_ip = wifi::configure_captive_portal(&wifi_inst);
        let ap_url = format!("http://{}.{}.{}.{}", ap_ip[0], ap_ip[1], ap_ip[2], ap_ip[3]);
        dns::start(ap_ip);
        log::info!("Captive portal ready: {}", ap_url);

        let mode = WiFiMode::AccessPoint { ip: ap_ip };

        // Wrap WiFi in Arc<Mutex> for scan access from HTTP server
        let shared_wifi: SharedWifi = Arc::new(Mutex::new(wifi_inst));

        (mode, None, Some(shared_wifi), true)
    };

    // === Mount Storage ===
    log::info!("Mounting storage...");
    let _spiffs = unsafe { esp_idf_svc::fs::spiffs::Spiffs::new(config::STORAGE_PARTITION) };

    let storage = Arc::new(Mutex::new(Storage::new(
        config::TASKS_FILE,
        config::HISTORY_FILE,
    )));

    // === Shared time source (synced from phone) ===
    let time_source: SharedTime = Arc::new(Mutex::new(None));

    // === Start HTTP Server ===
    log::info!("Starting HTTP server...");
    let _server = http_server::start_server(
        storage.clone(),
        time_source.clone(),
        wifi_mode.ip(),
        wifi_mode.clone(),
        shared_wifi,
        nvs_for_creds.clone(),
    )
    .unwrap();
    log::info!("HTTP server ready on port {}", config::HTTP_PORT);

    // === Initialize View Navigator ===
    let mut nav = ViewNavigator::new();
    nav.ctx.wifi_mode = wifi_mode.clone();

    // Set the URL based on WiFi mode
    nav.ctx.ap_url = match &wifi_mode {
        WiFiMode::Station { ip, .. } => wifi::web_url_from_ip(*ip),
        WiFiMode::AccessPoint { ip } => format!("http://{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]),
    };

    // Load initial data
    {
        let s = storage.lock().unwrap();
        let today = get_today(&time_source);
        let counts = s.get_task_counts(today);
        nav.set_task_counts(counts);
        let tasks = s.get_all_tasks(true);
        nav.set_tasks(tasks);
    }

    // AP mode (no WiFi provisioned): show WiFi QR code as the entry point
    // STA mode (connected to WiFi): always show Dashboard
    if matches!(&wifi_mode, WiFiMode::AccessPoint { .. }) {
        nav.ctx.state = ViewState::QrCode;
    }

    // === Main Event Loop ===
    log::info!("Ready. Entering main loop.");

    // Keep NVS partition reference for WiFi reset action
    let nvs_for_reset = nvs_for_creds;

    let mut last_idle_check = Instant::now();
    let mut needs_render = true;

    loop {
        // Poll encoder
        if let Some(event) = enc.poll() {
            let action = match event {
                EncoderEvent::Clockwise => {
                    nav.handle_clockwise();
                    None
                }
                EncoderEvent::CounterClockwise => {
                    nav.handle_counter_clockwise();
                    None
                }
                EncoderEvent::ShortPress => nav.handle_press(),
                EncoderEvent::LongPress => nav.handle_long_press(),
            };

            // Handle actions
            if let Some(action) = action {
                handle_action(action, &mut nav, &storage, &time_source, &nvs_for_reset);
            }

            needs_render = true;
        }

        // Render if state changed
        if needs_render {
            render_current_view(&mut fb, &nav, &storage, &time_source);
            flush_to_display(&mut hw_display, &fb);
            needs_render = false;
        }

        // Check idle timeout periodically
        let now = Instant::now();
        if now.duration_since(last_idle_check) > Duration::from_secs(1) {
            if nav.ctx.screen_timeout_enabled
                && enc.seconds_since_activity() > config::IDLE_TIMEOUT_SECS as f64
                && enc.is_backlight_on()
            {
                enc.set_backlight(false);
                log::info!("Screen off (idle timeout)");
            }
            last_idle_check = now;
        }

        // Poll interval
        FreeRtos::delay_ms(config::POLL_INTERVAL_MS as u32);
    }
}

/// Handle action strings from the view navigator
fn handle_action(
    action: &str,
    nav: &mut ViewNavigator,
    storage: &SharedStorage,
    time_source: &SharedTime,
    nvs_partition: &Option<EspDefaultNvsPartition>,
) {
    let today = get_today(time_source);

    match action {
        "complete" => {
            if let Some(task) = nav.ctx.current_task() {
                let task_id = task.id;
                let now_iso = get_now_iso(time_source);

                // Run completion animation
                let start = Instant::now();
                let duration_ms = config::COMPLETING_DURATION_MS;

                while start.elapsed().as_millis() < duration_ms as u128 {
                    let progress = start.elapsed().as_millis() as f32 / duration_ms as f32;
                    nav.ctx.completing_progress = progress.min(1.0);
                    FreeRtos::delay_ms(16); // ~60fps
                }

                // Actually complete in storage
                {
                    let mut s = storage.lock().unwrap();
                    s.complete_task(task_id, &now_iso, today);
                }

                // Reload tasks and counts
                reload_data(nav, storage, time_source);
                nav.complete_animation_done();
            }
        }
        "delete" => {
            if let Some(task) = nav.ctx.current_task() {
                let task_id = task.id;
                {
                    let mut s = storage.lock().unwrap();
                    s.delete_task(task_id);
                }
                reload_data(nav, storage, time_source);
            }
        }
        "load_history" => {
            if let Some(task) = nav.ctx.current_task() {
                let task_id = task.id;
                let s = storage.lock().unwrap();
                let history = s.get_task_history(task_id);
                nav.set_history(history);
            }
        }
        "toggle_timeout" => {
            log::info!(
                "Screen timeout {}",
                if nav.ctx.screen_timeout_enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
        "reset_wifi" => {
            log::info!("Resetting WiFi credentials and restarting...");
            if let Some(ref nvs_part) = nvs_partition {
                let _ = wifi::clear_wifi_creds(nvs_part);
            }
            FreeRtos::delay_ms(500);
            unsafe { esp_idf_svc::sys::esp_restart(); }
        }
        "filter_tasks" => {
            let urgency = nav.ctx.filtered_urgency.clone().unwrap_or_default();
            let s = storage.lock().unwrap();
            let tasks = s.get_tasks_by_urgency(&urgency, today);
            nav.set_tasks(tasks);
        }
        "show_all_tasks" => {
            let s = storage.lock().unwrap();
            let tasks = s.get_all_tasks(true);
            nav.set_tasks(tasks);
        }
        "go_dashboard" => {
            let s = storage.lock().unwrap();
            let counts = s.get_task_counts(today);
            nav.set_task_counts(counts);
        }
        "show_settings" | "show_qr" => {
            // View transition handled by navigator
        }
        _ => {}
    }
}

/// Reload tasks and counts after mutations
fn reload_data(nav: &mut ViewNavigator, storage: &SharedStorage, time_source: &SharedTime) {
    let today = get_today(time_source);
    let s = storage.lock().unwrap();

    let counts = s.get_task_counts(today);
    nav.set_task_counts(counts);

    let tasks = match &nav.ctx.filtered_urgency {
        Some(urgency) => s.get_tasks_by_urgency(urgency, today),
        None => s.get_all_tasks(true),
    };
    nav.set_tasks(tasks);
}

/// Render the current view to the framebuffer
fn render_current_view(
    fb: &mut FrameBuffer,
    nav: &ViewNavigator,
    storage: &SharedStorage,
    time_source: &SharedTime,
) {
    let today = get_today(time_source);

    match nav.get_render_command() {
        RenderCommand::Dashboard { counts, selected } => {
            Renderer::render_dashboard(fb, counts.overdue, counts.today, counts.week, counts.total, selected);
        }
        RenderCommand::TaskCard {
            task_index,
            total,
            filtered: _,
        } => {
            if let Some(task) = nav.ctx.tasks.get(task_index) {
                let display_data = TaskDisplayData {
                    name: task.name.clone(),
                    days_until_due: task.days_until_due(today),
                    urgency: String::from(task.urgency(today).as_str()),
                    next_due_date: task.formatted_due_date(),
                };
                Renderer::render_task_card(fb, &display_data, task_index, total);
            }
        }
        RenderCommand::BackCard { total } => {
            Renderer::render_back_card(fb, total);
        }
        RenderCommand::EmptyFiltered { filter_name } => {
            Renderer::render_empty_filtered(fb, &filter_name);
        }
        RenderCommand::Empty { ref wifi_mode } => {
            Renderer::render_empty(fb, wifi_mode);
        }
        RenderCommand::ActionMenu {
            task_name,
            selected,
            options,
        } => {
            let opt_refs: alloc::vec::Vec<&str> = options.iter().map(|s| s.as_str()).collect();
            Renderer::render_action_menu(fb, &task_name, selected, &opt_refs);
        }
        RenderCommand::ConfirmDialog {
            task_name,
            confirm_selected,
        } => {
            let msg = format!("Delete '{}'?", task_name);
            Renderer::render_confirm_dialog(fb, &msg, confirm_selected);
        }
        RenderCommand::Completing {
            task_name,
            progress,
        } => {
            Renderer::render_completing(fb, &task_name, progress);
        }
        RenderCommand::History {
            task_name,
            selected,
        } => {
            let entries: alloc::vec::Vec<HistoryDisplayEntry> = nav
                .ctx
                .history
                .iter()
                .map(|h| HistoryDisplayEntry {
                    completed_at: h.formatted_date(),
                    days_since_last: h.days_since_last,
                })
                .collect();
            Renderer::render_history(fb, &task_name, &entries, selected);
        }
        RenderCommand::Settings {
            selected,
            screen_timeout_enabled,
        } => {
            Renderer::render_settings(fb, selected, screen_timeout_enabled);
        }
        RenderCommand::QrCode { ref wifi_mode, ref url } => {
            Renderer::render_qr_code(fb, wifi_mode, url);
        }
        RenderCommand::ResetWifiConfirm { confirmed } => {
            Renderer::render_reset_wifi_confirm(fb, confirmed);
        }
    }
}

/// Flush framebuffer to the hardware display
fn flush_to_display(
    display: &mut impl embedded_graphics_core::draw_target::DrawTarget<Color = embedded_graphics_core::pixelcolor::Rgb565>,
    fb: &FrameBuffer,
) {
    use embedded_graphics_core::geometry::Point;
    use embedded_graphics_core::pixelcolor::Rgb565;
    use embedded_graphics_core::Pixel;

    // Write all pixels from framebuffer to display
    let pixels = fb.as_raw().iter().enumerate().map(|(i, &raw)| {
        let x = (i as u32) % config::DISPLAY_WIDTH;
        let y = (i as u32) / config::DISPLAY_WIDTH;
        Pixel(Point::new(x as i32, y as i32), Rgb565::from(embedded_graphics_core::pixelcolor::raw::RawU16::new(raw)))
    });

    let _ = display.draw_iter(pixels);
}
