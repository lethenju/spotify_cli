extern crate failure;
extern crate spotify_api;
extern crate termion;
extern crate tui;

#[allow(dead_code)]
mod interface;

use interface::{Albums, Tracks};
pub use spotify_api::EasyAPI;
use std::io;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, SelectableList, Text, Widget};
use tui::Terminal;

use interface::util::{Event, Events};


/// Entry point of the text user interface
fn main() -> Result<(), failure::Error> {

    let mut easy_api = EasyAPI::construct();
    easy_api.refresh().unwrap();

    let mut albums_data = Vec::new();
    easy_api.get_my_albums_chunk(0, &mut albums_data).unwrap();

    let mut current_artist = String::new();
    let mut current_track = String::new();
    if easy_api
        .get_currently_playing_artist(&mut current_artist)
        .is_err()
    {
        current_artist = "None".to_string();
    }
    if easy_api
        .get_currently_playing_track(&mut current_track)
        .is_err()
    {
        current_track = "None".to_string();
    }

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();
    // App
    let mut albums_pane = Albums::new(albums_data);
    let mut tracks_pane = Tracks::new();
    let mut tracks = Vec::new();
    let mut albums = Vec::new();
    let mut pages_loaded = 1;
    loop {
        let size = terminal.size()?;
        if size != albums_pane.size {
            terminal.resize(size)?;
            albums_pane.size = size;
        }

        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(size);

            let style = Style::default().fg(Color::Green).bg(Color::Rgb(25, 20, 20));

            let chunks_middle = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[0]);
            albums.clear();
            for album in albums_pane.albums.clone() {
                albums.push(album.name);
            }
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Albums"))
                .items(&albums)
                .select(albums_pane.selected)
                .style(style)
                .highlight_style(style.fg(Color::White).modifier(Modifier::Bold))
                .highlight_symbol(">")
                .render(&mut f, chunks_middle[0]);
            tracks.clear();
            for track in tracks_pane.tracks.clone() {
                tracks.push(track.name);
            }
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Tracks"))
                .items(&tracks)
                .select(tracks_pane.selected)
                .style(style)
                .highlight_style(style.fg(Color::White).modifier(Modifier::Bold))
                .highlight_symbol(">")
                .render(&mut f, chunks_middle[1]);

            let text = [
                Text::styled(
                    format!("Artist : {}\n", &current_artist.as_str()),
                    Style::default().fg(Color::White).modifier(Modifier::Bold),
                ),
                Text::styled(
                    format!("Track : {}", &current_track.as_str()),
                    Style::default().fg(Color::White),
                ),
            ];
            Paragraph::new(text.iter())
                .block(Block::default().title("Now Playing").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .wrap(true)
                .render(&mut f, chunks[1])
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Left => {
                    tracks_pane.selected = None;
                    albums_pane.selected = Some(0);
                }
                Key::Down => {
                    albums_pane.selected = if let Some(selected) = albums_pane.selected {
                        let mut end_of_albums = false;
                        if selected >= albums_pane.albums.len() - 1 {
                            // Adding next 20 albums to the list if there is some albums left
                            let before_size = albums_pane.albums.len();
                            easy_api
                                .get_my_albums_chunk(20 * pages_loaded, &mut albums_pane.albums)
                                .unwrap();
                            // if we got to the end of a user's album list
                            if before_size == albums_pane.albums.len() {
                                end_of_albums = true;
                            } else {
                                pages_loaded += 1;
                            }
                        }

                        if end_of_albums {
                            Some(0)
                        } else {
                            Some(selected + 1)
                        }
                    } else {
                        None
                    };
                    tracks_pane.selected = if let Some(selected) = tracks_pane.selected {
                        if selected >= tracks_pane.tracks.len() - 1 {
                            Some(0)
                        } else {
                            Some(selected + 1)
                        }
                    } else {
                        None
                    }
                }
                Key::Up => {
                    albums_pane.selected = if let Some(selected) = albums_pane.selected {
                        if selected > 0 {
                            Some(selected - 1)
                        } else {
                            Some(albums_pane.albums.len() - 1)
                        }
                    } else {
                        None
                    };

                    tracks_pane.selected = if let Some(selected) = tracks_pane.selected {
                        if selected > 0 {
                            Some(selected - 1)
                        } else {
                            Some(tracks_pane.tracks.len() - 1)
                        }
                    } else {
                        None
                    }
                }
                Key::Right => {
                    tracks_pane.selected = if let Some(selected) = tracks_pane.selected {
                        easy_api
                            .play_track_from_id(&tracks_pane.tracks[selected].id)
                            .unwrap();
                        Some(selected)
                    } else {
                        Some(1)
                    };

                    albums_pane.selected = if let Some(selected) = albums_pane.selected {
                        let mut tracks_added = Vec::new();
                        easy_api
                            .get_tracks_from_album(
                                &albums_pane.albums[selected].id,
                                &mut tracks_added,
                            ).unwrap();

                        tracks_pane.clear_tracks();
                        tracks_pane.add_tracks(&mut tracks_added);
                        tracks_pane.selected = Some(0);
                        None
                    } else {
                        None
                    };
                    albums_pane.selected = None
                }

                _ => {}
            },
            Event::Tick => {
                if easy_api
                    .get_currently_playing_artist(&mut current_artist)
                    .is_err()
                {
                    current_artist = "None".to_string();
                }
                if easy_api
                    .get_currently_playing_track(&mut current_track)
                    .is_err()
                {
                    current_track = "None".to_string();
                }
            }
        }
    }

    Ok(())
}
