use minui::prelude::*;

#[derive(Debug, Clone)]
enum MenuItem {
    StartPipeline,
    ViewStatus,
    Configuration,
    Exit,
}

impl MenuItem {
    fn label(&self) -> &str {
        match self {
            MenuItem::StartPipeline => "Start Documentation Pipeline",
            MenuItem::ViewStatus => "View Pipeline Status",
            MenuItem::Configuration => "Configuration",
            MenuItem::Exit => "Exit",
        }
    }
}

struct MenuState {
    selected: usize,
    items: Vec<MenuItem>,
    running: bool,
}

fn main() -> minui::Result<()> {
    let state = MenuState {
        selected: 0,
        items: vec![
            MenuItem::StartPipeline,
            MenuItem::ViewStatus,
            MenuItem::Configuration,
            MenuItem::Exit,
        ],
        running: true,
    };

    let mut app = App::new(state)?;

    app.run(
        |state, event| {
            match event {
                Event::Character('q') | Event::Escape => state.running = false,
                Event::KeyUp => {
                    if state.selected > 0 {
                        state.selected -= 1;
                    }
                }
                Event::KeyDown => {
                    if state.selected < state.items.len() - 1 {
                        state.selected += 1;
                    }
                }
                Event::Enter => {
                    let selected_item = state.items[state.selected].clone();
                    handle_selection(&selected_item, state);
                }
                _ => {}
            }
            state.running
        },
        |state, window| {
            let (width, height) = window.get_size();

            // Create main container
            let mut main_container = Container::fullscreen_with_size(width, height);

            // Create centered content container
            let mut content = Container::vertical()
                .with_padding(2)
                .with_border(BorderChars::double_line())
                .with_border_color(Color::Cyan);

            // ASCII Art Banner using figlet
            let banner = FigletText::standard("DocTown")
                .unwrap_or_else(|_| {
                    // Fallback to a simple label if figlet fails
                    FigletText::standard("").unwrap()
                })
                .with_color(ColorPair::new(Color::Cyan, Color::Transparent))
                .with_alignment(Alignment::Center);

            content = content.add_child(banner);

            // Add spacing
            content = content.add_child(Label::new(""));

            // Menu items
            for (idx, item) in state.items.iter().enumerate() {
                let label_text = if idx == state.selected {
                    format!("▶  {}  ◀", item.label())
                } else {
                    format!("   {}   ", item.label())
                };

                let color = if idx == state.selected {
                    Color::Yellow
                } else {
                    Color::White
                };

                let item_label = Label::new(&label_text)
                    .with_text_color(color)
                    .with_alignment(Alignment::Center);

                content = content.add_child(item_label);
            }

            // Add spacing
            content = content.add_child(Label::new(""));

            // Footer with controls
            let footer = Label::new("↑/↓: Navigate  |  Enter: Select  |  Q/Esc: Quit")
                .with_text_color(Color::DarkGray)
                .with_alignment(Alignment::Center);

            content = content.add_child(footer);

            // Add content to main container with auto-centering
            main_container = main_container
                .add_child(content)
                .with_auto_center();

            main_container.draw(window)?;
            Ok(())
        },
    )?;

    Ok(())
}

fn handle_selection(item: &MenuItem, state: &mut MenuState) {
    match item {
        MenuItem::StartPipeline => {
            // TODO: Launch pipeline
        }
        MenuItem::ViewStatus => {
            // TODO: Show status screen
        }
        MenuItem::Configuration => {
            // TODO: Show configuration screen
        }
        MenuItem::Exit => {
            state.running = false;
        }
    }
}
