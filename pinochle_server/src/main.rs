/*--------------------------------------------------------------------------------------------------------------
 * Copyright (c) Sheyne Anderson. All rights reserved.
 * Licensed under the MIT License.
 *-------------------------------------------------------------------------------------------------------------*/
 extern crate pinochle_lib;
 extern crate strum;

 use strum::IntoEnumIterator;
 use pinochle_lib::{Card, Rank, Suit, Board};
 #[macro_use] extern crate itertools;

 fn main() {
    let cards : Vec<Card> = iproduct!(Suit::iter(), Rank::iter()).map(|(s, r)| Card {suit: s, rank: r}).collect();

    print!("Cards: ");
    for card in cards {
        print!("{:?}, ", card);
    }
    println!();

    let mut board = Board {
        hands: [vec!(Card{ 
            suit: Suit::Heart,
            rank: Rank::Ten}), vec!(Card{suit: Suit::Diamond, rank: Rank::Ace}, Card{suit: Suit::Spade, rank: Rank::Ace}), vec!(), vec!()],
        play_area: vec!(),
        taken: [vec!(), vec!()],
        trump: Suit::Spade,
        turn: 0
    };

    board.play(Card{suit: Suit::Heart, rank: Rank::Ten}).unwrap();

    println!("Play area: {:?}", board.play_area);

    board.play(Card{suit: Suit::Spade, rank: Rank::Ace}).unwrap();

    println!("Play area: {:?}", board.play_area);
}