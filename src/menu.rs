mod item;

use std::io::{self, IsTerminal};

use skim::{
    Skim, SkimOptions,
    prelude::{SkimOptionsBuilder, unbounded},
    tui::{BorderType, Event, statusline::InfoDisplay},
};

use crate::{AppResult, servers::Server};

const HELP: &str = "Enter: connect   Esc: cancel";
const COLORS: &str = concat!(
    "dark,current:255:bold,current_bg:237,",
    "current_match:117:bold,current_match_bg:237,",
    "cursor:205:bold,prompt:117,header:245,border:240,",
    "matched:117,matched_bg:-1"
);

pub async fn select_server(servers: Vec<Server>) -> AppResult<Option<String>> {
    require_terminal()?;

    let use_colors = colors_enabled();
    let mut menu = create_menu(servers, use_colors)?;
    run_menu(&mut menu).await?;
    close_menu(&mut menu)?;

    let output = menu.output();
    if output.is_abort {
        return Ok(None);
    }

    Ok(output
        .selected_items
        .first()
        .map(|item| item.output().into_owned()))
}

fn close_menu(menu: &mut Skim) -> AppResult<()> {
    // Cancellation alone can leave the input reader competing with SSH.
    menu.tui_mut().stop_and_join();
    menu.tui_mut().exit()?;
    Ok(())
}

fn require_terminal() -> io::Result<()> {
    if io::stdin().is_terminal() && io::stderr().is_terminal() {
        Ok(())
    } else {
        Err(io::Error::other(
            "an interactive terminal is required to select a host",
        ))
    }
}

fn colors_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty())
}

fn menu_options(use_colors: bool) -> AppResult<SkimOptions> {
    let options = SkimOptionsBuilder::default()
        .reverse(true)
        .cycle(true)
        .border(BorderType::Rounded)
        .margin("1,2")
        .prompt("SSH: ")
        .header(HELP.to_owned())
        .highlight_line(true)
        // A full block fills the cursor cell even when skim resets its background.
        .selector_icon("█")
        .multi_select_icon("")
        .color(if use_colors { COLORS } else { "bw" })
        .build()?;

    Ok(options)
}

fn create_menu(servers: Vec<Server>, use_colors: bool) -> AppResult<Skim> {
    let options = menu_options(use_colors)?;
    let (sender, receiver) = unbounded();
    sender.send(item::create_items(servers, use_colors))?;
    drop(sender);

    let mut menu = Skim::init(options, Some(receiver))?;
    // Keep skim's cursor statistics available without displaying its status line.
    menu.app_mut().input.info = InfoDisplay::Hidden.into();

    Ok(menu)
}

async fn run_menu(menu: &mut Skim) -> AppResult<()> {
    menu.start();
    menu.init_tui()?;
    menu.enter().await?;

    loop {
        update_header(menu);
        if menu.tick().await? {
            return Ok(());
        }
    }
}

fn update_header(menu: &mut Skim) {
    let app = menu.app();
    let (Some(status), Some(area)) = (&app.input.status_info, app.layout.header_area) else {
        return;
    };

    let total = status.matched;
    let position = if total == 0 {
        0
    } else {
        status.current_item_idx + 1
    };
    let indent =
        app.options.selector_icon.chars().count() + app.options.multi_select_icon.chars().count();
    // Skim reserves a border cell on each side and indents the header by its icons.
    let width = usize::from(area.width).saturating_sub(2 + indent);
    let header = format_header(position, total, width);

    if app.header.header != header {
        menu.app_mut().header.header = header;
        let _ = menu.event_sender().try_send(Event::Render);
    }
}

fn format_header(position: usize, total: usize, width: usize) -> String {
    let counter = format!("{position}/{total}");
    let hint = if width > HELP.chars().count() + counter.len() {
        HELP
    } else {
        ""
    };
    let padding = width.saturating_sub(hint.chars().count() + counter.len());

    format!("{hint}{:padding$}{counter}", "")
}
