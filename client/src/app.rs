use failure::Error;
use yew::format::Json;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

use pinochle_lib::{Action, Card, Command, PlayerData, Rank, Response, Suit};

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
    Send,
    Received(Result<Response, Error>),
    EnterName(String),
    None,
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
                self.console.log("Connecting");
                let cbout = self.link.send_back(|Json(data)| Msg::Received(data));
                let cbnot = self.link.send_back(|input| match input {
                    WebSocketStatus::Closed | WebSocketStatus::Error => Msg::Disconnected,
                    WebSocketStatus::Opened => Msg::Connected,
                });
                if self.ws.is_none() {
                    let task = self
                        .wss
                        .connect("ws://localhost:3012/", cbout, cbnot.into());

                    self.ws = Some(task);
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
            Msg::Send => match self.ws {
                Some(ref mut task) => {
                    match serde_json::to_string(&Command::Action(Action::PlayCard(Card {
                        suit: Suit::Heart,
                        rank: Rank::Nine,
                    }))) {
                        Ok(a) => task.send(Ok(a)),
                        Err(e) => self.console.log(&format!("Err {}", e)),
                    }
                    false
                }
                None => false,
            },
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
            Msg::None => false,
        }
    }
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        match &self.state {
            State::Ready => html! {
                <div>
                    <input
                        type="text"
                        value={&self.name}
                        oninput=|e| Msg::EnterName(e.value) />
                    <button onclick=|_| Msg::Connect>{ "Connect" }</button>
                    </div>
            },

            State::Playing(data) => {
                html! {
                    <div>{ data.hand[0].to_string() }</div>
                }
            }
        }
    }
}
