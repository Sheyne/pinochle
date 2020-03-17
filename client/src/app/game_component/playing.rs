use pinochle_lib::{
    command::PlayingInput,
    game::{self, Game, Input},
    Card, Player, Suit,
};
use yew::callback::Callback;
use yew::events::InputData;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    pub game: Game,
    pub player: Player,
    pub ondo: Callback<PlayingInput>,
}

pub struct Data {
    bid: Option<usize>,
}

pub struct Playing {
    props: Props,
    link: ComponentLink<Self>,

    data: Data,
}

#[derive(Debug)]
pub enum Msg {
    SetBid(Option<usize>),
    SubmitBid,
    Pass,
    Play(Card),
    SetTrump(Suit),
    Next,
}

impl Component for Playing {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            data: Data { bid: None },
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetBid(b) => self.data.bid = b,
            Msg::SubmitBid => {
                self.data
                    .bid
                    .map(|b| self.props.ondo.emit(PlayingInput::Play(Input::Bid(b))));
            }
            Msg::Pass => {
                self.props.ondo.emit(PlayingInput::Play(Input::Pass));
            }
            Msg::Next => {
                self.props.ondo.emit(PlayingInput::Play(Input::Next));
            }
            Msg::Play(c) => {
                self.props.ondo.emit(PlayingInput::Play(Input::Play(c)));
            }
            Msg::SetTrump(t) => self
                .props
                .ondo
                .emit(PlayingInput::Play(Input::SelectSuit(t))),
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <div>{ self.props.player }</div>
                { self.input() }
                { self.show_hand(self.props.player) }
            </div>
        }
    }
}

impl Playing {
    fn input(&self) -> Html {
        match &self.props.game {
            Game::Bidding(_) => html! {
                <div>
                    <label for="bid"> { "Bid: " } </label>
                    <input id="bid" type="text" oninput=self.link.callback(|f: InputData|
                        Msg::SetBid(f.value.parse().map_or(None, |x| Some(x)))) />
                        <input type="button" value="Bid" onclick=self.link.callback(|_| Msg::SubmitBid) />
                        <input type="button" value="Pass" onclick=self.link.callback(|_| Msg::Pass) />
                    </div>
            },
            Game::SelectingTrump(_) => html! {
                <div>
                    <input type="button" value="Diamonds" onclick=self.link.callback(|_| Msg::SetTrump(Suit::Diamond)) />
                    <input type="button" value="Clubs" onclick=self.link.callback(|_| Msg::SetTrump(Suit::Club)) />
                    <input type="button" value="Hearts" onclick=self.link.callback(|_| Msg::SetTrump(Suit::Heart)) />
                    <input type="button" value="Spades" onclick=self.link.callback(|_| Msg::SetTrump(Suit::Spade)) />
                </div>
            },
            Game::PassingCards(_) => html! {
                "Passing"
            },
            Game::ReturningCards(_) => html! {
                "Returning"
            },
            Game::Playing(_) => self.show_play_area(),
            Game::FinishedRound(_) => html! {
                <input type="button" value="Next" onclick=self.link.callback(|_| Msg::Next) />
            },
            Game::Finished => html! {
                "Finished"
            },
        }
    }

    fn to_html_playable(&self, card: Option<Card>, playable: bool) -> Html {
        let card = card.clone();

        match card {
            Some(card) => {
                let playable = if playable { " playable" } else { "" };
                html! {
                    <div class={ format!("suit-{} card{}", card.suit, playable) }
                        onclick=self.link.callback(move |e| Msg::Play(card))
                        >
                    { card.to_string() }
                    </div>
                }
            }
            None => html! {
                <div class={ "card" }>
                { "U" }
                </div>
            },
        }
    }

    fn show_play_area(&self) -> Html {
        let game = &self.props.game;
        let play_area = game.playing().map(|s| s.play_area().clone());

        if let Some(play_area) = play_area {
            html! {
                <div id="play-area">{ for play_area.iter().map(|c| self.to_html_playable(Some(*c), false)) }</div>
            }
        } else {
            html! {}
        }
    }

    fn show_hand(&self, player: Player) -> Html {
        let game = &self.props.game;
        let is_current = game.can_play(player);

        let play_area = game.playing().map(|s| s.play_area().clone());
        let hand = game.hand(player);
        let trump = match game {
            Game::Playing(s) => Some(s.trump()),
            Game::FinishedRound(s) => Some(s.trump()),
            Game::PassingCards(s) => Some(s.trump()),
            Game::ReturningCards(s) => Some(s.trump()),
            Game::Bidding(_) | Game::Finished | Game::SelectingTrump(_) => None,
        };

        if let Some(hand) = hand {
            let check_legal = |c: Card| {
                if !is_current {
                    return false;
                }

                if let Some(play_area) = play_area {
                    if let Some(trump) = trump {
                        return game::states::is_legal(play_area, hand, &c, trump).is_ok();
                    }
                }
                false
            };

            let cards_playable = hand.iter().map(|c| (c, c.map_or(false, check_legal)));

            html! {
                <div id="hand">{ for cards_playable.map(|(c, playable)| self.to_html_playable(*c, playable)) }</div>
            }
        } else {
            html! {}
        }
    }
}
