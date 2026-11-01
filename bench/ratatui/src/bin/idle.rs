// ratatui's side of bench/idle.m31: raw mode, alternate screen, one frame, a 250 ms poll, `x` counts, `q` quits.
use std::io::stdout;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::widgets::Paragraph;
use ratatui::DefaultTerminal;

fn draw(terminal: &mut DefaultTerminal, count: u32) {
    terminal
        .draw(|f| {
            let a = f.area();
            let text = format!("READY-{}-{}x{}", &"pstuvw"[(count % 6) as usize..(count % 6) as usize + 1], a.width, a.height);
            f.render_widget(Paragraph::new(text), a);
        })
        .unwrap();
}

fn main() {
    enable_raw_mode().unwrap();
    execute!(stdout(), EnterAlternateScreen).unwrap();
    let mut terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout())).unwrap();
    let mut count = 0;
    draw(&mut terminal, count);
    loop {
        if event::poll(Duration::from_millis(250)).unwrap() {
            match event::read().unwrap() {
                Event::Key(k) if k.code == KeyCode::Char('q') => break,
                Event::Key(k) if k.code == KeyCode::Char('x') => {
                    count += 1;
                    draw(&mut terminal, count);
                }
                Event::Resize(_, _) => draw(&mut terminal, count),
                _ => {}
            }
        }
    }
    execute!(stdout(), LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}
