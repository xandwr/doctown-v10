use minui::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::process::{Child, Command};
use std::path::PathBuf;

#[derive(Debug, Clone)]
enum MenuItem {
    Launch,
    Restart,
    LaunchService,
    Configuration,
    Quit,
}

impl MenuItem {
    fn label(&self) -> &str {
        match self {
            MenuItem::Launch => "Launch",
            MenuItem::Restart => "Restart",
            MenuItem::LaunchService => "Launch Service",
            MenuItem::Configuration => "Configuration",
            MenuItem::Quit => "Quit",
        }
    }
}

#[derive(Debug, Clone)]
enum SubMenuItem {
    EmbeddingService,
    DocumenterService,
    DoctownMain,
    Database,
    Back,
}

impl SubMenuItem {
    fn label(&self) -> &str {
        match self {
            SubMenuItem::EmbeddingService => "Embedding Service (Python)",
            SubMenuItem::DocumenterService => "Documenter Service (Python)",
            SubMenuItem::DoctownMain => "Doctown Main (Rust)",
            SubMenuItem::Database => "Database",
            SubMenuItem::Back => "← Back to Main Menu",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum ServiceStatus {
    Online,
    Offline,
    Starting,
}

impl ServiceStatus {
    fn label(&self) -> &str {
        match self {
            ServiceStatus::Online => "●",
            ServiceStatus::Offline => "○",
            ServiceStatus::Starting => "◐",
        }
    }

    fn color(&self) -> Color {
        match self {
            ServiceStatus::Online => Color::Green,
            ServiceStatus::Offline => Color::Red,
            ServiceStatus::Starting => Color::Yellow,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ServiceType {
    PythonEmbedding,
    PythonDocumenter,
    RustMain,
    Database,
}

#[derive(Debug, Clone)]
struct Service {
    name: String,
    port: u16,
    status: ServiceStatus,
    endpoint: String,
    service_type: ServiceType,
}

#[allow(dead_code)]
struct ServiceProcess {
    child: Child,
    service_type: ServiceType,
}

impl Service {
    fn new(name: &str, port: u16, endpoint: &str, service_type: ServiceType) -> Self {
        Service {
            name: name.to_string(),
            port,
            status: ServiceStatus::Offline,
            endpoint: endpoint.to_string(),
            service_type,
        }
    }
}

#[derive(Debug, Clone)]
enum MenuMode {
    Main,
    ServiceSubmenu,
}

struct MenuState {
    selected: usize,
    items: Vec<MenuItem>,
    sub_items: Vec<SubMenuItem>,
    services: Arc<Mutex<Vec<Service>>>,
    processes: Arc<Mutex<Vec<ServiceProcess>>>,
    running: bool,
    mode: MenuMode,
}

fn main() -> minui::Result<()> {
    let services = Arc::new(Mutex::new(vec![
        Service::new("Embedding Service", 18115, "http://localhost:18115/health", ServiceType::PythonEmbedding),
        Service::new("Documenter Service", 18116, "http://localhost:18116/health", ServiceType::PythonDocumenter),
        Service::new("Doctown Main", 3000, "http://localhost:3000/health", ServiceType::RustMain),
        Service::new("Database", 5432, "http://localhost:5432", ServiceType::Database),
    ]));

    let processes = Arc::new(Mutex::new(Vec::<ServiceProcess>::new()));

    let state = MenuState {
        selected: 0,
        items: vec![
            MenuItem::Launch,
            MenuItem::Restart,
            MenuItem::LaunchService,
            MenuItem::Configuration,
            MenuItem::Quit,
        ],
        sub_items: vec![
            SubMenuItem::EmbeddingService,
            SubMenuItem::DocumenterService,
            SubMenuItem::DoctownMain,
            SubMenuItem::Database,
            SubMenuItem::Back,
        ],
        services: Arc::clone(&services),
        processes: Arc::clone(&processes),
        running: true,
        mode: MenuMode::Main,
    };

    // Spawn background task for status polling
    let services_clone = Arc::clone(&services);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            loop {
                check_services_status(&services_clone).await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    });

    let mut app = App::new(state)?;

    app.run(
        |state, event| {
            match event {
                Event::Character('q') | Event::Escape => match state.mode {
                    MenuMode::Main => state.running = false,
                    MenuMode::ServiceSubmenu => {
                        state.mode = MenuMode::Main;
                        state.selected = 0;
                    }
                },
                Event::KeyUp => {
                    if state.selected > 0 {
                        state.selected -= 1;
                    }
                }
                Event::KeyDown => {
                    let max_items = match state.mode {
                        MenuMode::Main => state.items.len(),
                        MenuMode::ServiceSubmenu => state.sub_items.len(),
                    };
                    if state.selected < max_items - 1 {
                        state.selected += 1;
                    }
                }
                Event::Enter => match state.mode {
                    MenuMode::Main => {
                        let selected_item = state.items[state.selected].clone();
                        handle_selection(&selected_item, state);
                    }
                    MenuMode::ServiceSubmenu => {
                        let selected_item = state.sub_items[state.selected].clone();
                        handle_submenu_selection(&selected_item, state);
                    }
                },
                _ => {}
            }
            state.running
        },
        |state, window| {
            let (width, height) = window.get_size();

            // Calculate panel widths (60% menu, 40% status)
            let menu_width = (width as f32 * 0.6) as u16;
            let status_width = width.saturating_sub(menu_width);

            // Create left panel (menu)
            let mut menu_panel = Container::new(0, 0, menu_width, height)
                .with_padding(2)
                .with_border(BorderChars::double_line())
                .with_border_color(Color::Cyan);

            // ASCII Art Banner
            let banner = FigletText::standard("DocTown")
                .unwrap_or_else(|_| FigletText::standard("").unwrap())
                .with_color(ColorPair::new(Color::Cyan, Color::Transparent))
                .with_alignment(Alignment::Center);

            menu_panel = menu_panel.add_child(banner);
            menu_panel = menu_panel.add_child(Label::new(""));

            // Menu title
            let menu_title = match state.mode {
                MenuMode::Main => "MAIN MENU",
                MenuMode::ServiceSubmenu => "LAUNCH SERVICE",
            };
            menu_panel = menu_panel.add_child(
                Label::new(menu_title)
                    .with_text_color(Color::Yellow)
                    .with_alignment(Alignment::Center),
            );
            menu_panel = menu_panel.add_child(Label::new(""));

            // Menu items
            let items_to_display: Vec<String> = match state.mode {
                MenuMode::Main => state.items.iter().map(|i| i.label().to_string()).collect(),
                MenuMode::ServiceSubmenu => state
                    .sub_items
                    .iter()
                    .map(|i| i.label().to_string())
                    .collect(),
            };

            for (idx, item_label) in items_to_display.iter().enumerate() {
                let label_text = if idx == state.selected {
                    format!("▶  {}  ◀", item_label)
                } else {
                    format!("   {}   ", item_label)
                };

                let color = if idx == state.selected {
                    Color::Yellow
                } else {
                    Color::White
                };

                let item = Label::new(&label_text)
                    .with_text_color(color)
                    .with_alignment(Alignment::Center);

                menu_panel = menu_panel.add_child(item);
            }

            // Add spacing
            menu_panel = menu_panel.add_child(Label::new(""));

            // Footer with controls
            let footer = Label::new("↑/↓: Navigate  |  Enter: Select  |  Q/Esc: Back/Quit")
                .with_text_color(Color::DarkGray)
                .with_alignment(Alignment::Center);

            menu_panel = menu_panel.add_child(footer);

            // Create right panel (status)
            let mut status_panel = Container::new(menu_width, 0, status_width, height)
                .with_padding(2)
                .with_border(BorderChars::double_line())
                .with_border_color(Color::Magenta);

            // Status panel title
            status_panel = status_panel.add_child(
                Label::new("SERVICE STATUS")
                    .with_text_color(Color::Magenta)
                    .with_alignment(Alignment::Center),
            );
            status_panel = status_panel.add_child(Label::new(""));

            // Service status list
            let services = state.services.lock().unwrap();
            for service in services.iter() {
                let status_line = format!(
                    "{} {} :{}",
                    service.status.label(),
                    service.name,
                    service.port
                );

                status_panel = status_panel.add_child(
                    Label::new(&status_line)
                        .with_text_color(service.status.color())
                        .with_alignment(Alignment::Left),
                );
            }

            status_panel = status_panel.add_child(Label::new(""));
            status_panel = status_panel.add_child(
                Label::new("Updated every 1s")
                    .with_text_color(Color::DarkGray)
                    .with_alignment(Alignment::Center),
            );

            // Draw both panels directly
            menu_panel.draw(window)?;
            status_panel.draw(window)?;

            Ok(())
        },
    )?;

    Ok(())
}

async fn check_services_status(services: &Arc<Mutex<Vec<Service>>>) {
    let mut services_guard = services.lock().unwrap();

    for service in services_guard.iter_mut() {
        // Simple TCP port check for database, HTTP check for others
        let is_online = if service.port == 5432 {
            check_tcp_port(service.port).await
        } else {
            check_http_endpoint(&service.endpoint).await
        };

        service.status = if is_online {
            ServiceStatus::Online
        } else {
            ServiceStatus::Offline
        };
    }
}

async fn check_http_endpoint(endpoint: &str) -> bool {
    match reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
    {
        Ok(client) => match client.get(endpoint).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

async fn check_tcp_port(port: u16) -> bool {
    use tokio::net::TcpStream;
    TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .is_ok()
}

fn get_project_root() -> PathBuf {
    // Assuming harness is in doctown-v10/doctown-harness
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from(".."))
}

fn launch_service(service_type: ServiceType, processes: &Arc<Mutex<Vec<ServiceProcess>>>) -> std::result::Result<(), String> {
    let project_root = get_project_root();

    let child = match service_type {
        ServiceType::PythonEmbedding => {
            let python_path = project_root.join("python").join("embedding");
            Command::new("python")
                .arg("server.py")
                .current_dir(&python_path)
                .spawn()
                .map_err(|e| format!("Failed to launch embedding service: {}. Make sure you're in the correct directory and Python is installed.", e))?
        }
        ServiceType::PythonDocumenter => {
            let python_path = project_root.join("python").join("documenter");
            Command::new("python")
                .arg("server.py")
                .current_dir(&python_path)
                .spawn()
                .map_err(|e| format!("Failed to launch documenter service: {}. Make sure you're in the correct directory and Python is installed.", e))?
        }
        ServiceType::RustMain => {
            Command::new("cargo")
                .arg("run")
                .arg("--release")
                .current_dir(&project_root)
                .spawn()
                .map_err(|e| format!("Failed to launch Doctown main: {}", e))?
        }
        ServiceType::Database => {
            return Err("Database management not implemented yet".to_string());
        }
    };

    let mut procs = processes.lock().unwrap();
    procs.push(ServiceProcess {
        child,
        service_type: service_type.clone(),
    });

    Ok(())
}

fn stop_all_services(processes: &Arc<Mutex<Vec<ServiceProcess>>>) {
    let mut procs = processes.lock().unwrap();
    for proc in procs.iter_mut() {
        let _ = proc.child.kill();
    }
    procs.clear();
}

fn handle_selection(item: &MenuItem, state: &mut MenuState) {
    match item {
        MenuItem::Launch => {
            // Launch all services
            let services = state.services.lock().unwrap().clone();
            for service in services.iter() {
                if let Err(e) = launch_service(service.service_type.clone(), &state.processes) {
                    eprintln!("Failed to launch {}: {}", service.name, e);
                }
            }
        }
        MenuItem::Restart => {
            // Restart all services
            stop_all_services(&state.processes);
            std::thread::sleep(Duration::from_secs(1));
            let services = state.services.lock().unwrap().clone();
            for service in services.iter() {
                if let Err(e) = launch_service(service.service_type.clone(), &state.processes) {
                    eprintln!("Failed to restart {}: {}", service.name, e);
                }
            }
        }
        MenuItem::LaunchService => {
            state.mode = MenuMode::ServiceSubmenu;
            state.selected = 0;
        }
        MenuItem::Configuration => {
            // TODO: Show configuration screen
        }
        MenuItem::Quit => {
            stop_all_services(&state.processes);
            state.running = false;
        }
    }
}

fn handle_submenu_selection(item: &SubMenuItem, state: &mut MenuState) {
    match item {
        SubMenuItem::EmbeddingService => {
            if let Err(e) = launch_service(ServiceType::PythonEmbedding, &state.processes) {
                eprintln!("Failed to launch embedding service: {}", e);
            }
        }
        SubMenuItem::DocumenterService => {
            if let Err(e) = launch_service(ServiceType::PythonDocumenter, &state.processes) {
                eprintln!("Failed to launch documenter service: {}", e);
            }
        }
        SubMenuItem::DoctownMain => {
            if let Err(e) = launch_service(ServiceType::RustMain, &state.processes) {
                eprintln!("Failed to launch Doctown main: {}", e);
            }
        }
        SubMenuItem::Database => {
            if let Err(e) = launch_service(ServiceType::Database, &state.processes) {
                eprintln!("Failed to launch database: {}", e);
            }
        }
        SubMenuItem::Back => {
            state.mode = MenuMode::Main;
            state.selected = 0;
        }
    }
}
