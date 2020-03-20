use yew::callback::Callback;
use yew::events::InputData;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};
use yew::services::ConsoleService;

#[derive(Debug)]
pub enum Msg {
    Connect,
    SetTable(String),
}

use Msg::*;

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    pub table: String,

    pub onsubmit: Callback<String>,
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
        self.console
            .log(&format!("{:?} {}", msg, self.props.table.clone()));

        match msg {
            Connect => {
                self.props.onsubmit.emit(self.props.table.clone());
            }
            SetTable(table) => self.props.table = table,
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <input
                    type="text"
                    value=&self.props.table
                    oninput=self.link.callback(|e: InputData| Msg::SetTable(e.value)) />
                <button onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
            </div>
        }
    }
}
