mod game_component;
use game_component::GameComponent;
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
        let window = web_sys::window().expect("no global `window` exists");
        let is_local = window.location().hostname().expect("has host") == "localhost";
        let num_repeats = if is_local { 4 } else { 1 };

        let input = if is_local {
            html! {
                <input type="text" oninput=self.link.callback(|t: InputData| Msg::SetTable(t.value)) />
            }
        } else {
            html! {}
        };

        html! {
            <div>
                {input}

                {for (0..num_repeats).map(|_| html!{
                    <GameComponent table=self.table.clone() />
                    })}
            </div>
        }
    }
}
