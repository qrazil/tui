// The ratatui side of bench/bench.m31: the same scenarios, the same frames, the same sampling
// (5 samples thrown away, 40 kept, a sample is the mean of `reps` frames of about 10 ms in all).
// Output goes through a BufWriter into a counting sink, so what is timed is build + diff + serialise
// (ratatui's Terminal::draw with the crossterm backend) and `bytes` is what a terminal would be sent.
use std::cell::Cell;
use std::io::{self, BufWriter, Write};
use std::rc::Rc;
use std::time::Instant;

use ratatui::backend::{Backend, ClearType, CrosstermBackend, WindowSize};
use ratatui::layout::{Position, Size};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Widget};
use ratatui::{Terminal, TerminalOptions, Viewport};

const W: u16 = 200;
const H: u16 = 50;
const WARMUP: usize = 5;
const SAMPLES: usize = 40;
const TARGET_NS: u128 = 10_000_000;

struct Counter {
    n: Rc<Cell<usize>>,
}
impl Write for Counter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.n.set(self.n.get() + buf.len());
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// CrosstermBackend writing to a sink: the three things it asks the tty for (cursor position, size,
/// window size) are answered with constants, so no tty is needed and nothing else is changed.
struct SinkBackend {
    inner: CrosstermBackend<BufWriter<Counter>>,
    size: Size,
}

impl Backend for SinkBackend {
    type Error = io::Error;
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
    {
        self.inner.draw(content)
    }
    fn hide_cursor(&mut self) -> io::Result<()> {
        self.inner.hide_cursor()
    }
    fn show_cursor(&mut self) -> io::Result<()> {
        self.inner.show_cursor()
    }
    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(Position::new(0, 0))
    }
    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        self.inner.set_cursor_position(position)
    }
    fn clear(&mut self) -> io::Result<()> {
        self.inner.clear()
    }
    fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
        self.inner.clear_region(clear_type)
    }
    fn size(&self) -> io::Result<Size> {
        Ok(self.size)
    }
    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize { columns_rows: self.size, pixels: Size::new(0, 0) })
    }
    fn flush(&mut self) -> io::Result<()> {
        Backend::flush(&mut self.inner)
    }
}

struct Term {
    t: Terminal<SinkBackend>,
    n: Rc<Cell<usize>>,
}

fn new_term(area: Rect) -> Term {
    let n = Rc::new(Cell::new(0));
    let inner = CrosstermBackend::new(BufWriter::with_capacity(1 << 16, Counter { n: n.clone() }));
    let backend = SinkBackend { inner, size: Size::new(area.width, area.height) };
    let t = Terminal::with_options(backend, TerminalOptions { viewport: Viewport::Fixed(area) }).unwrap();
    Term { t, n }
}

fn seg_text(row: usize, first_column: usize, count: usize) -> String {
    (0..count)
        .map(|offset| {
            let column = first_column + offset;
            if column % 8 == 7 {
                ' '
            } else {
                (b'a' + ((column * 7 + row * 13) % 26) as u8) as char
            }
        })
        .collect()
}

fn paint_grid(buf: &mut Buffer, width: u16, height: u16) {
    for y in 0..height as usize {
        for seg in 0..(width as usize / 25) {
            let style = Style::default().fg(Color::Indexed((1 + seg % 8) as u8));
            buf.set_string((seg * 25) as u16, y as u16, seg_text(y, seg * 25, 25), style);
        }
    }
}

fn name_of(kind: usize, number: usize) -> String {
    if kind == 0 {
        format!("src/ui/widget_{:04}.rs", number)
    } else {
        format!("\u{6587}\u{6863}/\u{65E5}\u{672C}\u{8A9E}_{:04}.txt", number)
    }
}

fn matches_of(kind: usize, query: &str) -> Vec<String> {
    let mut names = Vec::new();
    for number in 0..4000 {
        if names.len() >= 40 {
            break;
        }
        if format!("{:04}", number).contains(query) {
            names.push(name_of(kind, number));
        }
    }
    names
}

fn paint_finder(buf: &mut Buffer, names: &[String], query: &str) {
    let faint = Style::default().fg(Color::Indexed(8));
    let plain = Style::default();
    let sel = Style::default().fg(Color::Indexed(15)).bg(Color::Indexed(4));
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(faint)
        .render(Rect::new(0, 0, W, 44), buf);
    buf.set_string(2, 1, format!("> {}", query), plain);
    for x in 0..W {
        buf.set_string(x, 2, "\u{2500}", faint);
    }
    buf.set_string(0, 2, "\u{251C}", faint);
    buf.set_string(W - 1, 2, "\u{2524}", faint);
    for (row, name) in names.iter().enumerate() {
        let y = 3 + row as u16;
        if row == 0 {
            buf.set_style(Rect::new(1, y, W - 2, 1), sel);
            buf.set_string(1, y, format!("> {}", name), sel);
        } else {
            buf.set_string(1, y, format!("  {}", name), plain);
        }
    }
}

fn scroll_line(number: usize) -> String {
    format!(" the quick brown fox jumps over the lazy dog, {} and then some more text to fill", number * 7919 % 100003)
}

fn paint_scroll(buf: &mut Buffer, top: usize) {
    let number_style = Style::default().fg(Color::Indexed(3));
    let plain = Style::default();
    for y in 0..H as usize {
        let number = top + y;
        buf.set_string(0, y as u16, format!("{:05}", number), number_style);
        buf.set_string(5, y as u16, scroll_line(number), plain);
    }
}

