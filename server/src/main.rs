/*--------------------------------------------------------------------------------------------------------------
 * Copyright (c) Sheyne Anderson. All rights reserved.
 * Licensed under the MIT License.
 *-------------------------------------------------------------------------------------------------------------*/
extern crate ws;

use ws::listen;
extern crate pinochle_lib;
extern crate strum;

use pinochle_lib::{Card, Command, Rank, Response, Suit};
#[macro_use]
extern crate itertools;
use strum::IntoEnumIterator;

fn main() {
    listen("0.0.0.0:3012", |out| {
        let cards: Vec<Card> = iproduct!(Suit::iter(), Rank::iter())
            .map(|(s, r)| Card { suit: s, rank: r })
            .collect();

        move |msg| {
            match msg {
                ws::Message::Text(m) => {
                    let command: Command = serde_json::from_str(&m).unwrap();

                    println!("Command: {:?}", command)
                }
                ws::Message::Binary(_) => println!("Binary message"),
            }

            match serde_json::to_string(&Response::Update(cards.clone())) {
                Ok(j) => out.send(j),
                _ => panic!("Wat"),
            }
        }
    })
    .unwrap()
}

//  fn main() {
//     let cards : Vec<Card> = iproduct!(Suit::iter(), Rank::iter()).map(|(s, r)| Card {suit: s, rank: r}).collect();

//     print!("Cards: ");
//     for card in cards {
//         print!("{:?}, ", card);
//     }
//     println!();

//     let mut board = Board {
//         hands: [vec!(Card{
//             suit: Suit::Heart,
//             rank: Rank::Ten}), vec!(Card{suit: Suit::Diamond, rank: Rank::Ace}, Card{suit: Suit::Spade, rank: Rank::Ace}), vec!(), vec!()],
//         play_area: vec!(),
//         taken: [vec!(), vec!()],
//         trump: Suit::Spade,
//         turn: 0
//     };

//     board.play(Card{suit: Suit::Heart, rank: Rank::Ten}).unwrap();

//     println!("Play area: {:?}", board.play_area);

//     board.play(Card{suit: Suit::Spade, rank: Rank::Ace}).unwrap();

//     println!("Play area: {:?}", board.play_area);
// }
