extern crate pinochle_lib;

use yew::prelude::*;
use failure::Error;
use yew::format::Json;
use yew::services::ConsoleService;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use serde::{Deserialize, Serialize};

use pinochle_lib::{Card, Rank, Suit, Response, Command};

pub struct App {
    console: ConsoleService,
    pending_message: String,
    ws: Option<WebSocketTask>,
    wss: WebSocketService,
    link: ComponentLink<App>,
}

pub enum Msg {
    Connect,
    Disconnected,
    Send,
    Received(Result<Response, Error>),
    TextInput(String),
    None
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            pending_message: "".to_string(),
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
				let cbnot = self.link.send_back(|input| {
					match input {
						WebSocketStatus::Closed | WebSocketStatus::Error => {
							Msg::Disconnected
						}
						_ => Msg::None,
					}
				});
				if self.ws.is_none() {
					let task = self.wss.connect("ws://localhost:3012/", cbout, cbnot.into());
					self.ws = Some(task);
				}
				true
			}
			Msg::Disconnected => {
				self.ws = None;
				true
            }
            Msg::Send => {
				match self.ws {
					Some(ref mut task) => {
                        match serde_json::to_string(&Command::PlayCard(Card {suit: Suit::Heart, rank: Rank::Nine}, self.pending_message.clone())) {
                            Ok(a) => {
                                task.send(Ok(a))
                            }
                            Err(e) => self.console.log(&format!("Err {}", e)),
                        }

                        self.console.log(&format!("Sending"));
						self.pending_message = "".to_string();
						true
					}
					None => {
						false
					}
				}
            }
			Msg::Received(Ok(s)) => {
                self.console.log(&format!("Received: {:?}", &s));
                false
			}
			Msg::Received(Err(s)) => {
				self.console.log(&format!("Error when reading data from server: {}", &s));
				false
			}
            Msg::TextInput(x) => {
                self.pending_message = x;
                false
            }
            Msg::None => false
        }
    }
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <input
                    type="text"
                    value={&self.pending_message}
                    oninput=|e| Msg::TextInput(e.value) />
                <button onclick=|_| Msg::Send,>{ "Send" }</button>
                <button onclick=|_| Msg::Connect,>{ "Connect" }</button>
                </div>
        }
    }
}
