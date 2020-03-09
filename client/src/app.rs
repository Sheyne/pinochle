mod game_component;
use game_component::{GameComponent, Server};
use yew::prelude::*;

pub struct App {
    link: ComponentLink<App>,
    table: String,
}

pub enum Msg {
    SetTable(String),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link,
            table: "table".to_owned(),
        }
    }

    fn update(&mut self, m: Self::Message) -> ShouldRender {
        match m {
            Msg::SetTable(t) => self.table = t,
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <input type="text" oninput=self.link.callback(|t: InputData| Msg::SetTable(t.value)) />
                <GameComponent table=self.table.clone()
                               server=Server::Localhost />
                <GameComponent table=self.table.clone()
                               server=Server::Localhost />
                <GameComponent table=self.table.clone()
                               server=Server::Localhost />
                <GameComponent table=self.table.clone()
                               server=Server::Localhost />
            </div>
        }
    }
}
