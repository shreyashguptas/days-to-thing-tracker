/// Days Tracker Kiosk - ESP32-C6 Firmware
///
/// Standalone embedded firmware for XIAO ESP32-C6 with:
/// - ST7735 160x128 TFT display via SPI
/// - KY-040 rotary encoder for navigation
/// - WiFi SoftAP for phone-based task management
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

use chrono::NaiveDate;
use display_interface_spi::SPIInterface;
use mipidsi::models::ST7735s;
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::Builder;

mod config;
mod display;
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
use http_server::{SharedStorage, SharedTime};
use models::{HistoryDisplayEntry, TaskDisplayData};
use renderer::Renderer;
use storage::Storage;
use views::{RenderCommand, ViewNavigator, ViewState};

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
        .invert_colors(ColorInversion::Normal)
        .orientation(Orientation::new())
        .init(&mut FreeRtos)
        .unwrap();

    log::info!("Display initialized");

    // Create framebuffer
    let mut fb = FrameBuffer::new();

    // Show splash screen
    Renderer::render_connecting(&mut fb, "Starting...");
    flush_to_display(&mut hw_display, &fb);

    // === Initialize Encoder ===
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

    // === Start WiFi SoftAP ===
    log::info!("Starting WiFi SoftAP...");
    Renderer::render_connecting(&mut fb, "Starting WiFi...");
    flush_to_display(&mut hw_display, &fb);

    let _wifi = wifi::init_softap(peripherals.modem, sysloop, nvs).unwrap();
    log::info!("WiFi SoftAP ready");

    // === Mount Storage ===
    log::info!("Mounting storage...");

    // Mount LittleFS partition
    #[cfg(feature = "littlefs")]
    {
        // LittleFS mount code would go here
        // For now we use the ESP-IDF SPIFFS VFS
    }

    // Use SPIFFS for storage (configured as LittleFS-compatible in partitions)
    let _spiffs = unsafe { esp_idf_svc::fs::spiffs::Spiffs::new(config::STORAGE_PARTITION) };

    let storage = Arc::new(Mutex::new(Storage::new(
        config::TASKS_FILE,
        config::HISTORY_FILE,
    )));

    // === Shared time source (synced from phone) ===
    let time_source: SharedTime = Arc::new(Mutex::new(None));

    // === Start HTTP Server (in current thread context, runs async) ===
    log::info!("Starting HTTP server...");
    let _server = http_server::start_server(storage.clone(), time_source.clone()).unwrap();
    log::info!("HTTP server ready on port {}", config::HTTP_PORT);

    // === Initialize View Navigator ===
    let mut nav = ViewNavigator::new();

    // Load initial data
    {
        let s = storage.lock().unwrap();
        let today = get_today(&time_source);
        let counts = s.get_task_counts(today);
        nav.set_task_counts(counts);
        let tasks = s.get_all_tasks(true);
        nav.set_tasks(tasks);
    }

    // Show QR code if there are no tasks
    if nav.ctx.tasks.is_empty() {
        nav.ctx.state = ViewState::Empty;
    }

    // === Main Event Loop ===
    log::info!("Ready. Entering main loop.");

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
                handle_action(action, &mut nav, &storage, &time_source);
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
) {
    let today = get_today(time_source);

    match action {
        "complete" => {
            if let Some(task) = nav.ctx.current_task() {
                let task_id = task.id;
                let now_iso = get_now_iso(time_source);

                // Run completion animation
                // (In embedded, we handle this inline since we own the display)
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
        RenderCommand::Empty => {
            Renderer::render_empty(fb);
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
            let s = storage.lock().unwrap();
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
        RenderCommand::QrCode => {
            let qr_data = wifi::wifi_qr_string();
            Renderer::render_qr_code(fb, &qr_data);
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

/// Get today's date from the shared time source
fn get_today(time: &SharedTime) -> NaiveDate {
    let secs = time.lock().unwrap().unwrap_or(0);
    if secs > 0 {
        chrono::DateTime::from_timestamp(secs, 0)
            .map(|dt| dt.date_naive())
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
    } else {
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
