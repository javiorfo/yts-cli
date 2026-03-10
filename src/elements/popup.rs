use std::{
    fs::File,
    io::{self, Cursor},
    path::Path,
};

use opensubs::{Filters, Language, OrderBy, Page, Response, SearchBy, Subtitle};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Row, ScrollbarState, Table, TableState},
};
use yts::{Movie, Torrent, Yts};

pub struct Popup<'a> {
    pub table_state: TableState,
    pub scroll_state: ScrollbarState,
    pub show: bool,
    title: &'a str,
}

impl<'a> Popup<'a> {
    pub fn new(title: &'a str) -> Popup<'a> {
        let mut table_state = TableState::default();
        table_state.select_first();
        table_state.select_first_column();

        Self {
            title,
            table_state,
            scroll_state: ScrollbarState::default().position(1),
            show: false,
        }
    }

    pub fn centered_area(&self, area: Rect, x: u16, y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Length(y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Length(x)]).flex(Flex::Center);
        let [area] = area.layout(&vertical);
        let [area] = area.layout(&horizontal);
        area
    }

    pub fn scroll_bar_up(&mut self) {
        let position = self.scroll_state.get_position();
        if position > 1 {
            self.scroll_state = self.scroll_state.position(position.saturating_sub(1));
        }
    }

    pub fn scroll_bar_down(&mut self, len: usize) {
        let position = self.scroll_state.get_position();
        if len > 0 && position < len - 1 {
            self.scroll_state = self.scroll_state.position(position.saturating_add(1));
        }
    }
}

pub struct PopupNotification {
    pub text: String,
    pub show: bool,
}

impl PopupNotification {
    pub fn new() -> PopupNotification {
        Self {
            text: String::new(),
            show: false,
        }
    }

    pub fn area(&self, area: Rect) -> Rect {
        let vertical = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
        let horizontal =
            Layout::horizontal([Constraint::Length(self.text.len() as u16)]).flex(Flex::Center);
        let [area] = area.layout(&vertical);
        let [area] = area.layout(&horizontal);
        area
    }

    pub fn render(&self) -> Paragraph<'_> {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .title(" Notification ");

        Paragraph::new(self.text.clone())
            .style(Style::default().fg(Color::White))
            .block(block)
    }
}

pub struct PopupTorrent<'a> {
    pub popup: Popup<'a>,
    pub torrents: Vec<Torrent>,
    yts: Yts<'a>,
}

impl<'a> PopupTorrent<'a> {
    pub fn new() -> PopupTorrent<'a> {
        Self {
            popup: Popup::new(" Torrents "),
            yts: Yts::default(),
            torrents: vec![],
        }
    }

    pub fn area(&self, area: Rect) -> Rect {
        self.popup.centered_area(area, 70, 5)
    }

    pub async fn search_torrents(&mut self, movie: &Movie) -> yts::Result {
        self.torrents = self.yts.torrents(movie).await?;
        Ok(())
    }

    pub fn render(&self) -> Table<'a> {
        let widths = [
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ];

        let header = Row::new(["Quality", "Size", "Language", "Runtime", "Peers/Seeds"])
            .style(Style::new().dark_gray().bold())
            .bottom_margin(0);

        let mut rows: Vec<Vec<String>> = Vec::new();

        for torrent in &self.torrents {
            let quality = &torrent.quality;
            let quality: &str = quality.into();

            rows.push(vec![
                quality.to_owned(),
                torrent.size.clone(),
                torrent.language.clone(),
                torrent.runtime.clone(),
                torrent.peers_seeds.clone(),
            ]);
        }

        let rows = rows
            .iter()
            .map(|item| Row::new(item.iter().cloned()))
            .collect::<Vec<_>>();

        let footer = format!(" {} torrent/s ", rows.len());

        Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain)
                    .title(self.popup.title)
                    .title_style(Style::new().white().bold())
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .title_bottom(footer),
            )
            .column_spacing(1)
            .style(Style::default().fg(Color::White))
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .column_highlight_style(Color::Gray)
            .cell_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ")
    }
}

pub struct PopupSubtitle<'a> {
    pub popup: Popup<'a>,
    pub subtitles: Vec<Subtitle>,
    pub page: Page,
    languages: &'a [Language],
    order: OrderBy,
    download_dir: &'a str,
}

