use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use yew::callback::Callback;
use yew::components::Select;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};

use pinochle_lib::{
    command::{TableCommand, TableState},
    Player,
};

pub enum Msg {
    ToggleReady,
    SetPlayer(Option<Player>),
}

#[derive(Display, PartialEq, Clone, EnumIter, Debug, Copy)]
enum PlayerOption {
    Unset,
    A,
    B,
    C,
    D,
}

impl From<Option<Player>> for PlayerOption {
    fn from(p: Option<Player>) -> Self {
        match p {
            Some(Player::A) => PlayerOption::A,
            Some(Player::B) => PlayerOption::B,
            Some(Player::C) => PlayerOption::C,
            Some(Player::D) => PlayerOption::D,
            None => PlayerOption::Unset,
        }
    }
}

impl From<PlayerOption> for Option<Player> {
    fn from(p: PlayerOption) -> Self {
        match p {
            PlayerOption::A => Some(Player::A),
            PlayerOption::B => Some(Player::B),
            PlayerOption::C => Some(Player::C),
            PlayerOption::D => Some(Player::D),
            PlayerOption::Unset => None,
        }
    }
}

use Msg::*;

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    pub state: TableState,

    pub ontablecommand: Callback<TableCommand>,
}

pub struct Ready {
    props: Props,
    link: ComponentLink<Self>,
}

impl Ready {
    fn show_ready(&self, clickable: bool, ready: bool) -> Html {
        html! {
            <input type="checkbox"
                id="ready"
                checked={ready}
                disabled=!clickable
                onclick=if clickable { self.link.callback(|_| Msg::ToggleReady) } else { Callback::noop() }
                />
        }
    }
}

impl Component for Ready {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { props, link }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ToggleReady => {
                self.props.ontablecommand.emit(TableCommand::SetReady(
                    self.props
                        .state
                        .player
                        .map_or(false, |p| !self.props.state.ready.get_value(p)),
                ));
            }
            SetPlayer(player) => {
                player.map(|player| {
                    self.props
                        .ontablecommand
                        .emit(TableCommand::SetPlayer(player))
                });
            }
        }
        true
    }

    fn view(&self) -> Html {
        let players: Vec<PlayerOption> = PlayerOption::iter().collect();
        html! {
            <div>
                <label for="player">{ " Player: " } </label>

                <Select<PlayerOption> options=players
                                selected=PlayerOption::from(self.props.state.player)
                                onchange=self.link.callback(|e: PlayerOption| Msg::SetPlayer(e.into()))
                                />

                <label for="ready">{ " Ready: " } </label>

                {
                    for self.props.state.ready.iter().map(|(p, r)| self.show_ready(
                        self.props.state.player.map_or(false, |pl| pl == p), *r))
                }
            </div>
        }
    }
}
