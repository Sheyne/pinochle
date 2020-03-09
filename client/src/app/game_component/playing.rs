use pinochle_lib::{
    command::PlayingInput,
    game::{Game, Input},
    Card, Player, Suit,
};
use yew::callback::Callback;
use yew::events::InputData;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};
use yew::services::ConsoleService;

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    pub game: Game,
    pub ondo: Callback<PlayingInput>,
}

pub struct Data {
    bid: Option<usize>,
}

pub struct Playing {
    props: Props,
    link: ComponentLink<Self>,
    console: ConsoleService,

    data: Data,
}

#[derive(Debug)]
pub enum Msg {
    SetBid(Option<usize>),
    SubmitBid,
    SetTrump(Suit),
}

impl Component for Playing {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            console: ConsoleService::new(),
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
                { self.input() }
                { self.show_my_hand() }
            </div>
        }
    }

    // let hand = game.hand(*player);
    // let legality = hand.iter().map(|c| {
    //     is_current
    //         && c.map_or(false, |c| {
    //             is_legal(&game.play_area(), hand, &c, game.trump()).is_ok()
    //         })
    // });

    // html! {
    //     <div class={ if is_current { "active-player" } else { "" } } >
    //         <div>
    //             { "Position: " } { format!("{:?}", player) }
    //             { " Current Player: " } { format!("{:?}", state.turn()) }
    //         </div>
    //         <div id="hand">{ for hand.iter().zip(legality).map(|(c, playable)| self.to_html_playable(*c, playable)) }</div>
    //         <div id="play-area">{ for state.play_area().iter().map(|c| self.to_html(Some(*c))) }</div>
    //     </div>
    // }
}

impl Playing {
    fn input(&self) -> Html {
        match &self.props.game {
            Game::Bidding(state) => html! {
                <div>
                    <label for="bid"> { "Bid: " } </label>
                    <input id="bid" type="text" oninput=self.link.callback(|f: InputData|
                        Msg::SetBid(f.value.parse().map_or(None, |x| Some(x)))) />
                    <input type="button" value="Bid" onclick=self.link.callback(|_| Msg::SubmitBid) />
                </div>
            },
            Game::SelectingTrump(state) => html! {
                <div>
                    <input type="button" value="Diamonds" onclick=self.link.callback(|_| Msg::SetTrump(Suit::Diamond)) />
                    <input type="button" value="Clubs" onclick=self.link.callback(|_| Msg::SetTrump(Suit::Club)) />
                    <input type="button" value="Hearts" onclick=self.link.callback(|_| Msg::SetTrump(Suit::Heart)) />
                    <input type="button" value="Spades" onclick=self.link.callback(|_| Msg::SetTrump(Suit::Spade)) />
                </div>
            },
            Game::Playing(state) => html! {
                "Playing"
            },
            Game::FinishedRound(state) => html! {
                "FinishedRound"
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
                        // onclick=self.link.callback(move |e| Msg::PlayCard(card))
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

    fn show_my_hand(&self) -> Html {
        let players = [Player::A, Player::B, Player::C, Player::D];

        for player in players.iter() {
            if let Some(hand) = self.props.game.hand(*player) {
                if let Some(Some(card)) = hand.iter().next() {
                    return self.show_hand(*player);
                }
            }
        }
        html! { <div></div> }
    }
    fn show_hand(&self, player: Player) -> Html {
        if let Some(hand) = self.props.game.hand(player) {
            html! {
                <div>
                    {for hand.iter().map(|c| self.to_html(*c)) }
                </div>
            }
        } else {
            html! { <div></div> }
        }
    }

    fn to_html(&self, card: Option<Card>) -> Html {
        self.to_html_playable(card, false)
    }
}
