use pinochle_lib::{
    command::PlayingInput,
    game::{self, Game, Input},
    Card, Player, Suit,
};
use std::convert::TryInto;
use yew::callback::Callback;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};

use super::component::bid::BidInput;
use super::component::card;
use super::component::hand::HandInput;
use super::component::pass_cards::PassCardsInput;

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    pub game: Game,
    pub player: Player,
    pub ondo: Callback<PlayingInput>,
}

pub struct Playing {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for Playing {
    type Message = PlayingInput;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { props, link }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props.ondo.emit(msg);
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <div>{ self.props.player }</div>
                <h2>{"Input: "}</h2>
                { self.input(self.props.player) }
                <h2>{"Play Area: "}</h2>
                { self.show_play_area() }
            </div>
        }
    }
}

impl Playing {
    fn input(&self, current_player: Player) -> Html {
        match &self.props.game {
            Game::Bidding(s) => {
                let min_bid = s
                    .bids()
                    .iter()
                    .filter_map(|x| *x)
                    .map(|x| TryInto::<i32>::try_into(x))
                    .filter_map(|x| Result::ok(x))
                    .max()
                    .map_or(250, |x| x + 25);

                html! {
                    <BidInput increment=Some(25)
                              min_amount=min_bid
                              onsubmit=self.link.callback(|b: Option<i32>|
                                PlayingInput::Play(match b {
                                    Some(b) => Input::Bid(b.try_into().unwrap()),
                                    None => Input::Pass,
                                })) />
                }
            }
            Game::SelectingTrump(_) => html! {
                <div>
                    <input type="button" value="Diamonds" onclick=self.link.callback(|_|
                        PlayingInput::Play(Input::SelectSuit(Suit::Diamond))) />
                    <input type="button" value="Clubs" onclick=self.link.callback(|_|
                        PlayingInput::Play(Input::SelectSuit(Suit::Club))) />
                    <input type="button" value="Hearts" onclick=self.link.callback(|_|
                        PlayingInput::Play(Input::SelectSuit(Suit::Heart))) />
                    <input type="button" value="Spades" onclick=self.link.callback(|_|
                        PlayingInput::Play(Input::SelectSuit(Suit::Spade))) />
                </div>
            },
            Game::PassingCards(state) => {
                let cards: Vec<Card> = state
                    .hand(current_player)
                    .iter()
                    .filter_map(|x| *x)
                    .collect();
                html! {
                    <PassCardsInput
                        number=4
                        cards=cards
                        onpass=self.link.callback(|c: Vec<Card>| {
                            if let [a,b,c,d] = c[..] {
                                PlayingInput::Play(Input::PassCards(Some([a,b,c,d])))
                            } else {
                                panic!("Wrong number of cards passed");
                            }
                        }) />
                }
            }
            Game::ReturningCards(state) => {
                let cards: Vec<Card> = state
                    .hand(current_player)
                    .iter()
                    .filter_map(|x| *x)
                    .collect();
                html! {
                    <PassCardsInput
                        number=4
                        cards=cards
                        onpass=self.link.callback(|c: Vec<Card>| {
                            if let [a,b,c,d] = c[..] {
                                PlayingInput::Play(Input::PassCards(Some([a,b,c,d])))
                            } else {
                                panic!("Wrong number of cards passed");
                            }
                        }) />
                }
            }
            Game::Playing(game) => {
                let hand = game.hand(current_player);
                let trump = game.trump();
                let cards: Vec<(Card, bool)> = hand
                    .iter()
                    .filter_map(|card| *card)
                    .map(|card| {
                        (
                            card,
                            game::states::is_legal(game.play_area(), hand, &card, trump).is_ok(),
                        )
                    })
                    .collect();
                html! {
                    <HandInput cards=cards onchoose=self.link.callback(|c: Card|
                        PlayingInput::Play(Input::Play(c))) />
                }
            }
            Game::FinishedRound(_) => html! {
                <input type="button" value="Next" onclick=self.link.callback(|_|
                    PlayingInput::Play(Input::Next)) />
            },
            Game::Finished => html! {
                "Finished"
            },
        }
    }

    fn show_play_area(&self) -> Html {
        let game = &self.props.game;
        let play_area = game.playing().map(|s| s.play_area().clone());

        if let Some(play_area) = play_area {
            html! {
                <div id="play-area">{
                    for play_area.iter().map(|c|
                        html! { <card::Card card=c /> })
                }</div>
            }
        } else {
            html! {}
        }
    }
}
