use pinochle_lib::Card;
use std::collections::BTreeSet;
use yew::callback::Callback;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};

use super::card;

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    pub number: usize,
    pub cards: Vec<Card>,
    pub onpass: Callback<Vec<Card>>,
}

pub struct PassCardsInput {
    props: Props,
    selected: BTreeSet<usize>,
    link: ComponentLink<Self>,
}

#[derive(Debug)]
pub enum Msg {
    Toggle(usize),
    Submit,
}

impl Component for PassCardsInput {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            selected: BTreeSet::new(),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Submit => {
                if self.props.number == self.selected.len() {
                    self.props
                        .onpass
                        .emit(self.selected.iter().map(|i| self.props.cards[*i]).collect())
                }
            }
            Msg::Toggle(idx) => {
                if self.selected.contains(&idx) {
                    self.selected.remove(&idx);
                } else {
                    self.selected.insert(idx);
                }
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="hand">
                { for self.props.cards.iter().enumerate().map(|(idx, c)|
                    html! {
                        <card::Card
                            card=c
                            selected=self.selected.contains(&idx)
                            onchoose=self.link.callback(move |_| {
                                Msg::Toggle(idx)
                            }) />
                    }
                ) }
                <br />
                <input type="button"
                       disabled=self.props.number != self.selected.len()
                       value="Pass"
                       onclick=self.link.callback(|_| Msg::Submit) />
            </div>
        }
    }
}
