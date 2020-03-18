use pinochle_lib::Card;
use yew::callback::Callback;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};

use super::card;

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    pub cards: Vec<(Card, bool)>,
    #[prop_or_else(Callback::noop)]
    pub onchoose: Callback<(usize, Card)>,
    #[prop_or_else(Callback::noop)]
    pub onchoose_card: Callback<Card>,
    #[prop_or_else(Callback::noop)]
    pub onchoose_index: Callback<usize>,
}

pub struct HandInput {
    props: Props,
    link: ComponentLink<Self>,
}

#[derive(Debug)]
pub enum Msg {
    Choose(usize, Card),
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
            Msg::Choose(index, c) => {
                self.props.onchoose.emit((index, c));
                self.props.onchoose_card.emit(c);
                self.props.onchoose_index.emit(index);
            }
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="hand">
            { for self.props.cards.iter().map(|x| *x).enumerate().map(|(idx, (c, disabled))|
                html! {
                    <card::Card
                        card=c
                        disabled=disabled
                        onchoose=self.link.callback(move |e| Msg::Choose(idx, c)) />
                })
            }
            </div>
        }
    }
}
