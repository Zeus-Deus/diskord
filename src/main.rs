use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, prelude::*, widgets::*, Terminal};
use std::io::{stdout, Result};
use std::time::Duration;

pub mod scanner;
pub mod system;
pub mod theme;

#[derive(PartialEq, Clone, Copy)]
enum ActiveTab {
    System,
    Developer,
    Apps,
    DeepScanner,
    SessionTrash,
}

struct App {
    theme: theme::OmarchyTheme,
    should_quit: bool,
    disks: Vec<system::DiskUsage>,

    active_tab: ActiveTab,

    // System Junk Targets
    pacman_cache_size: u64,
    yay_cache_size: u64,
    journal_size: u64,
    trash_size: u64,
    orphaned_size: u64,
    orphaned_count: usize,

    clean_pacman: bool,
    clean_yay: bool,
    clean_journal: bool,
    clean_trash: bool,
    clean_orphaned: bool,

    // Dev Tools Targets
    docker_size: u64,
    cargo_size: u64,
    npm_size: u64,

    clean_docker: bool,
    clean_cargo: bool,
    clean_npm: bool,

    // Apps Targets (Read-only for now, just to show sizes, or we could add wipe buttons)
    steam_size: u64,
    flatpak_size: u64,

    // UI state
    system_index: usize,
    dev_index: usize,
    apps_index: usize,
    scanner_index: usize,

    // Scanner State
    current_scan_path: std::path::PathBuf,
    scan_results: Vec<scanner::DirEntry>,
    selected_paths: std::collections::HashSet<std::path::PathBuf>,

    // Session Trash State
    trashed_items: Vec<system::TrashedItem>,
    session_trash_index: usize,
    show_root_warning: bool,
}

impl App {
    fn new() -> Self {
        let orphaned = system::get_orphaned_packages();
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("~"));
        let initial_scan = scanner::scan_directory(&home);

