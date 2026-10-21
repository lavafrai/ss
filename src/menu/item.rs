use std::{borrow::Cow, sync::Arc};

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use skim::{DisplayContext, SkimItem};
use unicode_width::UnicodeWidthStr;

use crate::servers::Server;

const COLUMN_GAP: usize = 2;
const MARKER_PADDING: &str = " ";
const DESTINATION_COLOR: Color = Color::Indexed(245);

#[derive(Clone, Copy)]
struct TableLayout {
    alias_width: usize,
    minimum_width: usize,
}

impl TableLayout {
    fn for_servers(servers: &[Server]) -> Self {
        let alias_width = servers
            .iter()
            .map(|server| server.alias.width())
            .max()
            .unwrap_or(0);
        let destination_width = servers
            .iter()
            .map(|server| server.destination.width())
            .max()
            .unwrap_or(0);

        Self {
            alias_width,
            minimum_width: MARKER_PADDING.width() + alias_width + COLUMN_GAP + destination_width,
        }
    }
}

struct ServerItem {
    server: Server,
    layout: TableLayout,
    use_colors: bool,
}

pub(super) fn create_items(servers: Vec<Server>, use_colors: bool) -> Vec<Arc<dyn SkimItem>> {
    let layout = TableLayout::for_servers(&servers);

    servers
        .into_iter()
        .map(|server| {
            Arc::new(ServerItem {
                server,
                layout,
                use_colors,
            }) as Arc<dyn SkimItem>
        })
        .collect()
}

impl SkimItem for ServerItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.server.alias)
    }

    fn display(&self, context: DisplayContext) -> Line<'_> {
        let mut line = context.clone().to_line(self.text());
        line.spans
            .insert(0, Span::styled(MARKER_PADDING, context.base_style));

        if context.container_width >= self.layout.minimum_width {
            self.append_destination(&mut line, &context);
        }

        line
    }
}

impl ServerItem {
    fn append_destination<'a>(&'a self, line: &mut Line<'a>, context: &DisplayContext) {
        let padding = self.layout.alias_width - self.server.alias.width() + COLUMN_GAP;
        line.push_span(Span::styled(" ".repeat(padding), context.base_style));
        line.push_span(Span::styled(
            &self.server.destination,
            self.destination_style(context),
        ));
    }

    fn destination_style(&self, context: &DisplayContext) -> Style {
        let style = context.base_style.remove_modifier(Modifier::BOLD);
        if self.use_colors {
            style.fg(DESTINATION_COLOR)
        } else {
            style.add_modifier(Modifier::DIM)
        }
    }
}
