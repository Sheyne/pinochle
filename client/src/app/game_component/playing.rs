use pinochle_lib::{
    command::PlayingInput,
    game::{self, Game, Input},
    Card, Player, Suit,
};
use std::convert::TryInto;
use std::string::ToString;
use strum::IntoEnumIterator;
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
        let is_turn = self.props.game.can_play(self.props.player);
        let hand = self.props.game.hand(self.props.player);
        let needs_show_hand = hand.is_some() && !(is_turn && self.already_showing_hand());

        let hand: Option<Vec<(Card, bool)>> = if needs_show_hand {
            hand.map(|hand| hand.iter().filter_map(|x| x.map(|x| (x, true))).collect())
        } else {
            None
        };

        let play_area = if let Game::Playing(game) = &self.props.game {
            html! {
                <div>
                    <h2>{ "Play area:" }</h2>
                    <div> { "We've taken " } { game.taken(self.props.player.team()).len() } { " cards. They've taken " } {
                        game.taken(self.props.player.team().other()).len()
                    } {" cards."} </div>
                    <div id="play-area">{
                        for game.play_area().iter().map(|c|
                            html! { <card::Card card=c /> })
                    }</div>
                </div>
            }
        } else {
            html! {}
        };

        let can_play: Vec<_> = Player::iter()
            .filter(|p| self.props.game.can_play(*p))
            .map(|x| x.to_string())
            .collect();
        let can_play = if can_play.len() == 0 {
            "no one".to_owned()
        } else {
            let head = &can_play[0..can_play.len() - 1];
            let last = &can_play[can_play.len() - 1];

            if head.len() == 0 {
                last.to_owned()
            } else {
                format!("{}, and {}", &head.join(", "), last)
            }
        };

        html! {
            <div>
                <div> { "You are " } { self.props.player } { ". Currently  "} { can_play } { " can play." } </div>
                {if is_turn { html! {
                    <div>
                        <h2>{"Input: "}</h2>
                        { self.input(self.props.player) }
                    </div>
                }} else { html!{}}}
                {if let Some(hand) = hand { html! {
                    <div>
                        <h2>{"Hand: "}</h2>
                        <HandInput cards=hand />
                    </div>
                }} else { html!{}}}
                { play_area }
            </div>
        }
    }
}

impl Playing {
    fn already_showing_hand(&self) -> bool {
        match &self.props.game {
            Game::Bidding(_) => false,
            Game::SelectingTrump(_) => false,
            Game::PassingCards(_) => true,
            Game::ReturningCards(_) => true,
            Game::Playing(_) => true,
            Game::FinishedRound(_) => true,
            Game::Finished => true,
        }
    }

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
                            !game::states::is_legal(game.play_area(), hand, &card, trump).is_ok(),
                        )
                    })
                    .collect();
                html! {
                    <HandInput cards=cards onchoose_card=self.link.callback(|c: Card|
                        PlayingInput::Play(Input::Play(c))) />
                }
            }
            Game::FinishedRound(state) => html! {
                <div>
                    <h2>{ "Us" }</h2>
                    <div> {
                        for state.taken(current_player.team()).iter().map(|c|
                            html! { <card::Card card=c /> })
                    } </div>
                    <h2>{ "Them" }</h2>
                    <div> {
                        for state.taken(current_player.team().other()).iter().map(|c|
                            html! { <card::Card card=c /> })
                    } </div>
                    <input type="button" value="Next" onclick=self.link.callback(|_|
                        PlayingInput::Play(Input::Next)) />
                </div>
            },
            Game::Finished => html! {
                "Finished"
            },
        }
    }
}
