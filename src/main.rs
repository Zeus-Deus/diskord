use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, prelude::*, widgets::*, Terminal};
use std::io::{stdout, Result};
use std::time::Duration;

pub mod system;
pub mod theme;

struct App {
    theme: theme::OmarchyTheme,
    should_quit: bool,
    disks: Vec<system::DiskUsage>,

    // Junk targets
    pacman_cache_size: u64,
    yay_cache_size: u64,
    journal_size: u64,
    trash_size: u64,

    // UI state
    selected_index: usize,
    items_count: usize,

    // Checkboxes
    clean_pacman: bool,
    clean_yay: bool,
    clean_journal: bool,
    clean_trash: bool,
}

impl App {
    fn new() -> Self {
        Self {
            theme: theme::OmarchyTheme::load(),
            should_quit: false,
            disks: system::get_disks(),

            pacman_cache_size: system::get_pacman_cache_size(),
            yay_cache_size: system::get_yay_cache_size(),
            journal_size: system::get_journal_size(),
            trash_size: system::get_trash_size(),

            selected_index: 0,
            items_count: 4,

            clean_pacman: false,
            clean_yay: false,
            clean_journal: false,
            clean_trash: false,
        }
    }

    fn next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.items_count;
    }

    fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.items_count - 1;
        }
    }

    fn toggle_selection(&mut self) {
        match self.selected_index {
            0 => self.clean_pacman = !self.clean_pacman,
            1 => self.clean_yay = !self.clean_yay,
            2 => self.clean_journal = !self.clean_journal,
            3 => self.clean_trash = !self.clean_trash,
            _ => {}
        }
    }

    fn execute_clean(&mut self) {
        if self.clean_pacman {
            if system::clean_pacman_cache() {
                self.pacman_cache_size = system::get_pacman_cache_size();
                self.clean_pacman = false;
            } else {
                // If it fails (user hit cancel on prompt), refresh size anyway
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
        // Refresh disks after cleanup
        self.disks = system::get_disks();
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
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char(' ') => app.toggle_selection(),
                        KeyCode::Enter => app.execute_clean(),
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
        .title(" Diskord: Omarchy Storage Manager ")
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
            Constraint::Min(0),    // Junk Targets
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
        // For simplicity, we only show the first major disk (usually /) in this slot right now
        break;
    }

    // 2. System Junk
    let junk_block = Block::default()
        .title(" System Junk ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.color8));

    let junk_inner = junk_block.inner(chunks[1]);
    f.render_widget(junk_block, chunks[1]);

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
    ];

    let mut items = vec![];
    for (i, text) in list_items.into_iter().enumerate() {
        let style = if i == app.selected_index {
            Style::default()
                .fg(app.theme.background)
                .bg(app.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.foreground)
        };
        items.push(ListItem::new(text).style(style));
    }

    let list = List::new(items).highlight_style(
        Style::default()
            .fg(app.theme.background)
            .bg(app.theme.accent),
    );

    f.render_widget(list, junk_inner);

    // 3. Footer
    let footer_text =
        " [↑/↓, j/k] Navigate   [Space] Select   [Enter] Execute Clean   [q/Esc] Quit";
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(app.theme.color7)
                .bg(app.theme.background),
        );
    f.render_widget(footer, chunks[2]);
}

fn format_target(name: &str, size: u64, selected: bool) -> String {
    let checkbox = if selected { "[X]" } else { "[ ]" };
    let size_str = system::format_bytes(size);
    // basic padding
    format!(" {} {:<40} {} ", checkbox, name, size_str)
}
