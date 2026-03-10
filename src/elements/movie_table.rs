use std::time::Duration;

use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Row, Table, TableState},
};
use yts::{Filters, Page, Response, Yts};

use crate::elements::Focus;

#[derive(Debug)]
pub struct MovieTable<'a> {
    pub table_state: TableState,
    pub response: Response,
    default_order: yts::OrderBy,
    yts: Yts<'a>,
}

impl<'a> MovieTable<'a> {
    const TITLE: &'static str = " YTS MOVIES ";

    pub fn new(host: &'a str, default_order: yts::OrderBy) -> Self {
        let mut table_state = TableState::default();
        table_state.select_first();
        table_state.select_first_column();

        let yts = Yts::new(host, Duration::from_secs(30));

        Self {
            table_state,
            yts,
            default_order,
            response: Response {
                page: Page {
                    current: 0,
                    of: 0,
                    total: 0,
                },
                movies: vec![],
            },
        }
    }

    pub fn footer(&self) -> String {
        let page = &self.response.page;
        if page.total != 0 {
            format!(
                " {} Movie/s - Page {}/{} ",
                page.total, page.current, page.of
            )
        } else {
            String::from(" 0 Movie/s ")
        }
    }

    pub async fn search(&mut self, text: &str) -> yts::Result {
        let response = self
            .yts
            .search_with_filter(
                self.clean_search_text(text),
                Filters::default()
                    .year(self.year_filter(text))
                    .rating(self.rating_filter(text))
                    .order_by(self.order_filter(text))
                    .build(),
            )
            .await?;

        self.response = response;

        Ok(())
    }

    pub async fn next_page(&mut self, text: &str) -> yts::Result {
        let response = &self.response;
        let next_page = response.page.current + 1;
        if next_page <= response.page.of {
            self.response = self
                .yts
                .search_with_filter(
                    self.clean_search_text(text),
                    Filters::default()
                        .year(self.year_filter(text))
                        .rating(self.rating_filter(text))
                        .order_by(self.order_filter(text))
                        .page(next_page)
                        .build(),
                )
                .await?;
        }

        Ok(())
    }

    pub async fn previous_page(&mut self, text: &str) -> yts::Result {
        let response = &self.response;
        let prev_page = response.page.current - 1;
        if prev_page > 0 {
            self.response = self
                .yts
                .search_with_filter(
                    self.clean_search_text(text),
                    Filters::default()
                        .year(self.year_filter(text))
                        .rating(self.rating_filter(text))
                        .order_by(self.order_filter(text))
                        .page(prev_page)
                        .build(),
                )
                .await?;
        }

        Ok(())
    }

    fn clean_search_text(&self, text: &'a str) -> &'a str {
        let indices: Vec<Option<usize>> = vec![
            text.find(" year:"),
            text.find(" rating:"),
            text.find(" order:"),
        ];

        let min_index = indices.iter().filter_map(|&x| x).min();

        match min_index {
            Some(index) => &text[..index],
            None => text,
        }
    }

    fn year_filter(&self, text: &'a str) -> yts::Year {
        match Self::filter_value(text, " year:") {
            Some(year) => match year.parse::<u32>() {
                Ok(n) => yts::Year::Equal(n),
                _ => yts::Year::All,
            },
            _ => yts::Year::All,
        }
    }

    fn order_filter(&self, text: &'a str) -> yts::OrderBy {
        match Self::filter_value(text, " order:") {
            Some(o) => {
                let order: Result<yts::OrderBy, _> = o.try_into();
                order.unwrap_or(self.default_order.clone())
            }
            _ => self.default_order.clone(),
        }
    }

    fn rating_filter(&self, text: &'a str) -> yts::Rating {
        match Self::filter_value(text, " rating:") {
            Some(rating) => match rating {
                "1" => yts::Rating::One,
                "2" => yts::Rating::Two,
                "3" => yts::Rating::Three,
                "4" => yts::Rating::Four,
                "5" => yts::Rating::Five,
                "6" => yts::Rating::Six,
                "7" => yts::Rating::Seven,
                "8" => yts::Rating::Eight,
                "9" => yts::Rating::Nine,
                _ => yts::Rating::All,
            },
            _ => yts::Rating::All,
        }
    }

    fn filter_value(text: &'a str, filter: &'a str) -> Option<&'a str> {
        match text.split_once(filter) {
            Some((_, rest)) => {
                if let Some((year_str, _)) = rest.split_once(' ') {
                    Some(year_str)
                } else {
                    Some(rest)
                }
            }
            _ => None,
        }
    }

    fn response_to_rows(&self) -> Vec<Row<'a>> {
        let mut rows: Vec<Vec<String>> = Vec::new();

        if self.response.movies.is_empty() {
            return vec![];
        };

        for movie in &self.response.movies {
            let genres = movie
                .genres
                .iter()
                .map(|g| g.to_string())
                .collect::<Vec<String>>()
                .join("/");

            rows.push(vec![
                movie.year.to_string(),
                movie.name.clone(),
                genres,
                movie.rating.to_string(),
            ]);
        }

        rows.iter()
            .map(|item| Row::new(item.iter().cloned()))
            .collect::<Vec<_>>()
    }

    pub fn render(&mut self, focus: &Focus) -> (Table<'_>, u16) {
        let rows = self.response_to_rows();

        let (header, constraint) = if !rows.is_empty() {
            (["Year", "Name", "Genre", "Rating"], rows.len() as u16 + 4)
        } else {
            (["", "", "", ""], 2)
        };

        let header = Row::new(header)
            .style(Style::new().dark_gray().bold())
            .bottom_margin(0);

        let widths = [
            Constraint::Percentage(10),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
            Constraint::Percentage(10),
        ];

        let border_style = if matches!(focus, Focus::MovieTable) {
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        (
            Table::new(rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(border_style)
                        .title(Self::TITLE)
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
                .highlight_symbol("  "),
            constraint,
        )
    }
}
