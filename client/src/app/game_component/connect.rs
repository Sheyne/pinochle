use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use yew::callback::Callback;
use yew::components::Select;
use yew::events::InputData;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};
use yew::services::ConsoleService;

#[derive(Display, PartialEq, Clone, EnumIter, Debug, Copy)]
pub enum Server {
    #[strum(serialize = "ws://localhost:3011/")]
    Localhost,
    #[strum(serialize = "wss://pinochle.herokuapp.com/")]
    Heroku,
}

#[derive(Debug)]
pub enum Msg {
    Connect,
    SetTable(String),
    SetServer(Server),
}

use Msg::*;

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    pub server: Server,
    pub table: String,

    pub onsubmit: Callback<(Server, String)>,
}

pub struct Connect {
    props: Props,
    link: ComponentLink<Self>,
    console: ConsoleService,
}

impl Component for Connect {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            console: ConsoleService::new(),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.console.log(&format!("{:?}", props));
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.console.log(&format!(
            "{:?}, ({}, {})",
            msg,
            self.props.server,
            self.props.table.clone()
        ));

        match msg {
            Connect => {
                self.props
                    .onsubmit
                    .emit((self.props.server, self.props.table.clone()));
            }
            SetServer(server) => self.props.server = server,
            SetTable(table) => self.props.table = table,
        }
        false
    }

    fn view(&self) -> Html {
        let servers: Vec<Server> = Server::iter().collect();
        html! {
            <div>
            <Select<Server> options=servers selected=self.props.server onchange=self.link.callback(|e: Server| Msg::SetServer(e)) />

                <input
                    type="text"
                    value=&self.props.table
                    oninput=self.link.callback(|e: InputData| Msg::SetTable(e.value)) />
                <button onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
            </div>
        }
    }
}
