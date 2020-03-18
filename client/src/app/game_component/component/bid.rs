use yew::callback::Callback;
use yew::events::InputData;
use yew::html::{Component, ComponentLink, Html, ShouldRender};
use yew::macros::{html, Properties};

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props<Bid>
where
    Bid: Clone,
{
    #[prop_or(true)]
    pub optional: bool,
    #[prop_or_default]
    pub min_amount: Option<Bid>,
    #[prop_or_default]
    pub max_amount: Option<Bid>,
    #[prop_or_default]
    pub increment: Option<Bid>,
    #[prop_or_else(Callback::noop)]
    pub onsubmit: Callback<Option<Bid>>,
}

struct Data<Bid> {
    bid: Option<Bid>,
}

pub struct BidInput {
    props: Props<i32>,
    data: Data<i32>,
    link: ComponentLink<Self>,
}

#[derive(Debug)]
pub enum Msg<Bid>
where
    Bid: std::str::FromStr,
{
    SetBid(Option<Bid>),
    SubmitBid,
    Pass,
}

impl Component for BidInput {
    type Message = Msg<i32>;
    type Properties = Props<i32>;
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
            Msg::SetBid(b) => {
                self.data.bid = b
                    .filter(|b| self.props.min_amount.map_or(true, |min| b >= &min))
                    .filter(|b| self.props.max_amount.map_or(true, |max| b <= &max))
                    .filter(|b| {
                        self.props.increment.map_or(true, |increment| {
                            (b - self.props.min_amount.unwrap_or(0)) % increment == 0
                        })
                    });
                true
            }
            Msg::SubmitBid => {
                self.data
                    .bid
                    .as_ref()
                    .map(|v| self.props.onsubmit.emit(Some(v.clone())));
                false
            }
            Msg::Pass => {
                if self.props.optional {
                    self.props.onsubmit.emit(None);
                }
                false
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <label for="bid"> { "Bid: " } </label>
                <input id="bid" type="text" oninput=self.link.callback(|f: InputData|
                    Msg::SetBid(f.value.parse().map_or(None, |x| Some(x)))) />
                <input type="button" value="Bid" disabled=self.data.bid.is_none()
                                                 onclick=self.link.callback(|_| Msg::SubmitBid) />
                {
                    if self.props.optional {
                        html!{<input type="button" value="Pass" onclick=self.link.callback(|_| Msg::Pass) />}
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }
}
