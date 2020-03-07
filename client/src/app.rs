use anyhow::Error;
use std::string::ToString;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use yew::components::Select;
use yew::events::InputData;
use yew::format::Json;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

use pinochle_lib::{
    states::is_legal, Card, Command, Game, Message as GameMessage, Player, Playing,
    PlayingInput::Play, Response,
};

#[derive(Display, PartialEq, Clone, EnumIter, Copy)]
pub enum Server {
    #[strum(serialize = "ws://localhost:3011/")]
    Localhost,
    #[strum(serialize = "wss://pinochle.herokuapp.com/")]
    Heroku,
}

enum State {
    InGame(Player, Game),
    Ready,
}

pub struct App {
    console: ConsoleService,
    table: String,
    server: Server,
    ws: Option<WebSocketTask>,
    wss: WebSocketService,
    link: ComponentLink<App>,

    state: State,
}

pub enum Msg {
    Connect,
    Disconnected,
    Connected,
    PlayCard(Card),
    Received(Result<Response, Error>),
    SelectTable(String),
    SelectServer(Server),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            table: "".to_string(),
            server: Server::Localhost,
            state: State::Ready,
            ws: None,
            wss: WebSocketService::new(),
            console: ConsoleService::new(),
            link: link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Connect => {
                let cbout = self.link.callback(|Json(data)| Msg::Received(data));
                let cbnot = self.link.callback(|input: WebSocketStatus| match input {
                    WebSocketStatus::Closed | WebSocketStatus::Error => Msg::Disconnected,
                    WebSocketStatus::Opened => Msg::Connected,
                });
                if self.ws.is_none() {
                    let task = self
                        .wss
                        .connect_text(&self.server.to_string(), cbout, cbnot);

                    match task {
                        Ok(t) => self.ws = Some(t),
                        Err(_) => self.ws = None,
                    }
                }
                true
            }
            Msg::Connected => {
                match &mut self.ws {
                    Some(task) => {
                        match serde_json::to_string(&Command::Connect(self.table.to_string())) {
                            Ok(a) => {
                                self.console.log(&format!("sending {}", a));
                                task.send(Ok(a));
                            }
                            Err(e) => self.console.log(&format!("Err {}", e)),
                        }
                    }
                    _ => (),
                }
                false
            }
            Msg::Disconnected => {
                self.ws = None;
                true
            }
            Msg::PlayCard(card) => {
                if let Some(ref mut task) = self.ws {
                    task.send(Ok(
                        serde_json::to_string(&GameMessage::Play(Play(card))).unwrap()
                    ));
                }
                false
            }
            Msg::Received(Ok(response)) => {
                // match response {
                //     Response::Update(data) => {
                //         self.state = State::Playing(data);
                //     }
                //     Response::Error(e) => {
                //         self.console.log(&format!("Error: {:?}", &e));
                //     }
                // }
                true
            }
            Msg::Received(Err(s)) => {
                self.console
                    .log(&format!("Error when reading data from server: {}", &s));
                false
            }
            Msg::SelectTable(x) => {
                self.table = x;
                false
            }
            Msg::SelectServer(x) => {
                self.server = x;
                self.console.log(&format!("Server: {}", self.server));
                false
            }
        }
    }

    fn view(&self) -> Html {
        match &self.state {
            State::Ready => {
                let servers: Vec<Server> = Server::iter().collect();
                html! {
                    <div>
                    <Select<Server> options=servers selected=self.server onchange=self.link.callback(|e: Server| Msg::SelectServer(e)) />

                        <input
                            type="text"
                            value={&self.table}
                            oninput=self.link.callback(|e: InputData| Msg::SelectTable(e.value)) />
                        <button onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
                    </div>
                }
            }

            State::InGame(player, data) => {
                let is_current = *player == data.turn();

                match data {
                    Playing(state) => {
                        let hand = state.hand(*player);
                        let legality = hand.iter().map(|c| {
                            is_current
                                && c.map_or(false, |c| {
                                    is_legal(&state.play_area(), hand, &c, state.trump()).is_ok()
                                })
                        });

                        html! {
                            <div class={ if is_current { "active-player" } else { "" } } >
                                <div>
                                    { "Position: " } { format!("{:?}", player) }
                                    { " Current Player: " } { format!("{:?}", state.turn()) }
                                </div>
                                <div id="hand">{ for hand.iter().zip(legality).map(|(c, playable)| to_html_playable(&self.link, *c, playable)) }</div>
                                <div id="play-area">{ for state.play_area().iter().map(|c| to_html(&self.link, Some(*c))) }</div>
                            </div>
                        }
                    }
                    _ => html! {},
                }
            }
        }
    }
}

fn to_html_playable(link: &ComponentLink<App>, card: Option<Card>, playable: bool) -> Html {
    let card = card.clone();

    match card {
        Some(card) => {
            let playable = if playable { " playable" } else { "" };
            html! {
                <div class={ format!("suit-{} card{}", card.suit, playable) }
                    onclick=link.callback(move |e| Msg::PlayCard(card)) >
                { card.to_string() }
                </div>
            }
        }
        None => html! {
            <div class={ "card" }>
            { "U" }
            </div>
        },
    }
}

fn to_html(link: &ComponentLink<App>, card: Option<Card>) -> Html {
    to_html_playable(link, card, false)
}
