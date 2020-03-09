use std::string::ToString;
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
    SetPlayer(Player),
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
    fn show_ready(&self, ready: bool) -> Html {
        html! {
            <input type="checkbox"
                id="ready"
                checked={ready}
                onclick=self.link.callback(|_| Msg::ToggleReady)
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
            ToggleReady => panic!(),
            // self
            //     .props
            //     .ontablecommand
            //     .emit(TableCommand::SetReady(!self.props.state.ready)),
            SetPlayer(player) => self
                .props
                .ontablecommand
                .emit(TableCommand::SetPlayer(player)),
        }
        true
    }

    fn view(&self) -> Html {
        let players = vec![Player::A, Player::B, Player::C, Player::D];
        html! {
            <div>
                <label for="player">{ " Player: " } </label>

                <Select<Player> options=players
                                selected=self.props.state.player
                                onchange=self.link.callback(|e: Player| Msg::SetPlayer(e))
                                />

                <label for="ready">{ " Ready: " } </label>

                {
                    for self.props.state.ready.iter().map(|(p, r)| self.show_ready(*r))
                }
            </div>
        }
    }
}
