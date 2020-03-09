use anyhow::Error;
use connect::{Connect, Server};
use pinochle_lib::command;
use ready::Ready;
use serde::Serialize;
use serde_json::from_str;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

mod connect;
mod ready;

#[derive(Debug)]
pub enum State {
    Initial,
    Connecting(String),
    AtTable(command::TableState),
}

pub struct App {
    console: ConsoleService,
    ws: Option<WebSocketTask>,
    wss: WebSocketService,
    link: ComponentLink<App>,

    state: State,
}

pub enum Msg {
    Disconnected,
    Connected,
    Received(Result<String, Error>),
    TableCommand(command::TableCommand),
    ConnectCommand(connect::Server, String),
}

impl App {
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
        let state = match &self.state {
            State::Connecting(_) | State::AtTable(_) => {
                self.console.log(&message);
                Some(State::AtTable(from_str(&message).unwrap()))
            }
            _ => None,
        };

        if let Some(state) = state {
            self.state = state;
        }

        true
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            ws: None,
            wss: WebSocketService::new(),
            console: ConsoleService::new(),
            link: link,

            state: State::Initial,
        }
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
        }
    }

    fn view(&self) -> Html {
        match &self.state {
            State::Initial => html! {
                <Connect server=Server::Localhost
                         table="table 1"
                         onsubmit=self.link.callback(|(server, table): (Server, String)| Msg::ConnectCommand(server, table)) />
            },
            State::Connecting(_) => html! {
                "Connecting"
            },
            State::AtTable(ts) => html! {
                <Ready state=ts ontablecommand=self.link.callback(|c: command::TableCommand| Msg::TableCommand(c)) />
            },
        }
    }
}

//            State::Ready => {
// let servers: Vec<Server> = Server::iter().collect();
// html! {
//     <div>
//     <Select<Server> options=servers selected=self.server onchange=self.link.callback(|e: Server| Msg::SelectServer(e)) />

//         <input
//             type="text"
//             value={&self.table}
//             oninput=self.link.callback(|e: InputData| Msg::SelectTable(e.value)) />
//         <button onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
//     </div>
// }
// }

// State::InGame(player, data) => {
// let is_current = *player == data.turn();

// match data {
//     Playing(state) => {
//         let hand = state.hand(*player);
//         let legality = hand.iter().map(|c| {
//             is_current
//                 && c.map_or(false, |c| {
//                     is_legal(&state.play_area(), hand, &c, state.trump()).is_ok()
//                 })
//         });

//         html! {
//             <div class={ if is_current { "active-player" } else { "" } } >
//                 <div>
//                     { "Position: " } { format!("{:?}", player) }
//                     { " Current Player: " } { format!("{:?}", state.turn()) }
//                 </div>
//                 <div id="hand">{ for hand.iter().zip(legality).map(|(c, playable)| to_html_playable(&self.link, *c, playable)) }</div>
//                 <div id="play-area">{ for state.play_area().iter().map(|c| to_html(&self.link, Some(*c))) }</div>
//             </div>
//         }
//     }
//     _ => html! {},
// }
// }

// fn to_html_playable(link: &ComponentLink<App>, card: Option<Card>, playable: bool) -> Html {
//     let card = card.clone();

//     match card {
//         Some(card) => {
//             let playable = if playable { " playable" } else { "" };
//             html! {
//                 <div class={ format!("suit-{} card{}", card.suit, playable) }
//                     onclick=link.callback(move |e| Msg::PlayCard(card)) >
//                 { card.to_string() }
//                 </div>
//             }
//         }
//         None => html! {
//             <div class={ "card" }>
//             { "U" }
//             </div>
//         },
//     }
// }

// fn to_html(link: &ComponentLink<App>, card: Option<Card>) -> Html {
//     to_html_playable(link, card, false)
// }
