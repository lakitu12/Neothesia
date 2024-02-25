use iced_core::{
    alignment::{Horizontal, Vertical},
    Alignment, Length, Padding,
};
use iced_style::Theme;
use iced_widget::{button, column as col, container, row, vertical_space, Component};

use crate::{context::Context, scene::menu_scene::icons, song::PlayerConfig};
use neothesia_iced_widgets::{BarLayout, Element, Layout, NeoBtn, Renderer};

use super::{centered_text, theme};

pub struct TracksPage<'a, MSG> {
    ctx: &'a mut Context,
    on_back: Option<Box<dyn Fn() -> MSG>>,
    on_play: Option<Box<dyn Fn() -> MSG>>,
}

impl<'a, MSG> TracksPage<'a, MSG> {
    pub fn new(ctx: &'a mut Context) -> Self {
        Self {
            ctx,
            on_back: None,
            on_play: None,
        }
    }

    pub fn on_back(mut self, cb: impl Fn() -> MSG + 'static) -> Self {
        self.on_back = Some(Box::new(cb));
        self
    }

    pub fn on_play(mut self, cb: impl Fn() -> MSG + 'static) -> Self {
        self.on_play = Some(Box::new(cb));
        self
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Back,
    Play,
    AllTracksPlayer(PlayerConfig),
    TrackPlayer(usize, PlayerConfig),
    TrackVisibility(usize, bool),
}

impl<'a, MSG> Component<MSG, Theme, Renderer> for TracksPage<'a, MSG> {
    type State = ();
    type Event = Event;

    fn update(&mut self, _state: &mut Self::State, event: Self::Event) -> Option<MSG> {
        match event {
            Event::AllTracksPlayer(config) => {
                if let Some(song) = self.ctx.song.as_mut() {
                    for track in song.config.tracks.iter_mut() {
                        track.player = config.clone();
                    }
                }
            }
            Event::TrackPlayer(track, config) => {
                if let Some(song) = self.ctx.song.as_mut() {
                    song.config.tracks[track].player = config;
                }
            }
            Event::TrackVisibility(track, visible) => {
                if let Some(song) = self.ctx.song.as_mut() {
                    song.config.tracks[track].visible = visible;
                }
            }
            Event::Back => return self.on_back.as_ref().map(|cb| cb()),
            Event::Play => return self.on_play.as_ref().map(|cb| cb()),
        }

        None
    }

    fn view(&self, _state: &Self::State) -> Element<'a, Self::Event> {
        let ctx = &self.ctx;
        let mut tracks = Vec::new();
        if let Some(song) = ctx.song.as_ref() {
            for track in song.file.tracks.iter().filter(|t| !t.notes.is_empty()) {
                let config = &song.config.tracks[track.track_id];

                let visible = config.visible;

                let active = match config.player {
                    PlayerConfig::Mute => 0,
                    PlayerConfig::Auto => 1,
                    PlayerConfig::Human => 2,
                };

                let color = if !visible {
                    iced_core::Color::from_rgb8(102, 102, 102)
                } else {
                    let color_id = track.track_color_id % ctx.config.color_schema.len();
                    let color = &ctx.config.color_schema[color_id].base;
                    iced_core::Color::from_rgb8(color.0, color.1, color.2)
                };

                let name = if track.has_drums && !track.has_other_than_drums {
                    "Percussion"
                } else {
                    let instrument_id = track
                        .programs
                        .last()
                        .map(|p| p.program as usize)
                        .unwrap_or(0);
                    midi_file::INSTRUMENT_NAMES[instrument_id]
                };

                let body = neothesia_iced_widgets::SegmentButton::new()
                    .button(
                        "Mute",
                        Event::TrackPlayer(track.track_id, PlayerConfig::Mute),
                    )
                    .button(
                        "Auto",
                        Event::TrackPlayer(track.track_id, PlayerConfig::Auto),
                    )
                    .button(
                        "Human",
                        Event::TrackPlayer(track.track_id, PlayerConfig::Human),
                    )
                    .active(active)
                    .active_color(color);

                let card = neothesia_iced_widgets::TrackCard::new()
                    .title(name)
                    .subtitle(format!("{} Notes", track.notes.len()))
                    .track_color(color)
                    .body(body);

                let card = if track.has_drums && !track.has_other_than_drums {
                    card
                } else {
                    card.on_icon_press(Event::TrackVisibility(track.track_id, !visible))
                };

                tracks.push(card.into());
            }
        }

        let column = neothesia_iced_widgets::Wrap::with_elements(tracks)
            .spacing(14.0)
            .line_spacing(14.0)
            .padding(30.0)
            .align_items(Alignment::Center);

        let column = col![vertical_space().height(Length::Fixed(30.0)), column]
            .align_items(Alignment::Center)
            .width(Length::Fill);

        let column = iced_widget::scrollable(column);

        let right = {
            let play = NeoBtn::new(
                icons::play_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::Play);

            if ctx.song.is_some() {
                row![play]
            } else {
                row![]
            }
            .spacing(10)
            .width(Length::Shrink)
            .align_items(Alignment::Center)
        };

        let right = container(right)
            .width(Length::Fill)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            });

        let left = {
            let back = NeoBtn::new(
                icons::left_arrow_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::Back);

            row![back].align_items(Alignment::Start)
        };

        let left = container(left)
            .width(Length::Fill)
            .align_x(Horizontal::Left)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 10.0,
            });

        let center = {
            let listen = button(centered_text("Listen Only"))
                .on_press(Event::AllTracksPlayer(PlayerConfig::Auto))
                .style(theme::button());

            let play_along = button(centered_text("Play Along"))
                .on_press(Event::AllTracksPlayer(PlayerConfig::Human))
                .style(theme::button());

            row![listen, play_along]
                .width(Length::Shrink)
                .align_items(Alignment::Center)
                .spacing(14)
        };

        let center = container(center)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            });

        Layout::new()
            .body(column)
            .bottom(BarLayout::new().left(left).center(center).right(right))
            .into()
    }
}

impl<'a, MSG> From<TracksPage<'a, MSG>> for Element<'a, MSG>
where
    MSG: 'a,
{
    fn from(page: TracksPage<'a, MSG>) -> Self {
        iced_widget::component(page)
    }
}
