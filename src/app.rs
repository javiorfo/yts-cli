use crossterm::event::{self, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::symbols::scrollbar;
use ratatui::widgets::{Clear, Scrollbar, ScrollbarOrientation};

use crate::config::configuration;
use crate::downloads::Transmission;
use crate::elements::{
    Focus, InputBox, MovieTable, PopupNotification, PopupSubtitle, PopupTorrent,
};

pub async fn run() -> anyhow::Result<()> {
    let config = configuration()?;

    color_eyre::install().map_err(anyhow::Error::msg)?;

    let mut terminal = ratatui::init();

    let mut focus = Focus::default();
    let mut input_box = InputBox::default();
    let mut movie_table = MovieTable::new(&config.yts_host, config.yts_order);
    let mut popup_torrent = PopupTorrent::new();
    let mut popup_notification = PopupNotification::new();
    let mut popup_subtitle = PopupSubtitle::new(
        &config.opensubs_langs,
        config.opensubs_order,
        &config.yts_download_dir,
    );

    let mut transmission = Transmission::new(
        config.transmission_host,
        config.transmission_username,
        config.transmission_password,
        config.yts_download_dir.clone(),
    )?;

    let mut last_redraw_time = tokio::time::Instant::now();
    let redraw_interval = tokio::time::Duration::from_secs(1);

    transmission.scan().await.map_err(anyhow::Error::msg)?;

    loop {
        terminal.draw(|frame| {
            render(
                frame,
                &mut movie_table,
                &focus,
                &input_box,
                &popup_torrent,
                &popup_subtitle,
                &popup_notification,
                &mut transmission,
            )
        })?;

        let time_since_last_redraw = tokio::time::Instant::now().duration_since(last_redraw_time);
        let timeout = redraw_interval.saturating_sub(time_since_last_redraw);

        if tokio::time::Instant::now().duration_since(last_redraw_time) >= redraw_interval {
            transmission.scan().await.map_err(anyhow::Error::msg)?;
            last_redraw_time = tokio::time::Instant::now();
        }

        if event::poll(timeout)?
            && let Some(key) = event::read()?.as_key_press_event()
        {
            match focus {
                Focus::InputBox => match key.code {
                    KeyCode::Tab => {
                        focus = Focus::MovieTable;
                    }
                    KeyCode::Enter => {
                        if let Err(e) = movie_table.search(&input_box.text).await {
                            popup_notification.text = format!("  Error searching movies {e}");
                            popup_notification.show = true;
                            focus = Focus::PopupNotification;
                        } else {
                            focus = Focus::MovieTable;
                        }
                    }
                    KeyCode::Char(c) => {
                        input_box.text.push(c);
                    }
                    KeyCode::Backspace => {
                        input_box.text.pop();
                    }
                    KeyCode::Esc => {
                        ratatui::restore();
                        return Ok(());
                    }
                    _ => {}
                },
                Focus::MovieTable => match key.code {
                    KeyCode::Tab => {
                        focus = if transmission.is_visible() {
                            Focus::TorrentTable
                        } else {
                            Focus::InputBox
                        };
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        ratatui::restore();
                        return Ok(());
                    }
                    KeyCode::Char('j') | KeyCode::Down => movie_table.table_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => movie_table.table_state.select_previous(),
                    KeyCode::Char('l') | KeyCode::Right => {
                        if let Err(e) = movie_table.next_page(&input_box.text).await {
                            popup_notification.text = format!("  Error getting next page {e}");
                            popup_notification.show = true;
                            focus = Focus::PopupNotification;
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        if let Err(e) = movie_table.previous_page(&input_box.text).await {
                            popup_notification.text = format!("  Error getting previous page {e}");
                            popup_notification.show = true;
                            focus = Focus::PopupNotification;
                        }
                    }
                    KeyCode::Char('g') => movie_table.table_state.select_first(),
                    KeyCode::Char('G') => movie_table.table_state.select_last(),
                    KeyCode::Char('t') => {
                        if let Some(selected) = movie_table.table_state.selected()
                            && !movie_table.response.movies.is_empty()
                        {
                            let movie = &movie_table.response.movies[selected];
                            if let Err(e) = popup_torrent.search_torrents(movie).await {
                                popup_notification.text =
                                    format!("  Error searching torrents {e}");
                                popup_notification.show = true;
                                focus = Focus::PopupNotification;
                            } else {
                                popup_torrent.popup.show = true;
                                focus = Focus::PopupTorrent;
                            }
                        }
                    }
                    KeyCode::Char('s') => {
                        if let Some(selected) = movie_table.table_state.selected()
                            && !movie_table.response.movies.is_empty()
                        {
                            let movie = &movie_table.response.movies[selected];
                            if let Err(e) = popup_subtitle.search_subtitles(movie).await {
                                popup_notification.text =
                                    format!("  Error searching subtitles {e}");
                                popup_notification.show = true;
                                focus = Focus::PopupNotification;
                            } else {
                                popup_subtitle.popup.show = true;
                                focus = Focus::PopupSubtitle;
                            }
                        }
                    }
                    _ => {}
                },
                Focus::TorrentTable => match key.code {
                    KeyCode::Char('s') => {
                        if let Some(selected) = transmission.table_state.selected()
                            && !transmission.torrents.is_empty()
                        {
                            transmission
                                .toggle(selected)
                                .await
                                .map_err(anyhow::Error::msg)?;
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        ratatui::restore();
                        return Ok(());
                    }
                    KeyCode::Tab => {
                        focus = Focus::InputBox;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        transmission.table_state.select_next();
                        transmission.scroll_bar_up();
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        transmission.table_state.select_previous();
                        transmission.scroll_bar_down();
                    }
                    KeyCode::Char('d') => {
                        if let Some(selected) = transmission.table_state.selected()
                            && !transmission.torrents.is_empty()
                        {
                            transmission
                                .remove(selected)
                                .await
                                .map_err(anyhow::Error::msg)?;
                        }
                    }
                    _ => {}
                },
                Focus::PopupTorrent => match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        popup_torrent.popup.table_state.select_next();
                        popup_torrent
                            .popup
                            .scroll_bar_down(popup_torrent.torrents.len());
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        popup_torrent.popup.table_state.select_previous();
                        popup_torrent.popup.scroll_bar_up();
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        popup_torrent.popup.show = false;
                        focus = Focus::MovieTable;
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = popup_torrent.popup.table_state.selected() {
                            let torrent = &popup_torrent.torrents[selected];
                            transmission
                                .add(&torrent.link)
                                .await
                                .map_err(anyhow::Error::msg)?;
                        }
                        popup_torrent.popup.show = false;
                        focus = Focus::MovieTable;
                    }
                    _ => {}
                },
                Focus::PopupSubtitle => match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        popup_subtitle.popup.table_state.select_next();
                        popup_subtitle
                            .popup
                            .scroll_bar_down(popup_subtitle.subtitles.len());
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        popup_subtitle.popup.table_state.select_previous();
                        popup_subtitle.popup.scroll_bar_up();
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        popup_subtitle.popup.show = false;
                        focus = Focus::MovieTable;
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = popup_subtitle.popup.table_state.selected() {
                            let sub = &popup_subtitle.subtitles[selected];

                            popup_subtitle
                                .download_subtitle(&sub.download_link, &sub.movie)
                                .await?;

                            popup_notification.text =
                                format!("󰸞  Subtitle {}.srt downloaded", &sub.movie);
                        }
                        popup_subtitle.popup.show = false;
                        popup_notification.show = true;
                        focus = Focus::PopupNotification;
                    }
                    _ => {}
                },
                Focus::PopupNotification => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        popup_notification.show = false;
                        focus = Focus::MovieTable;
                    }
                    _ => {}
                },
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render(
    frame: &mut Frame,
    movie_table: &mut MovieTable,
    focus: &Focus,
    input_box: &InputBox,
    popup_torrent: &PopupTorrent,
    popup_subtitle: &PopupSubtitle,
    popup_notification: &PopupNotification,
    transmission: &mut Transmission,
) {
    let mut movie_table_state = movie_table.table_state;
    let (table, constraint) = movie_table.render(focus);

    let visible = transmission.is_visible();
    let mut transmission_table_state = transmission.table_state;
    let (torrent_table, torrent_constraint) = transmission.render(focus);

    let area = frame.area();
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(constraint),
        Constraint::Length(torrent_constraint),
    ]);

    let [input_box_area, movie_table_area, torrent_table_area] = area.layout(&layout);

    frame.render_widget(input_box.render(focus), input_box_area);

    if matches!(focus, Focus::InputBox) {
        frame.set_cursor_position((
            input_box_area.x + input_box.text.len() as u16 + 1,
            input_box_area.y + 1,
        ));
    }

    frame.render_stateful_widget(table, movie_table_area, &mut movie_table_state);

    if popup_notification.show {
        let popup_area = popup_notification.area(movie_table_area);
        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup_notification.render(), popup_area);
    }

    if popup_torrent.popup.show {
        let popup_area = popup_torrent.area(movie_table_area);
        let mut table_state = popup_torrent.popup.table_state;
        frame.render_widget(Clear, popup_area);
        frame.render_stateful_widget(popup_torrent.render(), popup_area, &mut table_state);

        let mut scroll_state = popup_torrent
            .popup
            .scroll_state
            .content_length(popup_torrent.torrents.len() + 2);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .track_symbol(None)
                .end_symbol(None),
            popup_area,
            &mut scroll_state,
        );
    }

    if popup_subtitle.popup.show {
        let popup_area = popup_subtitle.area(movie_table_area, constraint);
        let mut table_state = popup_subtitle.popup.table_state;
        frame.render_widget(Clear, popup_area);
        frame.render_stateful_widget(popup_subtitle.render(), popup_area, &mut table_state);

        let len = popup_subtitle.subtitles.len();

        if len > 0 {
            let mut scroll_state = popup_subtitle.popup.scroll_state.content_length(len + 2);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .symbols(scrollbar::VERTICAL)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
                popup_area,
                &mut scroll_state,
            );
        }
    }

    if visible {
        frame.render_stateful_widget(
            torrent_table,
            torrent_table_area,
            &mut transmission_table_state,
        );

        let torrents_len = transmission.torrents.len();

        if torrents_len > 5 {
            let mut scroll_state = transmission.scroll_state.content_length(torrents_len + 2);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .symbols(scrollbar::VERTICAL)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
                torrent_table_area,
                &mut scroll_state,
            );
        }
    }
}