        Self {
            theme: theme::OmarchyTheme::load(),
            should_quit: false,
            disks: system::get_disks(),

            active_tab: ActiveTab::System,

            pacman_cache_size: system::get_pacman_cache_size(),
            yay_cache_size: system::get_yay_cache_size(),
            journal_size: system::get_journal_size(),
            trash_size: system::get_trash_size(),
            orphaned_size: orphaned.0,
            orphaned_count: orphaned.1,

            clean_pacman: false,
            clean_yay: false,
            clean_journal: false,
            clean_trash: false,
            clean_orphaned: false,

            docker_size: system::get_docker_size(),
            cargo_size: system::get_cargo_cache_size(),
            npm_size: system::get_npm_cache_size(),

            clean_docker: false,
            clean_cargo: false,
            clean_npm: false,

            steam_size: system::get_steam_size(),
            flatpak_size: system::get_flatpak_size(),

            system_index: 0,
            dev_index: 0,
            apps_index: 0,
            scanner_index: 0,

            current_scan_path: home,
            scan_results: initial_scan,
            selected_paths: std::collections::HashSet::new(),

            trashed_items: Vec::new(),
            session_trash_index: 0,
            show_root_warning: false,
        }
    }

    fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::System => ActiveTab::Developer,
            ActiveTab::Developer => ActiveTab::Apps,
            ActiveTab::Apps => ActiveTab::DeepScanner,
            ActiveTab::DeepScanner => ActiveTab::SessionTrash,
            ActiveTab::SessionTrash => ActiveTab::System,
        };
    }

    fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::System => ActiveTab::SessionTrash,
            ActiveTab::Developer => ActiveTab::System,
            ActiveTab::Apps => ActiveTab::Developer,
            ActiveTab::DeepScanner => ActiveTab::Apps,
            ActiveTab::SessionTrash => ActiveTab::DeepScanner,
        };
    }

    fn next_item(&mut self) {
        match self.active_tab {
            ActiveTab::System => self.system_index = (self.system_index + 1) % 5,
            ActiveTab::Developer => self.dev_index = (self.dev_index + 1) % 3,
            ActiveTab::Apps => self.apps_index = (self.apps_index + 1) % 2,
            ActiveTab::DeepScanner => {
                if !self.scan_results.is_empty() {
                    self.scanner_index = (self.scanner_index + 1) % self.scan_results.len();
                }
            }
            ActiveTab::SessionTrash => {
                if !self.trashed_items.is_empty() {
                    self.session_trash_index =
                        (self.session_trash_index + 1) % self.trashed_items.len();
                }
            }
        }
    }

    fn prev_item(&mut self) {
        match self.active_tab {
            ActiveTab::System => {
                if self.system_index > 0 {
                    self.system_index -= 1;
                } else {
                    self.system_index = 4;
                }
            }
            ActiveTab::Developer => {
                if self.dev_index > 0 {
                    self.dev_index -= 1;
                } else {
                    self.dev_index = 2;
                }
            }
            ActiveTab::Apps => {
                if self.apps_index > 0 {
                    self.apps_index -= 1;
                } else {
                    self.apps_index = 1;
                }
            }
            ActiveTab::DeepScanner => {
                if !self.scan_results.is_empty() {
                    if self.scanner_index > 0 {
                        self.scanner_index -= 1;
                    } else {
                        self.scanner_index = self.scan_results.len() - 1;
                    }
                }
            }
            ActiveTab::SessionTrash => {
                if !self.trashed_items.is_empty() {
                    if self.session_trash_index > 0 {
                        self.session_trash_index -= 1;
                    } else {
                        self.session_trash_index = self.trashed_items.len() - 1;
                    }
                }
            }
        }
    }

    fn toggle_selection(&mut self) {
        match self.active_tab {
            ActiveTab::System => match self.system_index {
                0 => self.clean_pacman = !self.clean_pacman,
                1 => self.clean_yay = !self.clean_yay,
                2 => self.clean_journal = !self.clean_journal,
                3 => self.clean_trash = !self.clean_trash,
                4 => self.clean_orphaned = !self.clean_orphaned,
                _ => {}
            },
            ActiveTab::Developer => match self.dev_index {
                0 => self.clean_docker = !self.clean_docker,
                1 => self.clean_cargo = !self.clean_cargo,
                2 => self.clean_npm = !self.clean_npm,
                _ => {}
            },
            ActiveTab::DeepScanner => {
                if !self.scan_results.is_empty() {
                    let path = self.scan_results[self.scanner_index].path.clone();
                    if self.selected_paths.contains(&path) {
                        self.selected_paths.remove(&path);
                    } else {
                        self.selected_paths.insert(path);
                    }
                }
            }
            _ => {}
        }
    }

    fn drill_down(&mut self) {
        if self.scan_results.is_empty() {
            return;
        }
        let selected = &self.scan_results[self.scanner_index];
        if selected.is_dir {
            self.current_scan_path = selected.path.clone();
            self.scan_results = scanner::scan_directory(&self.current_scan_path);
            self.scanner_index = 0;
        }
    }

    fn drill_up(&mut self) {
        if let Some(parent) = self.current_scan_path.parent() {
            self.current_scan_path = parent.to_path_buf();
            self.scan_results = scanner::scan_directory(&self.current_scan_path);
            self.scanner_index = 0;
        }
    }

    fn execute_clean(&mut self) {
        if self.active_tab == ActiveTab::DeepScanner {
            self.execute_deep_scanner_trash();
            return;
        }
        if self.active_tab == ActiveTab::SessionTrash {
            self.execute_session_trash_delete();
            return;
        }

        // System Junk
        if self.clean_pacman {
            if system::clean_pacman_cache() {
                self.pacman_cache_size = system::get_pacman_cache_size();
                self.clean_pacman = false;
            }
        }
        if self.clean_yay {
            if system::clean_yay_cache() {
                self.yay_cache_size = system::get_yay_cache_size();
                self.clean_yay = false;
            }
        }
        if self.clean_journal {
            if system::vacuum_journal() {
                self.journal_size = system::get_journal_size();
                self.clean_journal = false;
            }
        }
        if self.clean_trash {
            if system::empty_trash() {
                self.trash_size = system::get_trash_size();
                self.clean_trash = false;
            }
        }
        if self.clean_orphaned {
            if system::clean_orphaned_packages() {
                let orphaned = system::get_orphaned_packages();
                self.orphaned_size = orphaned.0;
                self.orphaned_count = orphaned.1;
                self.clean_orphaned = false;
            }
        }

        // Dev Tools
        if self.clean_docker {
            if system::clean_docker() {
                self.docker_size = system::get_docker_size();
                self.clean_docker = false;
            }
        }
        if self.clean_cargo {
            if system::clean_cargo_cache() {
                self.cargo_size = system::get_cargo_cache_size();
                self.clean_cargo = false;
            }
        }
        if self.clean_npm {
            if system::clean_npm_cache() {
                self.npm_size = system::get_npm_cache_size();
                self.clean_npm = false;
            }
        }

        self.disks = system::get_disks();
    }

    fn execute_deep_scanner_trash(&mut self) {
        if self.selected_paths.is_empty() {
            return;
        }

        let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("~"));
        let mut has_root_files = false;
        for path in &self.selected_paths {
            if !path.starts_with(&home_dir) {
                has_root_files = true;
                break;
            }
        }

        if has_root_files && !self.show_root_warning {
            self.show_root_warning = true;
            return;
        }

        // Proceed with trashing/deleting
        let paths: Vec<_> = self.selected_paths.drain().collect();
        for path in paths {
            if let Ok(item) = system::move_to_trash(&path) {
                self.trashed_items.push(item);
            }
        }

        self.show_root_warning = false;
        self.scan_results = scanner::scan_directory(&self.current_scan_path);
        self.scanner_index = 0;
    }

    fn execute_session_trash_delete(&mut self) {
        if self.trashed_items.is_empty() {
            return;
        }
        let item = self.trashed_items.remove(self.session_trash_index);
        let _ = system::perm_delete_trash_item(&item);

        if self.session_trash_index >= self.trashed_items.len() && self.session_trash_index > 0 {
            self.session_trash_index -= 1;
        }
    }

    fn execute_undo_trash(&mut self) {
        if self.active_tab != ActiveTab::SessionTrash || self.trashed_items.is_empty() {
            return;
        }
        let item = self.trashed_items.remove(self.session_trash_index);
        let _ = system::restore_trash_item(&item);

        if self.session_trash_index >= self.trashed_items.len() && self.session_trash_index > 0 {
            self.session_trash_index -= 1;
        }
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();

    while !app.should_quit {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Esc => {
                            if app.show_root_warning {
                                app.show_root_warning = false;
                            } else {
                                app.should_quit = true;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                        KeyCode::Up | KeyCode::Char('k') => app.prev_item(),
                        KeyCode::Right | KeyCode::Char('l') => {
                            if app.active_tab == ActiveTab::DeepScanner {
                                app.drill_down();
                            } else {
                                app.next_tab();
                            }
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            if app.active_tab == ActiveTab::DeepScanner {
                                app.drill_up();
                            } else {
                                app.prev_tab();
                            }
                        }
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::BackTab => app.prev_tab(),
                        KeyCode::Char(' ') => app.toggle_selection(),
                        KeyCode::Enter => app.execute_clean(),
                        KeyCode::Char('u') => app.execute_undo_trash(),
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    let block = Block::default()
        .title(" Diskord: Storage Manager ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.accent))
        .bg(app.theme.background);

    let inner_area = block.inner(size);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4), // Disks overview
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    // 1. Dashboard / Disks
    for disk in &app.disks {
        let percent = if disk.total_space > 0 {
            (disk.used_space as f64 / disk.total_space as f64 * 100.0) as u16
        } else {
            0
        };

        let title = format!(
            " {} ({} / {}) ",
            disk.mount_point,
            system::format_bytes(disk.used_space),
            system::format_bytes(disk.total_space)
        );

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.color8)),
            )
            .gauge_style(Style::default().fg(app.theme.color2).bg(app.theme.color0))
            .percent(percent);

        f.render_widget(gauge, chunks[0]);
        break;
    }

    // 2. Tabs
    let tab_titles = vec![
        "System Junk",
        "Developer Tools",
        "Apps & Games",
        "Deep Scanner",
        "Session Trash",
    ];
    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.color8)),
        )
        .highlight_style(
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .divider(ratatui::symbols::line::VERTICAL)
        .select(app.active_tab as usize);

    f.render_widget(tabs, chunks[1]);

    // 3. Content
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.color8));
    let content_inner = content_block.inner(chunks[2]);
    f.render_widget(content_block, chunks[2]);

    match app.active_tab {
        ActiveTab::System => render_system_tab(f, app, content_inner),
        ActiveTab::Developer => render_dev_tab(f, app, content_inner),
        ActiveTab::Apps => render_apps_tab(f, app, content_inner),
        ActiveTab::DeepScanner => render_deep_scan_tab(f, app, content_inner),
        ActiveTab::SessionTrash => render_session_trash_tab(f, app, content_inner),
    }

    // 4. Footer
    let footer_text = match app.active_tab {
        ActiveTab::DeepScanner => {
            if app.show_root_warning {
                " [Enter] Confirm PERMANENT DELETE   [Esc] Cancel"
            } else {
                " [Space] Toggle Select   [Enter] Move Selected to Trash   [h/l] Navigate Folder"
            }
        }
        ActiveTab::SessionTrash => {
            " [u] Undo/Restore   [Enter] Permanently Delete Selected   [h/l, Tab] Switch Tabs"
        }
        _ => {
            " [h/l, Tab] Switch Tabs   [j/k] Navigate   [Space] Select   [Enter] Clean   [q/Esc] Quit"
        }
    };
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(app.theme.color7)
                .bg(app.theme.background),
        );
    f.render_widget(footer, chunks[3]);
}

