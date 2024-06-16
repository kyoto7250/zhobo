use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, List, ListItem, Widget},
    Frame,
};
use std::iter::Iterator;

struct ScrollableList<'b, L>
where
    L: Iterator<Item = Line<'b>>,
{
    block: Option<Block<'b>>,
    items: L,
    style: Style,
}

impl<'b, L> ScrollableList<'b, L>
where
    L: Iterator<Item = Line<'b>>,
{
    fn new(items: L) -> Self {
        Self {
            block: None,
            items,
            style: Style::default(),
        }
    }

    fn block(mut self, block: Block<'b>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'b, L> Widget for ScrollableList<'b, L>
where
    L: Iterator<Item = Line<'b>>,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        List::new(self.items.map(ListItem::new).collect::<Vec<ListItem>>())
            .block(self.block.unwrap_or_default())
            .style(self.style)
            .render(area, buf);
    }
}

pub fn draw_list_block<'b, L>(f: &mut Frame, r: Rect, block: Block<'b>, items: L)
where
    L: Iterator<Item = Line<'b>>,
{
    let list = ScrollableList::new(items).block(block);
    f.render_widget(list, r);
}
