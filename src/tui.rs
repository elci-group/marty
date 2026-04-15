use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, Tabs, Paragraph, Gauge, ListState},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, sync::Arc};
use parking_lot::Mutex;
use crate::memory::{Hotspots, Beliefs, Trace};
use crate::error::Result;

pub struct TuiApp {
    hotspots: Arc<Mutex<Hotspots>>,
    beliefs: Arc<Mutex<Beliefs>>,
    trace: Arc<Mutex<Trace>>,
    active_tab: usize,
    should_quit: bool,
    hotspots_state: ListState,
    beliefs_state: ListState,
    trace_state: ListState,
}

impl TuiApp {
    pub fn new(hotspots: Arc<Mutex<Hotspots>>, beliefs: Arc<Mutex<Beliefs>>, trace: Arc<Mutex<Trace>>) -> Self {
        Self {
            hotspots,
            beliefs,
            trace,
            active_tab: 0,
            should_quit: false,
            hotspots_state: ListState::default(),
            beliefs_state: ListState::default(),
            trace_state: ListState::default(),
        }
    }

    pub fn run(mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        while !self.should_quit {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Right | KeyCode::Tab => self.active_tab = (self.active_tab + 1) % 3,
                        KeyCode::Left => self.active_tab = (self.active_tab + 2) % 3,
                        KeyCode::Down | KeyCode::Char('j') => self.next_item(),
                        KeyCode::Up | KeyCode::Char('k') => self.previous_item(),
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }
    
    fn next_item(&mut self) {
        match self.active_tab {
            0 => {
                let hs = self.hotspots.lock();
                let count = hs.items.lock().len();
                if count > 0 {
                    let i = match self.hotspots_state.selected() {
                        Some(i) => if i >= count - 1 { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.hotspots_state.select(Some(i));
                }
            }
            1 => {
                let count = self.beliefs.lock().nodes.len();
                if count > 0 {
                    let i = match self.beliefs_state.selected() {
                        Some(i) => if i >= count - 1 { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.beliefs_state.select(Some(i));
                }
            }
            2 => {
                let count = std::cmp::min(self.trace.lock().entries.len(), 50);
                if count > 0 {
                    let i = match self.trace_state.selected() {
                        Some(i) => if i >= count - 1 { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.trace_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    fn previous_item(&mut self) {
        match self.active_tab {
            0 => {
                let hs = self.hotspots.lock();
                let count = hs.items.lock().len();
                if count > 0 {
                    let i = match self.hotspots_state.selected() {
                        Some(i) => if i == 0 { count - 1 } else { i - 1 },
                        None => count - 1,
                    };
                    self.hotspots_state.select(Some(i));
                }
            }
            1 => {
                let count = self.beliefs.lock().nodes.len();
                if count > 0 {
                    let i = match self.beliefs_state.selected() {
                        Some(i) => if i == 0 { count - 1 } else { i - 1 },
                        None => count - 1,
                    };
                    self.beliefs_state.select(Some(i));
                }
            }
            2 => {
                let count = std::cmp::min(self.trace.lock().entries.len(), 50);
                if count > 0 {
                    let i = match self.trace_state.selected() {
                        Some(i) => if i == 0 { count - 1 } else { i - 1 },
                        None => count - 1,
                    };
                    self.trace_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    fn ui(&mut self, f: &mut ratatui::Frame) {
        let size = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(size);

        let titles = vec!["🔥 Hotspots", "🧠 Beliefs", "📜 Trace"];
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Marty Explorer"))
            .select(self.active_tab)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        f.render_widget(tabs, chunks[0]);

        match self.active_tab {
            0 => self.render_hotspots(f, chunks[1]),
            1 => self.render_beliefs(f, chunks[1]),
            2 => self.render_trace(f, chunks[1]),
            _ => {}
        }
    }

    fn render_hotspots(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(area);

        let hs = self.hotspots.lock();
        let items_lock = hs.items.lock();
        let mut sorted: Vec<_> = items_lock.values().collect();
        sorted.sort_by(|a, b| b.energy.partial_cmp(&a.energy).unwrap_or(std::cmp::Ordering::Equal));

        let list_items: Vec<ListItem> = sorted
            .iter()
            .map(|h| {
                ListItem::new(vec![Line::from(vec![Span::raw(&h.path)])])
            })
            .collect();

        if self.hotspots_state.selected().is_none() && !list_items.is_empty() {
            self.hotspots_state.select(Some(0));
        }

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Hotspots"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, chunks[0], &mut self.hotspots_state);

        if let Some(selected) = self.hotspots_state.selected() {
            if let Some(h) = sorted.get(selected) {
                let detail_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                    .split(chunks[1]);
                
                let block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Details");
                f.render_widget(block.clone(), chunks[1]);
                
                let energy_percent = ((h.energy / 100.0) * 100.0).clamp(0.0, 100.0) as u16;
                let gauge = Gauge::default()
                    .block(Block::default().title("Energy Level"))
                    .gauge_style(Style::default().fg(Color::Yellow).bg(Color::Black).add_modifier(Modifier::ITALIC))
                    .percent(energy_percent);
                f.render_widget(gauge, detail_chunks[0]);
                
                let text = vec![
                    Line::from(Span::raw(format!("Path: {}", h.path))),
                    Line::from(Span::raw(format!("Energy Score: {:.2}", h.energy))),
                    Line::from(Span::raw(format!("Last Visited: {}", h.last_ts))),
                ];
                let p = Paragraph::new(text).block(Block::default());
                f.render_widget(p, detail_chunks[1]);
            }
        } else {
            let block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Details");
            f.render_widget(block, chunks[1]);
        }
    }

    fn render_beliefs(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(area);
            
        let b = self.beliefs.lock();
        let nodes: Vec<_> = b.nodes.iter().collect();
        
        let list_items: Vec<ListItem> = nodes
            .iter()
            .map(|(_path, node)| {
                ListItem::new(vec![Line::from(vec![
                    Span::styled(format!("🔹 {} ", node.label), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                ])])
            })
            .collect();

        if self.beliefs_state.selected().is_none() && !list_items.is_empty() {
            self.beliefs_state.select(Some(0));
        }

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Beliefs"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, chunks[0], &mut self.beliefs_state);
        
        if let Some(selected) = self.beliefs_state.selected() {
            if let Some((path, node)) = nodes.get(selected) {
                let mut text = vec![
                    Line::from(Span::raw(format!("Label: {}", node.label))),
                    Line::from(Span::raw(format!("Path: {}", path))),
                    Line::from(Span::raw("Metadata:")),
                ];
                
                for (key, value) in &node.metadata {
                    text.push(Line::from(Span::raw(format!("  {}: {}", key, value))));
                }
                
                let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Details"));
                f.render_widget(p, chunks[1]);
            }
        } else {
            let block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Details");
            f.render_widget(block, chunks[1]);
        }
    }

    fn render_trace(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(area);
            
        let t = self.trace.lock();
        let entries: Vec<_> = t.entries.iter().rev().take(50).collect();
        
        let list_items: Vec<ListItem> = entries
            .iter()
            .map(|entry| {
                let (icon, color, details) = match entry {
                    crate::signals::Signal::Visit { path, .. } => ("🚶", Color::Green, path.to_string()),
                    crate::signals::Signal::Tag { path, tag, .. } => ("🏷️", Color::Magenta, format!("{} (Tag: {})", path, tag)),
                    _ => ("🔹", Color::Gray, "Unknown signal".to_string()),
                };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!("{} ", icon), Style::default()),
                        Span::styled(details, Style::default().fg(color)),
                    ]),
                ])
            })
            .collect();

        if self.trace_state.selected().is_none() && !list_items.is_empty() {
            self.trace_state.select(Some(0));
        }

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Trace History"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, chunks[0], &mut self.trace_state);
        
        if let Some(selected) = self.trace_state.selected() {
            if let Some(entry) = entries.get(selected) {
                let text = match entry {
                    crate::signals::Signal::Visit { path, ts } => vec![
                        Line::from(Span::raw("Type: Visit 🚶")),
                        Line::from(Span::raw(format!("Path: {}", path))),
                        Line::from(Span::raw(format!("Timestamp: {}", ts))),
                    ],
                    crate::signals::Signal::Tag { path, tag, ts } => vec![
                        Line::from(Span::raw("Type: Tag 🏷️")),
                        Line::from(Span::raw(format!("Path: {}", path))),
                        Line::from(Span::raw(format!("Tag: {}", tag))),
                        Line::from(Span::raw(format!("Timestamp: {}", ts))),
                    ],
                    crate::signals::Signal::Reinforce { path, weight, ts } => vec![
                        Line::from(Span::raw("Type: Reinforce 💪")),
                        Line::from(Span::raw(format!("Path: {}", path))),
                        Line::from(Span::raw(format!("Weight: {}", weight))),
                        Line::from(Span::raw(format!("Timestamp: {}", ts))),
                    ],
                    crate::signals::Signal::Promote { path, ts } => vec![
                        Line::from(Span::raw("Type: Promote 🚀")),
                        Line::from(Span::raw(format!("Path: {}", path))),
                        Line::from(Span::raw(format!("Timestamp: {}", ts))),
                    ],
                    crate::signals::Signal::Demote { path, reason, ts } => vec![
                        Line::from(Span::raw("Type: Demote 📉")),
                        Line::from(Span::raw(format!("Path: {}", path))),
                        Line::from(Span::raw(format!("Reason: {}", reason))),
                        Line::from(Span::raw(format!("Timestamp: {}", ts))),
                    ],
                };
                
                let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Details"));
                f.render_widget(p, chunks[1]);
            }
        } else {
            let block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Details");
            f.render_widget(block, chunks[1]);
        }
    }
}