fn render_system_tab(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let list_items = vec![
        format_target(
            "Pacman Cache (Requires pkexec)",
            app.pacman_cache_size,
            app.clean_pacman,
        ),
        format_target("Yay Cache", app.yay_cache_size, app.clean_yay),
        format_target(
            "Systemd Journals (Requires pkexec)",
            app.journal_size,
            app.clean_journal,
        ),
        format_target("User Trash", app.trash_size, app.clean_trash),
        format_target(
            &format!("Orphaned Packages ({})", app.orphaned_count),
            app.orphaned_size,
            app.clean_orphaned,
        ),
    ];

    let mut items = vec![];
    for (i, text) in list_items.into_iter().enumerate() {
        let style = if i == app.system_index {
            Style::default()
                .fg(app.theme.background)
                .bg(app.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.foreground)
        };
        items.push(ListItem::new(text).style(style));
    }

    let list = List::new(items);
    f.render_widget(list, area);
}

fn render_dev_tab(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let list_items = vec![
        format_target(
            "Docker System Caches (Requires pkexec)",
            app.docker_size,
            app.clean_docker,
        ),
        format_target("Cargo Cache (~/.cargo)", app.cargo_size, app.clean_cargo),
        format_target("NPM Cache (~/.npm/_cacache)", app.npm_size, app.clean_npm),
    ];

    let mut items = vec![];
    for (i, text) in list_items.into_iter().enumerate() {
        let style = if i == app.dev_index {
            Style::default()
                .fg(app.theme.background)
                .bg(app.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.foreground)
        };
        items.push(ListItem::new(text).style(style));
    }

    let list = List::new(items);
    f.render_widget(list, area);
}

