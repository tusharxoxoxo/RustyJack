use std::ops::AddAssign;

use crate::{
    card_manager::{Card, Shoe},
    player_manager::{self, Player, Players, Hand},
};

pub fn increase_bet(player: &mut Player) {
    if player.bank_balance > 0 && player.can_change_bet {
        player.bet += 1;
        player.bank_balance -= 1;
    }
}

pub fn decrease_bet(player: &mut Player) {
    if player.bet > 0 && player.can_change_bet {
        player.bet -= 1;
        player.bank_balance += 1;
    }
}

pub fn hit(player: &mut Player, shoe: &mut Shoe) {
    let which_hand = player.which_hand_being_played;
    if !player.is_bust
        && get_hand_value(&player.hands[which_hand].hand) != 21
        && !player.has_checked
    {
        let mut card = shoe.draw_card();
        let index = player.hands[which_hand].hand.len();

        let mut coords = player.hands[which_hand].hand[index - 1].coords;
        coords.0 += 20;
        coords.1 -= 20;
        card.coords = coords;
        player.hands[which_hand].hand.push(card);
    }
}

pub fn double(player: &mut Player) {
    if !player.has_checked {
        player.bet *= 2
    }
}

pub fn stand(dealer: &mut Player, shoe: &mut Shoe) {
    while get_hand_value(&dealer.hands[0].hand) < 17 {
        let mut card = shoe.draw_card();
        let index = dealer.hands[0].hand.len();

        let mut coords = dealer.hands[0].hand[index - 1].coords;
        coords.0 -= 20;
        coords.1 += 20;
        card.coords = coords;
        dealer.hands[0].hand.push(card);
        change_aces(dealer);
    }

    dealer.has_finished_dealing = true;

    if get_hand_value(&dealer.hands[0].hand) > 21 {
        dealer.is_bust = true;
    }
}

pub fn split(player: &mut Player, shoe: &mut Shoe) {
    if player.hands.len() < 4 {
        split_hands(player, shoe);
        change_coords_of_split_hands(player);
    }
}

pub fn check_for_blackjack_and_bust(player: &mut Player) {
    let which_hand = player.which_hand_being_played;
    change_aces(player);

    if get_hand_value(&player.hands[which_hand].hand) > 21 && !player.is_bust {
        player.is_bust = true;
        player.bank_balance -= player.bet;
    } else if get_hand_value(&player.hands[which_hand].hand) == 21
        && player.hands[which_hand].hand.len() <= 2
    {
        player.has_blackjack = true;
        player.has_checked = true;
        player.has_won = true;
    } else if get_hand_value(&player.hands[which_hand].hand) == 21 && !player.is_bust {
        player.has_checked = true;
    }
}

pub fn change_aces(player: &mut Player) {
    let which_hand = player.which_hand_being_played;

    let has_ace = check_for_ace(&player.hands[which_hand].hand);
    let mut hand_val = get_hand_value(&player.hands[which_hand].hand);

    if hand_val > 21 && has_ace {
        'change_ace: loop {
            for i in 0..player.hands[which_hand].hand.len() {
                if player.hands[which_hand].hand[i].value == 11 {
                    player.hands[which_hand].hand[i].value = 1;
                    hand_val = get_hand_value(&player.hands[which_hand].hand);
                    if hand_val < 21 {
                        break 'change_ace;
                    }
                }
            }
        }
    }
}

pub fn check_for_ace(hand: &Vec<Card>) -> bool {
    for i in 0..hand.len() {
        if hand[i].value == 11 {
            return true;
        }
    }
    return false;
}

pub fn check_for_winner(players: &mut Players) {
    if !player_manager::check_if_hand_can_be_split(&players.players[0].hands[0].hand) {
        if players.players[0].has_blackjack {
            players.players[0].bet *= 2;
            update_player_winnings(players);
            players.players[0].has_blackjack = false;
        } else {
            let player_hand_val = get_hand_value(&players.players[0].hands[0].hand);
            let dealer_hand_val = get_hand_value(&players.dealer.hands[0].hand);

            if player_hand_val > dealer_hand_val && !players.players[0].is_bust
                || players.dealer.is_bust
            {
                players.players[0].has_won = true;
            } else if dealer_hand_val > player_hand_val && !players.dealer.is_bust {
                players.dealer.has_won = true;
            } else if player_hand_val == dealer_hand_val && !players.players[0].is_bust
                || !players.dealer.is_bust
            {
                players.players[0].has_won = true;
                players.dealer.has_won = true;
            }
            update_player_winnings(players);
        }
    } else {
    }
}

