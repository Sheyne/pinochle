use pinochle_lib::Card;
use yew::callback::Callback;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};

use super::card;

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    pub cards: Vec<(Card, bool)>,
    pub onchoose: Callback<Card>,
}

pub struct HandInput {
    props: Props,
    link: ComponentLink<Self>,
}

#[derive(Debug)]
pub enum Msg {
    Choose(Card),
}

impl Component for HandInput {
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
            Msg::Choose(c) => self.props.onchoose.emit(c),
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="hand">
            { for self.props.cards.iter().map(|(c, playable)|
                self.to_html_playable(*c, *playable)) }
            </div>
        }
    }
}

impl HandInput {
    fn to_html_playable(&self, card: Card, playable: bool) -> Html {
        html! {
            <card::Card
                card=card
                disabled=playable
                onchoose=if playable {
                            self.link.callback(move |e| Msg::Choose(card))
                        } else {
                            Callback::noop()
                        } />
        }
    }
}