fn render_apps_tab(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let list_items = vec![
        format_target(
            "Steam Library (~/.local/share/Steam)",
            app.steam_size,
            false,
        ),
        format_target("Flatpak Apps (/var/lib/flatpak)", app.flatpak_size, false),
    ];

    let mut items = vec![];
    for (i, text) in list_items.into_iter().enumerate() {
        let style = if i == app.apps_index {
            Style::default()
                .fg(app.theme.background)
                .bg(app.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.foreground)
        };
        items.push(ListItem::new(text).style(style));
    }

    let list = List::new(items);
    f.render_widget(list, area);
}

fn render_deep_scan_tab(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.show_root_warning {
        let p = Paragraph::new("\n\nWarning: System files selected.\nTrashing outside your Home directory is not supported.\nThese items will be PERMANENTLY DELETED.\n\nPress [Enter] to confirm permanent deletion, or [Esc] to cancel.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(ratatui::style::Color::Red));
        f.render_widget(p, area);
        return;
    }

    let mut items = vec![];

    for entry in app.scan_results.iter() {
        let is_selected = app.selected_paths.contains(&entry.path);
        let checkbox = if is_selected { "[X]" } else { "[ ]" };
        let prefix = if entry.is_dir { "[DIR]" } else { "[FILE]" };
        let size_str = system::format_bytes(entry.size);

        let max_name_len = (area.width as usize).saturating_sub(30);
        let mut display_name = entry.name.clone();
        if display_name.len() > max_name_len && max_name_len > 3 {
            display_name.truncate(max_name_len - 3);
            display_name.push_str("...");
        }

        let text = format!(
            " {} {} {:<width$} {}",
            checkbox,
            prefix,
            display_name,
            size_str,
            width = max_name_len
        );
        items.push(ListItem::new(text).style(Style::default().fg(app.theme.foreground)));
    }

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.scanner_index));

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" Path: {} ", app.current_scan_path.display()))
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(app.theme.color8)),
        )
        .highlight_style(
            Style::default()
                .fg(app.theme.background)
                .bg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut state);
}

fn render_session_trash_tab(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.trashed_items.is_empty() {
        let p = Paragraph::new(
            "\n\nSession Trash is empty.\n(Items trashed in the Deep Scanner will appear here)",
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(app.theme.foreground));
        f.render_widget(p, area);
        return;
    }

    let mut items = vec![];

    for item in app.trashed_items.iter() {
        let max_name_len = (area.width as usize).saturating_sub(10);
        let mut display_name = item.original_path.to_string_lossy().into_owned();
        if display_name.len() > max_name_len && max_name_len > 3 {
            let overflow = display_name.len() - max_name_len + 3;
            display_name.replace_range(0..overflow, "...");
        }

        let text = format!(" {} ", display_name);
        let mut style = Style::default().fg(app.theme.foreground);
        if item.is_root {
            style = style
                .add_modifier(Modifier::CROSSED_OUT)
                .fg(ratatui::style::Color::Red);
        }

        items.push(ListItem::new(text).style(style));
    }

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.session_trash_index));

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Session Trash (Restorable) ")
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(app.theme.color8)),
        )
        .highlight_style(
            Style::default()
                .fg(app.theme.background)
                .bg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut state);
}

fn format_target(name: &str, size: u64, selected: bool) -> String {
    let checkbox = if selected { "[X]" } else { "[ ]" };
    let size_str = system::format_bytes(size);
    // basic padding
    format!(" {} {:<40} {} ", checkbox, name, size_str)
}
