use unicode_width::UnicodeWidthStr;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Widget};

#[derive(Debug, Clone)]
pub struct Tabs<'a> {
    block: Option<Block<'a>>,
    titles: Vec<String>,
    selected: usize,
    style: Style,
    highlight_style: Style,
    #[allow(dead_code)]
    divider: &'a str,
    #[allow(dead_code)]
    divider_inactive: &'a str,
    #[allow(dead_code)]
    divider_style: Style,
}

impl<'a> Default for Tabs<'a> {
    fn default() -> Tabs<'a> {
        Tabs {
            block: None,
            titles: vec![String::from("")],
            selected: 0,
            style: Default::default(),
            highlight_style: Default::default(),
            divider: "",
            divider_inactive: "",
            divider_style: Default::default(),
        }
    }
}

#[allow(dead_code)]
impl<'a> Tabs<'a> {
    pub fn block(mut self, block: Block<'a>) -> Tabs<'a> {
        self.block = Some(block);
        self
    }

    pub fn titles(mut self, titles: Vec<String>) -> Tabs<'a> {
        self.titles = titles;
        self
    }

    pub fn select(mut self, selected: usize) -> Tabs<'a> {
        self.selected = selected;
        self
    }

    pub fn style(mut self, style: Style) -> Tabs<'a> {
        self.style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Tabs<'a> {
        self.highlight_style = style;
        self
    }

    pub fn divider(mut self, divider: &'a str) -> Tabs<'a> {
        self.divider = divider;
        self
    }

    pub fn divider_style(mut self, style: Style) -> Tabs<'a> {
        self.divider_style = style;
        self
    }
}

impl<'a> Widget for Tabs<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        let tabs_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if tabs_area.height < 1 {
            return;
        }

        let mut x = tabs_area.left();
        for (i, title) in self.titles.iter().enumerate() {
            let padded = format!("  {}  ", title);
            let w = padded.width() as u16;
            if x + w > tabs_area.right() {
                break;
            }
            let style = if i == self.selected {
                self.highlight_style
            } else {
                self.style
            };
            buf.set_string(x, tabs_area.top(), &padded, style);
            x += w;
        }
    }
}