fn paint_wide(buf: &mut Buffer, tick: usize) {
    let a = Style::default().fg(Color::Indexed(2));
    let b = Style::default().fg(Color::Indexed(5));
    for y in 0..H as usize {
        buf.set_string(0, y as u16, format!("{:02} \u{65E5}\u{672C}\u{8A9E}\u{306E}\u{30C6}\u{30AD}\u{30B9}\u{30C8} \u{4E2D}\u{6587}\u{6587}\u{5B57}", y), a);
        buf.set_string(30, y as u16, "\u{1F680} launch \u{1F525} fire \u{2728} sparkle \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} family \u{2764}\u{FE0F} love", b);
        buf.set_string(100, y as u16, "\u{D55C}\u{AD6D}\u{C5B4} \u{D14C}\u{C2A4}\u{D2B8} \u{1F600}\u{1F601}\u{1F602}\u{1F923} ascii tail", a);
    }
    buf.set_string(0, 10, format!("{:02}", tick % 100), b);
}

struct Scene {
    id: usize,
    term: Term,
    counter: usize,
    lists: Vec<Vec<String>>,
    queries: Vec<&'static str>,
    plain: Buffer,
    resize_w: [u16; 8],
    resize_h: [u16; 8],
}

impl Scene {
    fn step(&mut self) -> usize {
        self.counter += 1;
        let counter = self.counter;
        let before = self.term.n.get();
        match self.id {
            0 => {
                self.term.t.clear().unwrap();
                self.term.t.draw(|f| paint_grid(f.buffer_mut(), W, H)).unwrap();
            }
            1 | 2 => {
                let id = self.id;
                self.term
                    .t.draw(|f| {
                        paint_grid(f.buffer_mut(), W, H);
                        if id == 1 && counter % 2 == 0 {
                            f.buffer_mut().set_string(100, 25, "#", Style::default().fg(Color::Indexed(9)));
                        }
                    })
                    .unwrap();
            }
            3 => {
                let k = counter % 8;
                let (lists, queries) = (&self.lists, &self.queries);
                self.term.t.draw(|f| paint_finder(f.buffer_mut(), &lists[k], queries[k])).unwrap();
            }
            4 => {
                let top = counter % 9900;
                self.term.t.draw(|f| paint_scroll(f.buffer_mut(), top)).unwrap();
            }
            7 => {
                self.term.t.clear().unwrap();
                self.term.t.draw(|f| paint_wide(f.buffer_mut(), counter)).unwrap();
            }
            8 => {
                self.term.t.draw(|f| paint_wide(f.buffer_mut(), counter)).unwrap();
            }
            9 => {
                let k = counter % 8;
                let (w, h) = (self.resize_w[k], self.resize_h[k]);
                self.term.t.resize(Rect::new(0, 0, w, h)).unwrap();
                self.term.t.draw(|f| paint_grid(f.buffer_mut(), w, h)).unwrap();
            }
            10 => {
                self.plain.reset();
                paint_grid(&mut self.plain, W, H);
                return 0;
            }
            11 => {
                let k = counter % 8;
                self.plain.reset();
                paint_finder(&mut self.plain, &self.lists[k], self.queries[k]);
                return 0;
            }
            _ => {}
        }
        self.term.n.get() - before
    }
}

fn pct(ns: u128) -> String {
    format!("{}.{:02}", ns / 1000, ns % 1000 / 10)
}

fn run(name: &str, id: usize, kind: usize) {
    let area = Rect::new(0, 0, W, H);
    let queries = vec!["", "1", "12", "123", "1230", "123", "12", "1"];
    let lists = queries.iter().map(|q| matches_of(kind, q)).collect();
    let mut scene = Scene {
        id,
        term: new_term(area),
        counter: 0,
        lists,
        queries,
        plain: Buffer::empty(area),
        resize_w: [80, 120, 200, 132, 250, 100, 160, 90],
        resize_h: [24, 40, 50, 43, 70, 30, 50, 24],
    };
    for _ in 0..3 {
        scene.step();
    }
    let started = Instant::now();
    scene.step();
    let one = started.elapsed().as_nanos();
    let reps = std::cmp::max(1, TARGET_NS / (one + 1)) as usize;
    let mut times: Vec<u128> = Vec::new();
    let mut bytes_n = 0;
    for sample in 0..WARMUP + SAMPLES {
        let mut total = 0;
        let started = Instant::now();
        for _ in 0..reps {
            total += scene.step();
        }
        let ns = started.elapsed().as_nanos() / reps as u128;
        if sample >= WARMUP {
            times.push(ns);
        }
        bytes_n = total / reps;
    }
    times.sort();
    println!(
        "RESULT {} {} {} {} {}",
        name,
        pct(times[times.len() / 2]),
        pct(times[times.len() * 95 / 100]),
        pct(times[0]),
        bytes_n
    );
}

fn main() {
    run("full-200x50", 0, 0);
    run("diff-1cell", 1, 0);
    run("diff-0", 2, 0);
    run("finder-ascii-key", 3, 0);
    run("finder-cjk-key", 3, 1);
    run("scroll10k-diff", 4, 0);
    run("wide-full", 7, 0);
    run("wide-diff-1row", 8, 0);
    run("resize-storm-full", 9, 0);
    run("paint-grid", 10, 0);
    run("paint-finder", 11, 0);
}
