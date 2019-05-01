use crate::interaction::wait::{Action, WaitEvent};
use crate::models::bank::Bank;
use crate::schema::banks::dsl::*;
use crate::DatabaseConnection;
use crate::Waiter;
use chrono::prelude::*;
use diesel::prelude::*;
use rand::prelude::*;
use serenity::model::{channel::Message, channel::ReactionType, id::ChannelId, id::MessageId};
use serenity::utils::{content_safe, ContentSafeOptions};

command!(play(ctx, msg, args) {
    let mut rng = rand::thread_rng();
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.channel_id.say("Datenbankfehler, bitte informiere einen Moderator!");
            return Ok(());
        }
    };
    let amount_to_bet = match args.single::<i64>() {
        Ok(v) if v > 0 => v,
        Ok(_) => {
            // log
            let _ = msg.channel_id.say("Ungültiger Wetteinsatz!");
            return Ok(());
        }
        Err(e) => {
            // log
            let _ = msg.channel_id.say("Ungültiger Wetteinsatz!");
            return Ok(());
        }
    };
    // check if user already owns a bank & has enough balance
    let results = banks.filter(user_id.eq(*msg.author.id.as_u64() as i64)).load::<Bank>(&*conn.lock()).expect("could not retrieve banks");
    
    if !results.is_empty() && results[0].amount >= amount_to_bet {
        // roll
        let full_reels: Vec<Vec<i64>> = (0..3)
            .map(|_| {
                let roll = rng.gen_range(0, 7);
                let prev;
                let next;
                if roll == 6 {
                    prev = 5;
                    next = 0;
                } else if roll == 0 {
                    prev = 6;
                    next = 1;
                } else {
                    prev = roll - 1;
                    next = roll + 1;
                }
                vec![prev, roll, next]
            })
            .collect();
    
        let payout = get_payout(&full_reels, amount_to_bet);
        let delta = payout - amount_to_bet;
        let updated_amount =  results[0].amount + delta;

        // TODO: investigate why the aschangeset version does not work
        diesel::update(banks.filter(id.eq(results[0].id))).set(amount.eq(updated_amount)).execute(&*conn.lock()).expect("failed update bank");

        let slot_machine_output = display_reels(&full_reels, payout, updated_amount);
        let _ = msg.channel_id.send_message(|m| m.embed(|e| 
                e.description(&slot_machine_output)
                .color((0,120,220))));
    } else {
        let _ = msg.channel_id.say("Du besitzt entweder keine Bank, oder nicht genügend credits!");
    }
    
});

fn get_payout(full_reels: &Vec<Vec<i64>>, betted_amount: i64) -> i64 {
    // win condition 1
    if full_reels[0][1] == full_reels[1][1] && full_reels[1][1] == full_reels[2][1] {
        // win 1
        4 * betted_amount
    } else if full_reels[0][0] == full_reels[1][1] && full_reels[1][1] == full_reels[2][2] {
        // win 2
        3 * betted_amount
    } else if full_reels[0][2] == full_reels[1][1] && full_reels[1][1] == full_reels[2][0] {
        // win 3
        2 * betted_amount
    } else {
        0
    }
}

fn display_reels(full_reels: &Vec<Vec<i64>>, payout: i64, updated_amount: i64) -> String {
    format!(
        "{} | {} | {} \n{} | {} | {}\n {} | {} | {}\n\n Gewonnen: {}\nBank: {}",
        number_to_emoji(full_reels[0][0]),
        number_to_emoji(full_reels[1][0]),
        number_to_emoji(full_reels[2][0]),
        number_to_emoji(full_reels[0][1]),
        number_to_emoji(full_reels[1][1]),
        number_to_emoji(full_reels[2][1]),
        number_to_emoji(full_reels[0][2]),
        number_to_emoji(full_reels[1][2]),
        number_to_emoji(full_reels[2][2]),
        payout,
        updated_amount
    )
}

fn number_to_emoji(n: i64) -> &'static str {
    match n {
        0 => "🧀",
        1 => "🍉",
        2 => "🍒",
        3 => "🥝",
        4 => "🍩",
        5 => "🥔",
        _ => "🍆",
    }
}

#[cfg(test)]
mod tests {
    use super::get_payout;
    use rand::Rng;

    #[test]
    fn test_reels() {
        let mut rng = rand::thread_rng();

        let full_reels: Vec<Vec<i32>> = (0..3)
            .map(|_| {
                let roll = rng.gen_range(0, 7);
                let prev;
                let next;
                if roll == 6 {
                    prev = 5;
                    next = 0;
                } else if roll == 0 {
                    prev = 6;
                    next = 1;
                } else {
                    prev = roll - 1;
                    next = roll + 1;
                }
                vec![prev, roll, next]
            })
            .collect();

        dbg!(full_reels);
    }

    #[test]
    fn test_payout() {
        let mut rng = rand::thread_rng();

        let full_reels_1 = vec![vec![0, 1, 2], vec![0, 1, 2], vec![0, 1, 2]];
        let full_reels_2 = vec![vec![0, 1, 2], vec![6, 0, 1], vec![5, 6, 0]];
        let full_reels_3 = vec![vec![0, 1, 2], vec![1, 2, 3], vec![2, 3, 4]];

        dbg!(&full_reels_1);
        dbg!(get_payout(&full_reels_1, 10));

        dbg!(&full_reels_2);
        dbg!(get_payout(&full_reels_2, 20));

        dbg!(&full_reels_3);
        dbg!(get_payout(&full_reels_3, 30));
    }
}