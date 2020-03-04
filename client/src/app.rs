use anyhow::Error;
use yew::events::InputData;
use yew::format::Json;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

use pinochle_lib::{is_legal, Action, Card, Command, PlayerData, Response};

enum State {
    Playing(PlayerData),
    Ready,
}

pub struct App {
    console: ConsoleService,
    name: String,
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
    EnterName(String),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            name: "".to_string(),
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
                    let task = self.wss.connect_text("ws://localhost:3012/", cbout, cbnot);

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
                        match serde_json::to_string(&Command::Connect(self.name.to_string())) {
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
                    task.send(Ok(serde_json::to_string(&Action::PlayCard(card)).unwrap()));
                }
                false
            }
            Msg::Received(Ok(response)) => {
                match response {
                    Response::Update(data) => {
                        self.state = State::Playing(data);
                    }
                    Response::Error(e) => {
                        self.console.log(&format!("Error: {:?}", &e));
                    }
                }
                true
            }
            Msg::Received(Err(s)) => {
                self.console
                    .log(&format!("Error when reading data from server: {}", &s));
                false
            }
            Msg::EnterName(x) => {
                self.name = x;
                false
            }
        }
    }

    fn view(&self) -> Html {
        match &self.state {
            State::Ready => html! {
                <div>
                    <input
                        type="text"
                        value={&self.name}
                        oninput=self.link.callback(|e: InputData| Msg::EnterName(e.value)) />
                    <button onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
                </div>
            },

            State::Playing(data) => {
                let is_current = data.player == data.turn;

                let hand = data.hand.iter().map(|c| {
                    if is_current {
                        (
                            c,
                            match is_legal(&data.play_area, &data.hand, c, data.trump) {
                                Err(_) => false,
                                Ok(_) => true,
                            },
                        )
                    } else {
                        (c, false)
                    }
                });

                html! {
                    <div class={ if is_current { "active-player" } else { "" } } >
                        <div>
                            { "Position: " } { data.player }
                            { " Current Player: " } { data.turn }
                        </div>
                        <div id="hand">{ for hand.map(|(c, playable)| to_html_playable(&self.link, c, playable)) }</div>
                        <div id="play-area">{ for data.play_area.iter().map(|c| to_html(&self.link, c)) }</div>
                    </div>
                }
            }
        }
    }
}

fn to_html_playable(link: &ComponentLink<App>, card: &Card, playable: bool) -> Html {
    let card = card.clone();

    let playable = if playable { " playable" } else { "" };

    html! {
        <div class={ format!("suit-{} card{}", card.suit, playable) }
             onclick=link.callback(move |e| Msg::PlayCard(card)) >
        { card.to_string() }
        </div>
    }
}

fn to_html(link: &ComponentLink<App>, card: &Card) -> Html {
    to_html_playable(link, card, false)
}
