use std::{array::from_fn, collections::VecDeque, mem};

use rand::prelude::*;
use colored::Colorize;



const TEAM_COUNT : usize = 10;
const PLAYER_COUNT : usize = 3 * TEAM_COUNT;
fn main() {
    // create arrays
    let mut players : [Player; PLAYER_COUNT] = from_fn(|i| Player::create(i, PLAYER_COUNT));
    let mut teams : [Team; TEAM_COUNT] = from_fn(|_| Team::default());

    // to see how the algorithm scales
    let mut total_iterations = 0;

    /*** BEGIN MAIN ALGORITHM: ***/

    // while there is a non-teamed player...
    while let Some(player_id) = next(&players) {
        print!("Player {} is sending proposals to:\n", player_id);

        // they will make a proposal to players (a,b)
        // this list is a preference of pairs- 
        // that preference prioritizes having a single more preferenced player:
        // p : a > b > c > d,
        // then p : (a,b) > (a,c) > (a,d)
        // and p : (a,d) > (b,c) 
        'propose_to_preferred: while let Some((a,b)) 
            = players[player_id].preferred_pairs.pop_front() // this proposal is only made once.
        {

            total_iterations += 1;

            print!("\t Players {} and {}\t|", a, b);
            let a_pair = (b, player_id);
            let b_pair = (a, player_id);

            // both a and b have their own pair-preference list.
            let a_prefers = players[a].prefers_team(a_pair, &teams);
            let b_prefers = players[b].prefers_team(b_pair, &teams);

            // if the proposed team is higher on their list for BOTH, accept / swap.
            if a_prefers && b_prefers {
                print!("{}\n", " Accepted.".green());
                
                // fully dissolve their pre-existing teams.
                // the other players will not be on a team any more.
                dissolve_player_team(a, &mut players, &mut teams);
                dissolve_player_team(b, &mut players, &mut teams);

                // build the new proposed team.
                create_team(player_id,a,b, &mut players, &mut teams);
                break 'propose_to_preferred;
            } else {
                print!("{}{}{}{}\n", " Rejected by ".red(), 
                    if !a_prefers { a.to_string().red() } else { "".red() }, 
                    if a_prefers == b_prefers { " and ".red() } else { "".red() }, 
                    if !b_prefers { b.to_string().red() } else { "".red() });
            }
        }
        print!("\n");
    }

    /*** END MAIN ALGORITHM ***/

    // Print the results:
    println!("{}", "Players:".bold());
    for i in 0..players.len() {
        print!("  {}\t{}", format!("Player {}", i).bright_cyan(),"|".dimmed());
        let team = &teams[players[i].team.unwrap()];
        for other in &players[i].preference_list {
            if team.on_the_team(*other) {
                print!("{}, ", (*other).to_string().green());
            } else {
                print!("{}, ", *other);
            }
        }
        print!("\n");
    }
    
    println!("Matched Teams:");
    for i in 0..teams.len() {
        println!("\tTeam {}: {:?}, {:?}, {:?}", i, teams[i].0.unwrap(), teams[i].1.unwrap(), teams[i].2.unwrap());
    }

    println!("{}", format!("Total Iterations: {}", total_iterations).bright_purple());
}


// you can see the exact implementation details below:

struct Player {
    pub id: usize,
    pub team: Option<usize>,
    pub preference_list: Vec<usize>,
    pub preferred_pairs: VecDeque<(usize,usize)>,
}
impl Player {
    pub fn create(id: usize, total: usize) -> Self {
        let preference_list = {
            let mut list : Vec<usize> = (0..total).collect();
            list.remove(id);

            let mut rng = rand::thread_rng();
            for i in 0..(total-1) {
                list.swap(i, rng.gen_range(i..(total-1)));
            }
            list
        };
        let mut preferred_pairs = VecDeque::with_capacity(preference_list.len() * (preference_list.len() + 1) / 2);
        for i in 0..(preference_list.len()-1) {
            for j in i+1..preference_list.len() {
                preferred_pairs.push_back((preference_list[i],preference_list[j]));
            }
        }
        Self {
            id,
            preference_list,
            preferred_pairs,
            team: None
        }
    }