impl<'a> PopupSubtitle<'a> {
    pub fn new(
        languages: &'a [Language],
        order: OrderBy,
        download_dir: &'a str,
    ) -> PopupSubtitle<'a> {
        Self {
            popup: Popup::new(" Subtitles "),
            languages,
            order,
            subtitles: vec![],
            download_dir,
            page: Self::empty_page(),
        }
    }

    pub fn area(&self, area: Rect, y: u16) -> Rect {
        let y = y.min(self.subtitles.len() as u16) + 4;
        self.popup.centered_area(area, 120, y)
    }

    pub async fn search_subtitles(&mut self, movie: &Movie) -> opensubs::Result {
        self.subtitles.clear();
        self.page = Self::empty_page();

        let name = &movie.name;
        let year = movie.year;
        let results = opensubs::search(SearchBy::MovieAndFilter(
            &movie.name,
            Filters::default()
                .year(movie.year)
                .languages(self.languages)
                .order_by(self.order.clone())
                .build(),
        ))
        .await?;

        match results {
            Response::Movie(movies) => {
                if let Some(movie) = movies.iter().find(|&movie| {
                    movie.name.to_lowercase() == format!("{} ({})", name.to_lowercase(), year)
                }) && let Response::Subtitle(page, subtitles) =
                    opensubs::search(SearchBy::Url(&movie.subtitles_link)).await?
                {
                    self.subtitles = subtitles;
                    self.page = page;
                }
            }
            Response::Subtitle(page, subtitles) => {
                self.subtitles = subtitles;
                self.page = page;
            }
        }

        Ok(())
    }

    pub async fn download_subtitle(&self, link: &str, movie_name: &str) -> anyhow::Result<()> {
        let file_name = format!("{movie_name}.srt");
        let output = Path::new(self.download_dir).join(&file_name);

        let response = reqwest::get(link).await?;
        let zip_bytes = response.bytes().await?.to_vec();

        self.save_first_srt(&zip_bytes, &output)?;

        Ok(())
    }

    fn save_first_srt(&self, zip_data: &[u8], output: &Path) -> Result<(), io::Error> {
        let cursor = Cursor::new(zip_data);
        let mut archive = zip::ZipArchive::new(cursor)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name().to_owned();

            if file.is_dir() || !file_name.to_lowercase().ends_with(".srt") {
                continue;
            }

            let mut outfile = File::create(output)?;
            io::copy(&mut file, &mut outfile)?;

            return Ok(());
        }
        Ok(())
    }

    pub fn render(&self) -> Table<'a> {
        let widths = [
            Constraint::Percentage(5),
            Constraint::Percentage(35),
            Constraint::Percentage(20),
            Constraint::Percentage(5),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ];

        let header = Row::new([
            "#",
            "Movie",
            "Language",
            "CD",
            "Uploaded",
            "Downloads",
            "Rating",
        ])
        .style(Style::new().dark_gray().bold())
        .bottom_margin(0);

        let mut rows: Vec<Vec<String>> = Vec::new();

        for (i, sub) in self.subtitles.iter().enumerate() {
            rows.push(vec![
                (i + 1).to_string(),
                sub.movie.clone(),
                sub.language.clone(),
                sub.cd.clone(),
                sub.uploaded.clone(),
                sub.downloads.to_string(),
                sub.rating.to_string(),
            ]);
        }

        let rows = rows
            .iter()
            .map(|item| Row::new(item.iter().cloned()))
            .collect::<Vec<_>>();

        Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain)
                    .title(self.popup.title)
                    .title_style(Style::new().white().bold())
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .title_bottom(self.footer()),
            )
            .column_spacing(1)
            .style(Style::default().fg(Color::White))
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .column_highlight_style(Color::Gray)
            .cell_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ")
    }

    fn footer(&self) -> String {
        let page = &self.page;
        if page.total != 0 {
            format!(
                " {} subtitle/s - From {} to {} ",
                page.total, page.from, page.to
            )
        } else {
            String::from(" 0 subtitles ")
        }
    }

    fn empty_page() -> Page {
        Page {
            from: 0,
            to: 0,
            total: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use opensubs::Language;

    use crate::elements::PopupSubtitle;

    #[tokio::test]
    async fn search_subtitles() {
        let mut popup_subtitle =
            PopupSubtitle::new(&[Language::Spanish], opensubs::OrderBy::Rating, "");

        let response = yts::Yts::default().search("The Godfather").await.unwrap();

        let movie = response.movies.first().unwrap();

        popup_subtitle.search_subtitles(movie).await.unwrap();

        assert!(!popup_subtitle.subtitles.is_empty());
    }
}