pub fn update_player_winnings(players: &mut Players) {
    if players.players[0].has_won {
        players.players[0].bank_balance += players.players[0].bet
    } else if players.dealer.has_won {
        players.players[0].bank_balance -= players.players[0].bet
    } else if players.players[0].has_won && players.players[0].has_blackjack {
        players.players[0].bank_balance += players.players[0].bet * 2
    } else if players.players[0].has_won && players.dealer.has_won {
    }
}

pub fn deal_again(players: &mut Players, shoe: &mut Shoe, window_size: &(u32, u32)) {
    players.dealer.hands[0].hand.drain(..);
    players.dealer.hands[0].hand.push(shoe.draw_card());
    players.dealer.has_won = false;
    players.dealer.is_bust = false;
    players.dealer.has_finished_dealing = false;

    for i in 0..players.players.len() {
        players.players[i].bet = 20;
        players.players[i].hands.drain(..);
        players.players[i].hands[0].hand.push(shoe.draw_card());
        players.players[i].has_won = false;
        players.players[i].has_checked = false;
        players.players[i].is_bust = false;
        players.players[i].can_change_bet = true;
        players.players[i].has_blackjack = false;
    }

    players.deal_cards(shoe, &window_size);
}

pub fn get_hand_value(hand: &Vec<Card>) -> u8 {
    let mut hand_val = 0;

    for i in 0..hand.len() {
        hand_val += hand[i].value;
    }

    hand_val
}

pub fn split_hands(player: &mut Player, shoe: &mut Shoe) {
        if player_manager::check_if_hand_can_be_split(&player.hands[player.which_hand_being_played].hand) {
            if let Some(card) = player.hands[player.which_hand_being_played].hand.pop() {
                let new_hand = Hand { hand: vec![card] };
                player.hands.push(new_hand);
                player.hands[player.which_hand_being_played].hand.push(shoe.shoe[10].clone());
                player.which_hand_being_played += 1;
                player.hands[player.which_hand_being_played].hand.push(shoe.shoe[10].clone());
            }
        }
}

pub fn change_coords_of_split_hands(player: &mut Player) {
    let offset = player.window_size.0 / 25;
    let coords = player.hands[0].hand[0].coords;

    let coords_of_bottom_hand_left = (coords.0 - offset, coords.1 + offset);
    let coords_of_bottom_hand_right = (coords.0 + offset, coords.1 + offset);
    let coords_of_top_hand_left = (coords.0 - offset, coords.1 - offset);
    let coords_of_top_hand_right = (coords.0 + offset, coords.1 - offset);

    match player.hands.len() {
        2 => {
            player.hands[0].hand[0].coords = coords_of_bottom_hand_left;
            player.hands[0].hand[1].coords = (coords_of_bottom_hand_left.0 + 20, coords_of_bottom_hand_left.1 - 20);
            player.hands[1].hand[0].coords = coords_of_bottom_hand_right;
            player.hands[1].hand[1].coords = (coords_of_bottom_hand_right.0 + 20, coords_of_bottom_hand_right.1 - 20);
        },
        3 => {
            player.hands[1].hand[0].coords = coords_of_bottom_hand_right;
            player.hands[1].hand[1].coords = (coords_of_bottom_hand_right.0 + 20, coords_of_bottom_hand_right.1 - 20);
            player.hands[2].hand[0].coords = coords_of_top_hand_left;
            player.hands[2].hand[1].coords = (coords_of_top_hand_left.0 + 20, coords_of_top_hand_left.1 - 20);
        },
        4 => {
            player.hands[2].hand[0].coords = coords_of_top_hand_left;
            player.hands[2].hand[1].coords = (coords_of_top_hand_left.0 + 20, coords_of_top_hand_left.1 - 20);
            player.hands[3].hand[0].coords = coords_of_top_hand_right;
            player.hands[3].hand[1].coords = (coords_of_top_hand_right.0 + 20, coords_of_top_hand_right.1 - 20);
        },
        _ => {}
    }

}