    pub fn prefers(&self, player_a: Option<usize>, player_b: Option<usize>) -> Option<bool> {
        match player_a {
            None => match player_b {
                None => None, // neither, so don't bother
                Some(_) => Some(false) // b is someone, so prefer
            },
            Some(player_a_id) => match player_b {
                None => Some(true), // only a is someone, so prefer
                Some(player_b_id) => {
                    // the preferred one is the first one on the list
                    for i in 0..self.preference_list.len() {
                        if self.preference_list[i] == player_a_id {
                            return Some(true);
                        } else if self.preference_list[i] == player_b_id {
                            return Some(false);
                        }
                    }
                    None
                },
            },
        }        
    }

    pub fn preference_order(&self, team: &Team) -> (Option<usize>, Option<usize>, Option<usize>) {
        let mut most = team.0;
        let mut mid = team.1;
        let mut least = team.2;
        if let Some(true) = self.prefers(least, mid) {
            mem::swap(&mut mid, &mut least);
        }
        if let Some(true) = self.prefers(mid, most) {
            mem::swap(&mut mid, &mut most);
        }
        if let Some(true) = self.prefers(least, mid) {
            mem::swap(&mut mid, &mut least);
        }
        (most, mid, least)
    }

    pub fn not_me(&self, team: &Team) -> (Option<usize>, Option<usize>) {
        let (a,b,c) = self.preference_order(team);
        (a,b)
    }

    pub fn prefers_team(&self, mut others: (usize, usize), teams: &[Team]) -> bool {
        if self.team.is_none() {
            return true;
        }

        if let Some(_) = self.prefers(Some(others.1), Some(others.0)) {
            others = (others.1, others.0);
        }

        let current_team = self.not_me(&teams[self.team.unwrap()]);   

        if current_team.0.is_none() || current_team.1.is_none() {
            return true;
        }

        let current_team = (current_team.0.unwrap(), current_team.1.unwrap());

        for i in 0..self.preferred_pairs.len() {
            if self.preferred_pairs[i] == others {
                return true;
            }
            if self.preferred_pairs[i] == current_team {
                return false;
            }
        }

        return false;
    }
}

struct Team(pub Option<usize>, pub Option<usize>, pub Option<usize>);
impl Default for Team {
    fn default() -> Self {
        Self(None, None, None)
    }
}
impl Team {
    pub fn has_room(&self) -> bool {
        self.0.is_none() || self.1.is_none() || self.2.is_none()
    }

    pub fn on_the_team(&self, player: usize) -> bool {
        if let Some(player_0) = self.0 {
            if player_0 == player {
                return true;
            }
        }
        if let Some(player_1) = self.1 {
            if player_1 == player {
                return true;
            }
        }
        if let Some(player_2) = self.2 {
            if player_2 == player {
                return true;
            }
        }
        false
    }
}

fn next(on_players: &[Player]) -> Option<usize> {
    for i in 0..on_players.len() {
        if on_players[i].team.is_none() && on_players[i].preferred_pairs.len() > 0 {
            return Some(i);
        }
    }
    None
    //Some((last_player + 1) % on_players.len())
}

fn dissolve_player_team(player: usize, players: &mut [Player], teams: &mut [Team]) {
    match players[player].team {
        None => (),
        Some(team) => dissolve_team(team, players, teams)
    }
}

fn dissolve_team(team: usize, players: &mut [Player], teams: &mut [Team]) {
    let team = &mut teams[team];
    if let Some(p0) = team.0 {
        players[p0].team = None;
        team.0 = None;
    }
    if let Some(p1) = team.1 {
        players[p1].team = None;
        team.1 = None;
    }
    if let Some(p2) = team.2 {
        players[p2].team = None;
        team.2 = None;
    }
}

fn create_team(p0: usize, p1 : usize, p2 : usize, players: &mut [Player], teams: &mut [Team]) {
    if players[p0].team.is_some() {
        panic!("bad!")
    }
    if players[p1].team.is_some() {
        panic!("bad!")
    }
    if players[p1].team.is_some() {
        panic!("bad!")
    }

    for i in 0..teams.len() {
        if teams[i].has_room() {
            dissolve_team(i, players, teams);
            teams[i] = Team(Some(p0),Some(p1),Some(p2));
            players[p0].team = Some(i);
            players[p1].team = Some(i);
            players[p2].team = Some(i);
            return;
        }
    }
}
