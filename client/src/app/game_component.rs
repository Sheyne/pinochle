use anyhow::Error;
use connect::Connect;
pub use connect::Server;
use pinochle_lib::{command, game};
use playing::Playing;
use ready::Ready;
use serde::Serialize;
use serde_json::from_str;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

mod connect;
mod playing;
mod ready;

#[derive(Debug)]
pub enum State {
    Initial,
    Connecting(String),
    ReadyToGetTable,
    AtTable(command::TableState),
    ReadyToPlay,
    Playing(game::Game),
}

pub struct GameComponent {
    console: ConsoleService,
    ws: Option<WebSocketTask>,
    wss: WebSocketService,
    link: ComponentLink<GameComponent>,

    state: State,
    props: Props,
}

pub enum Msg {
    Disconnected,
    Connected,
    Received(Result<String, Error>),
    TableCommand(command::TableCommand),
    ConnectCommand(connect::Server, String),
    Do(command::PlayingInput),
}

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    pub table: String,
    pub server: Server,
}

impl GameComponent {
    fn send<T>(&mut self, t: T)
    where
        T: Serialize,
    {
        if let Some(ref mut task) = self.ws {
            task.send(Ok(serde_json::to_string(&t).unwrap()));
        }
    }

    fn got_message(&mut self, message: String) -> bool {
        self.console.log(&message);
        let state = match &mut self.state {
            State::Initial => None,
            State::Connecting(_) | State::ReadyToGetTable | State::AtTable(_) => {
                match from_str::<command::TableState>(&message) {
                    Ok(state) => {
                        if state.ready.iter().all(|(_, r)| *r) {
                            Some(State::ReadyToPlay)
                        } else {
                            Some(State::AtTable(state))
                        }
                    }
                    _ => None,
                }
            }
            State::ReadyToPlay => match from_str(&message) {
                Ok(game) => Some(State::Playing(game)),
                Err(_) => None,
            },
            State::Playing(game) => match from_str(&message) {
                Ok(command::PlayingResponse::BackToReady) => Some(State::ReadyToGetTable),
                Ok(command::PlayingResponse::Played(player, input)) => {
                    self.console.log(&format!(
                        "{} played: {:?}. {:?}",
                        player,
                        input.clone(),
                        game.play(input)
                    ));
                    None
                }
                Ok(command::PlayingResponse::Resigned(player)) => {
                    panic!("Resignation by {}", player);
                    None
                }
                Ok(command::PlayingResponse::Error(error)) => {
                    self.console.log(&format!("Error: {}", error));
                    None
                }
                Err(e) => {
                    self.console.log(&format!("{:?}", e));
                    None
                }
            },
        };

        if let Some(state) = state {
            self.state = state;
        }

        true
    }
}

impl Component for GameComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        GameComponent {
            ws: None,
            wss: WebSocketService::new(),
            console: ConsoleService::new(),
            link: link,

            props: props,
            state: State::Initial,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Connected => {
                match &self.state {
                    State::Connecting(table) => {
                        let table = table.clone();
                        self.send(command::Command::JoinTable(table));
                    }
                    a => self.console.log(&format!(
                        "Connected, but not in connecting state, in: {:?}",
                        a
                    )),
                }
                true
            }
            Msg::Disconnected => {
                self.ws = None;
                true
            }
            Msg::Received(Ok(response)) => self.got_message(response),
            Msg::Received(Err(response)) => {
                self.console.log(&format!("{:?}", response));
                true
            }
            Msg::ConnectCommand(server, table) => {
                self.console
                    .log(&format!("Connecting to: {} {}", server, table));

                let cbout = self.link.callback(|s| Msg::Received(s));
                let cbnot = self.link.callback(|input: WebSocketStatus| match input {
                    WebSocketStatus::Closed | WebSocketStatus::Error => Msg::Disconnected,
                    WebSocketStatus::Opened => Msg::Connected,
                });
                if self.ws.is_none() {
                    let task = self.wss.connect_text(&server.to_string(), cbout, cbnot);

                    match task {
                        Ok(t) => {
                            self.ws = Some(t);
                            self.state = State::Connecting(table);
                        }
                        Err(_) => self.ws = None,
                    }
                }

                true
            }

            Msg::TableCommand(command) => {
                self.console.log(&format!("Command: {:?}", command));

                self.send(command);

                false
            }

            Msg::Do(input) => {
                self.send(input);
                false
            }
        }
    }

    fn view(&self) -> Html {
        match &self.state {
            State::Initial => html! {
                <Connect server=self.props.server
                         table=self.props.table.clone()
                         onsubmit=self.link.callback(|(server, table): (Server, String)| Msg::ConnectCommand(server, table)) />
            },
            State::Connecting(_) | State::ReadyToGetTable => html! {
                <div> { "Connecting" } </div>
            },
            State::AtTable(ts) => html! {
                <Ready state=ts ontablecommand=self.link.callback(|c: command::TableCommand| Msg::TableCommand(c)) />
            },
            State::ReadyToPlay => html! {
                <div> { "Ready to play" } </div>
            },
            State::Playing(game) => html! {
                <Playing game=game ondo=self.link.callback(|d| Msg::Do(d)) />
            },
        }
    }
}
