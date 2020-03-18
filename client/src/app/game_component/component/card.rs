use yew::callback::Callback;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    #[prop_or(false)]
    pub disabled: bool,
    pub card: pinochle_lib::Card,
    #[prop_or_else(Callback::noop)]
    pub onchoose: Callback<()>,
}

pub struct Card {
    props: Props,
    link: ComponentLink<Self>,
}

#[derive(Debug)]
pub enum Msg {
    Choose,
}

impl Component for Card {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { props, link }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Choose => self.props.onchoose.emit(()),
        }
        false
    }

    fn view(&self) -> Html {
        let class = {
            let disabled = if self.props.disabled { " disabled" } else { "" };
            format!("suit-{} card{}", self.props.card.suit, disabled)
        };
        html! {
            <div class=class
                onclick=self.link.callback(move |e| Msg::Choose)>
            { self.props.card.to_string() }
            </div>
        }
    }
}
