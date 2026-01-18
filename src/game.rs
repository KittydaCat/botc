use rand::{Rng, SeedableRng, rngs::SmallRng};

#[derive(Clone, Debug)]
pub struct Player {
    role: Role,
    alive: bool,
    notes: Vec<Note>,
    ghost_vote: bool,
}

pub type PlayerId = usize;

pub enum Never {}

impl Into<()> for Never {
    fn into(self) -> () {
        unreachable!()
    }
}

#[enum_ids::enum_ids(derive = "Clone, Copy, PartialEq, Debug")]
#[derive(Clone, Debug, Copy)]
pub enum Role {
    // townsfolk
    WasherWoman,
    Librarian,
    Investigator,
    Chef,
    Empath,
    FortuneTeller { red_herring: PlayerId },
    Undertaker { last_exec: Option<RoleId> },
    Monk,
    RavensKeeper,
    Virgin { ability_used: bool },
    Slayer { ability_used: bool },
    Soldier,
    Mayor,

    // outsiders
    Butler { butlered: Option<PlayerId> },
    Drunk { role: RoleId },
    Recluse,
    Saint,

    // minions
    Poisoner,
    Spy,
    ScarletWoman,
    Baron,

    // demon
    Imp, // { bluffs: [Option<RoleId>; 3] },
}

impl RoleId {
    pub fn all() -> Vec<RoleId> {
        vec![
            RoleId::WasherWoman,
            RoleId::Librarian,
            RoleId::Investigator,
            RoleId::Chef,
            RoleId::Empath,
            RoleId::FortuneTeller,
            RoleId::Undertaker,
            RoleId::Monk,
            RoleId::RavensKeeper,
            RoleId::Virgin,
            RoleId::Slayer,
            RoleId::Soldier,
            RoleId::Mayor,
            RoleId::Butler,
            RoleId::Drunk,
            RoleId::Recluse,
            RoleId::Saint,
            RoleId::Poisoner,
            RoleId::Spy,
            RoleId::ScarletWoman,
            RoleId::Baron,
            RoleId::Imp,
        ]
    }

    fn is_townsfolk(self) -> bool {
        match self {
            RoleId::WasherWoman
            | RoleId::Librarian
            | RoleId::Investigator
            | RoleId::Chef
            | RoleId::Empath
            | RoleId::FortuneTeller
            | RoleId::Undertaker
            | RoleId::Monk
            | RoleId::RavensKeeper
            | RoleId::Virgin
            | RoleId::Slayer
            | RoleId::Soldier
            | RoleId::Mayor => true,
            _ => false,
        }
    }

    fn is_outsider(self) -> bool {
        match self {
            RoleId::Butler | RoleId::Drunk | RoleId::Recluse | RoleId::Saint => true,
            _ => false,
        }
    }

    fn is_minion(self) -> bool {
        match self {
            RoleId::Poisoner | RoleId::Spy | RoleId::ScarletWoman | RoleId::Baron => true,
            _ => false,
        }
    }

    fn is_demon(self) -> bool {
        self == RoleId::Imp
    }

    fn is_good(self) -> bool {
        self.is_townsfolk() == self.is_outsider()
    }

    fn is_evil(self) -> bool {
        self.is_outsider() == self.is_demon()
    }
}

// actions indicate what the player does and what the effect is
//
// drunk should be recreated by prior actions
//
// only truly random actions need to be cataloged

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Action {
    // startup
    // Init(Vec<Player>), // should be implicit
    // MinionInfo(PlayerId),
    ImpInfo(
        [Option<RoleId>; 3],
        // minions: Vec<PlayerId>,
    ),

    // role inits
    WasherWoman {
        players: [PlayerId; 2],
        role: RoleId,
    },
    Librarian {
        players: [PlayerId; 2],
        role: RoleId,
    },
    Investigator {
        players: [PlayerId; 2],
        role: RoleId,
    },
    Chef {
        pairs: u32,
    },

    // each night
    Poisoner {
        target: PlayerId,
    },
    Monk {
        protected: PlayerId,
    },
    ImpKill(PlayerId),
    ImpTransfer(PlayerId),
    RavensKeeper {
        target: PlayerId,
        response: RoleId,
    },
    Empath(u32),
    FortuneTeller {
        target: [PlayerId; 2],
        response: bool,
    },
    Butler(PlayerId),
    Undertaker(RoleId),
    Spy(Vec<Player>),

    MayorDeflection(PlayerId),

    // day abilities
    // Virgin(PlayerId),
    // Slayer(PlayerId),

    // victories
    // TownsfolkWin,
    // MayorWin,
    // ImpWin,

    // other things
    Slay(usize, usize),
    Nomination(PlayerId, PlayerId),
    Vote([bool; 15]),
    // Died(PlayerId),
    // Executed(PlayerId),
}

// each is attached to a player
#[derive(Clone, Debug, PartialEq)]
enum Note {
    Poisoned,
    MonkProtected,
    DiedTonight,
    ExecToday,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Info {
    Number(u32),
    Bool(bool),
    Player(PlayerId),
    Role(RoleId),

    // this needs to include all state in the grim
    // if the grim is updated to keep state in other fields this will have to be chnaged
    Grim(Vec<Player>),
    Slays(PlayerId, PlayerId),
    Day,
    Night,
}

// pub struct GrimIO {
//     pub tell: Box<dyn FnMut(PlayerId, Info)>,
//     pub prompt_player: Box<dyn FnMut(PlayerId) -> PlayerId>,
//     pub prompt_player_optional: Box<dyn FnMut(PlayerId) -> Option<PlayerId>>,
//     pub prompt_tell: Box<dyn FnMut(PlayerId) -> Option<(PlayerId, PlayerId, RoleId)>>,
//     pub win: Box<dyn FnMut(bool) -> Never>,
// }
//
// impl std::fmt::Debug for GrimIO {
//     fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         std::fmt::Result::Ok(()) // ????
//     }
// }

pub trait GrimIO {
    fn tell(&mut self, id: PlayerId, info: Info);
    fn prompt_player(&mut self, id: PlayerId) -> PlayerId;
    fn prompt_player_option(&mut self, id: PlayerId) -> Option<PlayerId>;
    fn prompt_tell(&mut self, id: PlayerId) -> Option<(PlayerId, Info)>;
    fn win(&mut self, team: bool) -> Never;
}

pub struct DebugIO;

impl GrimIO for DebugIO {
    fn tell(&mut self, id: PlayerId, info: Info) {
        println!("Player {id}: {info:?}.");
    }

    fn prompt_player(&mut self, id: PlayerId) -> PlayerId {
        println!("PLayer {id} is prompted.");

        let mut str = String::new();

        std::io::stdin().read_line(&mut str).unwrap();

        str.trim_end().parse().unwrap()
    }

    fn prompt_player_option(&mut self, id: PlayerId) -> Option<PlayerId> {
        println!("PLayer {id} is optionally prompted.");

        let mut str = String::new();
        std::io::stdin().read_line(&mut str).unwrap();
        if str.is_empty() {
            None
        } else {
            Some(str.parse().unwrap())
        }
    }

    fn prompt_tell(&mut self, id: PlayerId) -> Option<(PlayerId, Info)> {
        println!("PLayer {id} is prompted for a tell.");

        let mut str = String::new();

        std::io::stdin().read_line(&mut str).unwrap();

        if str.is_empty() {
            None
        } else {
            // num num role
            let mut chunks = str.split_whitespace();

            let target: PlayerId = chunks.next().unwrap().parse().unwrap();
            let player: PlayerId = chunks.next().unwrap().parse().unwrap();

            let role = {
                let chunk = chunks.next().unwrap();

                let roles: Vec<RoleId> = RoleId::all()
                    .into_iter()
                    .map(|x| (x, format!("{x:?}")))
                    .filter(|(_, x)| x.to_lowercase().strip_prefix(chunk).is_some())
                    .map(|x| x.0)
                    .collect();
                // this might filter out perfect matches

                assert_eq!(roles.len(), 1);

                roles[0]
            };

            todo!() // Some((target, player, role))
        }
    }

    fn win(&mut self, _team: bool) -> Never {
        todo!()
    }
}

#[derive(Debug)]
pub struct Grimoir<IO: GrimIO> {
    actions: Vec<Action>,
    players: Vec<Player>,
    rand: SmallRng,
    io: IO,
}

impl<IO: GrimIO> Grimoir<IO> {
    pub fn gen_roles(seed: Option<u64>) -> Vec<Role> {
        // at 15, 9 2 3 1
        let mut ids = Vec::new();

        let mut rand = SmallRng::seed_from_u64(seed.unwrap_or(0));

        let all = RoleId::all();

        let mut towns = all
            .iter()
            .filter(|r| r.is_townsfolk())
            .cloned()
            .collect::<Vec<RoleId>>();
        let mut outside = all
            .iter()
            .filter(|r| r.is_outsider())
            .cloned()
            .collect::<Vec<RoleId>>();
        let mut minions = all
            .iter()
            .filter(|r| r.is_minion())
            .cloned()
            .collect::<Vec<RoleId>>();

        for _ in 0..3 {
            ids.push(minions.remove(Rng::random_range(&mut rand, 0..minions.len())));
        }

        for _ in 0..(if ids.contains(&RoleId::Baron) { 4 } else { 2 }) {
            ids.push(outside.remove(Rng::random_range(&mut rand, 0..outside.len())));
        }

        for _ in 0..(if ids.contains(&RoleId::Baron) { 7 } else { 9 }) {
            ids.push(towns.remove(Rng::random_range(&mut rand, 0..towns.len())));
        }

        ids.push(RoleId::Imp);

        assert_eq!(ids.len(), 15);
        for _ in 0..Rng::random_range(&mut rand, 5..10) {
            ids.swap(
                Rng::random_range(&mut rand, 0..15),
                Rng::random_range(&mut rand, 0..15),
            );
        }

        ids.iter()
            .map(|x| match x {
                RoleId::WasherWoman => Role::WasherWoman,
                RoleId::Librarian => Role::Librarian,
                RoleId::Investigator => Role::Investigator,
                RoleId::Chef => Role::Chef,
                RoleId::Empath => Role::Empath,
                RoleId::FortuneTeller => Role::FortuneTeller {
                    red_herring: {
                        let goods = ids
                            .iter()
                            .filter(|x| x.is_good() && **x != RoleId::FortuneTeller)
                            .collect::<Vec<_>>();

                        let r = goods[Rng::random_range(&mut rand, 0..goods.len())];

                        ids.iter().position(|x| x == r).unwrap()
                    },
                },
                RoleId::Undertaker => Role::Undertaker { last_exec: None },
                RoleId::Monk => Role::Monk,
                RoleId::RavensKeeper => Role::RavensKeeper,
                RoleId::Virgin => Role::Virgin {
                    ability_used: false,
                },
                RoleId::Slayer => Role::Slayer {
                    ability_used: false,
                },
                RoleId::Soldier => Role::Soldier,
                RoleId::Mayor => Role::Mayor,
                RoleId::Butler => Role::Butler { butlered: None },
                RoleId::Drunk => Role::Drunk {
                    role: { towns.remove(Rng::random_range(&mut rand, 0..towns.len())) },
                },
                RoleId::Recluse => Role::Recluse,
                RoleId::Saint => Role::Saint,
                RoleId::Poisoner => Role::Poisoner,
                RoleId::Spy => Role::Spy,
                RoleId::ScarletWoman => Role::ScarletWoman,
                RoleId::Baron => Role::Baron,
                // RoleId::Imp => Role::Imp {
                //     bluffs: {
                //         let mut i = towns.iter();
                //         // todo make Rngom
                //         [i.next().cloned(), i.next().cloned(), i.next().cloned()]
                //     },
                // },
                RoleId::Imp => Role::Imp,
            })
            .collect()
    }

    fn tell(&mut self, player: PlayerId, info: Info) {
        // (*self.io.tell)(player, info);
        self.io.tell(player, info)
    }

    fn tell_all(&mut self, info: Info) {
        for id in 0..self.players.len() {
            self.tell(id, info.clone());
        }
    }

    fn prompt_player_optional(&mut self, player: PlayerId) -> Option<PlayerId> {
        self.io.prompt_player_option(player)
    }

    fn prompt_player(&mut self, player: PlayerId) -> PlayerId {
        self.io.prompt_player(player)
    }

    fn prompt_tell(&mut self, player: PlayerId) -> Option<(PlayerId, Info)> {
        self.io.prompt_tell(player)
    }

    fn win(&mut self, team: bool) {
        self.io.win(team).into()
    }

    pub fn new(roles: Vec<Role>, seed: Option<u64>, io: IO) -> Grimoir<IO> {
        let players = roles
            .iter()
            .map(|r| Player {
                role: r.clone(),
                alive: true,
                notes: Vec::new(),
                ghost_vote: true,
            })
            .collect();

        Grimoir {
            players,
            actions: Vec::new(),
            io,
            rand: SmallRng::seed_from_u64(seed.unwrap_or(0)),
        }
    }

    pub fn first_night(&mut self) {
        self.tell_all(Info::Night);

        let roles: Vec<Role> = self.players.iter().map(|x| x.role.clone()).collect();

        roles.iter().enumerate().for_each(|(i, x)| {
            if let Role::Drunk { role } = x {
                self.tell(i, Info::Role(role.clone()));
            } else {
                self.tell(i, Info::Role(x.id()));
            }
        });

        // minion and demon info
        let minions: Vec<_> = self
            .players
            .iter()
            .enumerate()
            .filter(|(_, x)| x.role.id().is_minion())
            .map(|(i, _)| i)
            .collect();

        let demon = self.get_role(RoleId::Imp).unwrap();

        // minion info
        minions.iter().for_each(|x| {
            self.tell(*x, Info::Player(demon));
            self.tell(demon, Info::Player(*x))
        });

        // self.actions.push(Action::MinionInfo(demon));

        // demon info
        let all = RoleId::all();

        let mut free_roles: Vec<_> = all
            .into_iter()
            .filter(|x| x.is_good() && self.get_role(*x).is_none())
            .collect();

        let mut bluffs = [None; 3];
        for i in 0..3 {
            let bluff = Rng::random_range(&mut self.rand, 0..free_roles.len());

            if bluff == 0 {
                break;
            } else {
                let bluff = free_roles.remove(bluff);

                bluffs[i] = Some(bluff);

                self.tell(demon, Info::Role(bluff));
            }
        }

        // self.tell(demon, Info::Players(minions.clone()));
        // self.tell(
        //     demon,
        //     Info::Roles(
        //         bluffs
        //             .iter()
        //             .filter(|x| x.is_some())
        //             .map(|x| x.unwrap())
        //             .collect(),
        //     ),
        // );

        // dbg!();

        self.actions.push(Action::ImpInfo(bluffs));

        self.exec(RoleId::Poisoner);
        self.exec(RoleId::WasherWoman);
        self.exec(RoleId::Librarian);
        self.exec(RoleId::Investigator);
        self.exec(RoleId::Chef);
        self.exec(RoleId::Empath);
        self.exec(RoleId::FortuneTeller);
        self.exec(RoleId::Butler);
        self.exec(RoleId::Spy);
    }

    pub fn night(&mut self) {
        self.tell_all(Info::Night);

        for player in &mut self.players {
            player.notes = player
                .notes
                .iter()
                .filter(|x| ![Note::MonkProtected, Note::Poisoned].contains(x))
                .cloned()
                .collect();
        }

        self.exec(RoleId::Poisoner);
        self.exec(RoleId::Monk);
        self.exec(RoleId::Imp);
        self.exec(RoleId::RavensKeeper);
        self.exec(RoleId::Empath);
        self.exec(RoleId::FortuneTeller);
        self.exec(RoleId::Butler);
        self.exec(RoleId::Undertaker);
        self.exec(RoleId::Spy);
    }

    pub fn day(&mut self) {
        self.tell_all(Info::Day);

        let mut i = 0;
        while i < self.players.len() {
            if self.players[i].notes.contains(&Note::DiedTonight) {
                self.tell_all(Info::Player(i)); // they died
            }
            i += 1;
        }

        // clears exec and died
        for player in &mut self.players {
            player.notes = player
                .notes
                .iter()
                .filter(|x| ![Note::ExecToday, Note::DiedTonight].contains(x))
                .cloned()
                .collect();
        }

        self.chat();

        let mut slays = false;

        // slays
        for id in 0..self.players.len() {
            let res = self.prompt_player_optional(id);

            if let Some(target) = res {
                slays = true;

                self.actions.push(Action::Slay(id, target));
                self.tell_all(Info::Slays(id, target));

                if let Role::Slayer {
                    ability_used: false,
                } = self.players[id].role
                {
                    // recluse might also die lol
                    if self.players[target].role.id().is_demon()
                        && !self.players[id].notes.contains(&Note::Poisoned)
                    {
                        // self.actions.push(Action::Slayer(target));
                        self.players[target].alive = false;
                        self.tell_all(Info::Player(target));

                        self.demon_kill();
                    }
                }

                if let Role::Slayer { ability_used } = &mut self.players[id].role {
                    *ability_used = true;
                }
            }
        }

        if slays {
            self.chat();
        }

        // noms

        // (nom, target, votes)
        let mut last_nom: Option<usize> = None;
        let mut i = 0;

        let mut voting_his = Vec::new();

        if loop {
            if self.players[i].alive && voting_his.iter().all(|(x, _, _)| *x != i) {
                if let Some(nom) = self.prompt_player_optional(i)
                    && voting_his.iter().all(|(_, x, _)| *x != nom)
                {
                    self.actions.push(Action::Nomination(i, nom));

                    if let Role::Virgin {
                        ability_used: false,
                    } = self.players[nom].role
                    {
                        if self.players[i].role.id().is_townsfolk()
                            && !self.players[nom].notes.contains(&Note::Poisoned)
                        {
                            //self.actions.push(Action::Virgin(i));

                            self.execute(i);

                            let Role::Virgin { ability_used } = &mut self.players[nom].role else {
                                panic!()
                            };

                            *ability_used = true;

                            break false;
                        }

                        let Role::Virgin { ability_used } = &mut self.players[nom].role else {
                            panic!()
                        };

                        *ability_used = true;
                    }

                    // is this needed?
                    self.chat();

                    let mut votes = [false; 15];
                    for j in 0..self.players.len() {
                        if self.players[j].alive {
                            if self.prompt_player_optional(j).is_some() {
                                votes[j] = true;
                            }
                        } else if self.players[j].ghost_vote {
                            if self.prompt_player_optional(j).is_some() {
                                votes[j] = true;
                                self.players[j].ghost_vote = false;
                            }
                        }
                    }

                    last_nom = Some(i);

                    voting_his.push((
                        i,
                        nom,
                        votes.iter().fold(0, |x, b| if *b { x + 1 } else { x }),
                    ));

                    // TODO: display voting results

                    self.actions.push(Action::Vote(votes));
                }
            }

            i = (i + 1) % self.players.len();

            if i == last_nom.unwrap_or(0) {
                break true;
            }
        } {
            // if we didnt aready execute someone

            // exec code
            let thresh = (self.players.iter().filter(|x| x.alive).count() + 1) / 2;

            if let Some((_, nomed, votes)) = voting_his.iter().max_by_key(|x| x.2).copied() {
                if votes >= thresh && voting_his.iter().filter(|x| x.2 == votes).count() == 1 {
                    self.execute(nomed);
                }
            }
        }

        //eod abilties
        let count = self.players.iter().filter(|x| x.alive).count();
        if count == 3 {
            if let Some(mayor) = self.get_role(RoleId::Mayor) {
                if !self.players[mayor].notes.contains(&Note::ExecToday) {
                    self.win(true);
                }
            }
        } else if count == 2 {
            self.win(false);
        }
    }

    fn chat(&mut self) {
        let mut needs_update = [true; 15];
        while needs_update.iter().any(|x| *x) {
            for i in 0..15 {
                if needs_update[i] {
                    if let Some((player, info)) = self.prompt_tell(i) {
                        // if let Some((player, subject, role)) = self.prompt_tell(i) {
                        self.tell(player, Info::Player(i));
                        //     self.tell(player, Info::Player(subject));
                        //     self.tell(player, Info::Role(role));

                        self.tell(player, info);

                        needs_update[player] = true;
                    } else {
                        needs_update[i] = false;
                    }
                }
            }
        }
    }

    fn execute(&mut self, player: usize) {
        // self.actions.push(Action::Executed(player));
        self.tell_all(Info::Player(player));
        self.players[player].alive = false;

        if let Some(mayor) = self.get_role(RoleId::Mayor) {
            self.players[mayor].notes.push(Note::ExecToday);
        }

        if let Some(undertaker) = self.get_role(RoleId::Undertaker) {
            let role = Some(self.players[player].role.id());

            let Role::Undertaker { last_exec } = &mut self.players[undertaker].role else {
                panic!()
            };

            *last_exec = role;
        }

        // if there is a drunk who thinks they are the undertaker
        if let Some(drunk) = self.get_role(RoleId::Drunk) {
            if let Role::Drunk {
                role: RoleId::Undertaker,
            } = self.players[drunk].role
            {
                self.players[drunk].notes.push(Note::ExecToday);
            }
        }

        if self.players[player].role.id().is_demon() {
            self.demon_kill();
        }
    }

    fn demon_kill(&mut self) {
        assert!(
            self.players
                .iter()
                .all(|x| !(x.role.id().is_demon() && x.alive))
        );

        if let Some(scarlet) = self.get_role(RoleId::ScarletWoman)
            && !self.players[scarlet].notes.contains(&Note::Poisoned)
        {
            // self.actions.push(Action::ScarletWoman);
            self.players[scarlet].role = Role::Imp; // { bluffs: [None; 3] };

            self.tell(scarlet, Info::Role(RoleId::Imp));
        } else {
            self.win(true);
        }
    }

    fn exec(&mut self, role: RoleId) {
        let Some(mut id) = self.get_role(role) else {
            return;
        };

        // scarlet woman is the only role who can create a double i.e. two demons one of which
        // might be dead
        if role == RoleId::Imp && !self.players[id].alive {
            let Some(scarlet) = self.get_rand_player(&mut |(_, x): &(PlayerId, &Player)| {
                x.role.id().is_demon() && x.alive
            }) else {
                // this means no demon is alive
                panic!();
            };

            id = scarlet;
        }

        if role == RoleId::Drunk || self.players[id].notes.contains(&Note::Poisoned) {
            self.poisoned_exec(role);
            return;
        }

        // only the ravenskeeper activates after they are dead
        if !self.players[id].alive && role != RoleId::RavensKeeper {
            return;
        }

        match role {
            RoleId::WasherWoman | RoleId::Librarian | RoleId::Investigator => {
                let play1 = self
                    .get_rand_player(&mut |(_, x)| match role {
                        RoleId::WasherWoman => x.role.id().is_townsfolk(),
                        RoleId::Librarian => x.role.id().is_outsider(),
                        RoleId::Investigator => {
                            x.role.id().is_minion() && x.role.id() != RoleId::Spy
                        }
                        _ => unreachable!(),
                    })
                    .unwrap();

                let play2 = self.get_rand_player(&mut |(i, _)| *i != play1).unwrap();

                let mut players = [play1, play2];

                players.sort();

                self.actions.push(match role {
                    RoleId::WasherWoman => Action::WasherWoman {
                        players,
                        role: self.players[play1].role.id(),
                    },
                    RoleId::Librarian => Action::Librarian {
                        players,
                        role: self.players[play1].role.id(),
                    },
                    RoleId::Investigator => Action::Investigator {
                        players,
                        role: self.players[play1].role.id(),
                    },
                    _ => unreachable!(),
                });

                self.tell(id, Info::Role(self.players[play1].role.id()));
                self.tell(id, Info::Player(players[0]));
                self.tell(id, Info::Player(players[1]));
            }
            RoleId::Chef => {
                let last = self.players.last().unwrap();

                let pairs = self
                    .players
                    .iter()
                    .fold(
                        (
                            0,
                            last.role.id() == RoleId::Recluse
                                || (last.role.id().is_evil() && last.role.id() != RoleId::Spy),
                        ),
                        |(pairs, last), player| {
                            let evil = player.role.id() == RoleId::Recluse
                                || (player.role.id().is_evil() && player.role.id() != RoleId::Spy);

                            if last && evil {
                                (pairs + 1, evil)
                            } else {
                                (pairs, evil)
                            }
                        },
                    )
                    .0;

                self.actions.push(Action::Chef { pairs });
                self.tell(id, Info::Number(pairs))
            }
            RoleId::Empath => {
                let mut count = 0;

                // this counts from the empath outwards
                for i in 1..self.players.len() {
                    let x = &self.players[(id + i) % self.players.len()];

                    if x.alive {
                        if x.role.id() == RoleId::Recluse
                            || (x.role.id().is_evil() && x.role.id() != RoleId::Spy)
                        {
                            count += 1;
                        }
                        break;
                    }
                }

                for i in (1..self.players.len()).rev() {
                    let x = &self.players[(id + i) % self.players.len()];

                    if x.alive {
                        if x.role.id() == RoleId::Recluse
                            || (x.role.id().is_evil() && x.role.id() != RoleId::Spy)
                        {
                            count += 1;
                        }
                        break;
                    }
                }

                self.actions.push(Action::Empath(count));
                self.tell(id, Info::Number(count));
            }
            RoleId::FortuneTeller => {
                let Role::FortuneTeller { red_herring } = self.players[id].role else {
                    panic!()
                };

                let play1 = self.prompt_player(id);
                let play2 = self.prompt_player(id);

                let response = play1 == red_herring
                    || play2 == red_herring
                    || self.players[play1].role.id().is_demon()
                    || self.players[play2].role.id().is_demon()
                    || self.players[play1].role.id() == RoleId::Recluse
                    || self.players[play2].role.id() == RoleId::Recluse;

                self.actions.push(Action::FortuneTeller {
                    target: [play1, play2],
                    response,
                });

                self.tell(id, Info::Bool(response));
            }
            RoleId::Undertaker => {
                let Role::Undertaker { last_exec } = self.players[id].role else {
                    panic!()
                };

                if let Some(role) = last_exec {
                    let seen = if role == RoleId::Spy {
                        let vec: Vec<_> =
                            RoleId::all().into_iter().filter(|x| x.is_good()).collect();
                        vec[Rng::random_range(&mut self.rand, 0..vec.len())]
                    } else if role == RoleId::Recluse {
                        let vec: Vec<_> =
                            RoleId::all().into_iter().filter(|x| x.is_evil()).collect();
                        vec[Rng::random_range(&mut self.rand, 0..vec.len())]
                    } else {
                        role
                    };

                    self.actions.push(Action::Undertaker(seen));
                    self.tell(id, Info::Role(seen));
                }

                let Role::Undertaker { ref mut last_exec } = self.players[id].role else {
                    panic!()
                };

                *last_exec = None;
            }
            RoleId::Monk => {
                let player = self.prompt_player(id);

                self.actions.push(Action::Monk { protected: player });

                self.players[player].notes.push(Note::MonkProtected);
            }
            RoleId::RavensKeeper => {
                assert!(self.players[id].notes.contains(&Note::DiedTonight));

                let player = self.prompt_player(id);

                let seen = if self.players[player].role.id() == RoleId::Spy {
                    let vec: Vec<_> = RoleId::all().into_iter().filter(|x| x.is_good()).collect();
                    vec[Rng::random_range(&mut self.rand, 0..vec.len())]
                } else if self.players[player].role.id() == RoleId::Recluse {
                    let vec: Vec<_> = RoleId::all().into_iter().filter(|x| x.is_evil()).collect();
                    vec[Rng::random_range(&mut self.rand, 0..vec.len())]
                } else {
                    self.players[player].role.id()
                };

                self.actions.push(Action::RavensKeeper {
                    target: player,
                    response: seen,
                });
                self.tell(id, Info::Role(seen));
            }
            RoleId::Butler => {
                let player = self.prompt_player(id);

                let Role::Butler { ref mut butlered } = self.players[id].role else {
                    panic!()
                };

                self.actions.push(Action::Butler(player));

                *butlered = Some(player);
            }

            // RoleId::Drunk => todo!(), // TODO this should give them fake info this is also equal
            // to the poisoned version of their role
            RoleId::Poisoner => {
                let player = self.prompt_player(id);

                self.actions.push(Action::Poisoner { target: player });

                self.players[player].notes.push(Note::Poisoned);
            }
            RoleId::Spy => {
                self.tell(id, Info::Grim(self.players.clone()));
                self.actions.push(Action::Spy(self.players.clone()));
            }
            RoleId::Imp => {
                let player = self.prompt_player(id);

                self.actions.push(Action::ImpKill(player));

                if !self.players[player].notes.contains(&Note::MonkProtected)
                    // if soldier
                    && !(self.players[player].role.id() == RoleId::Soldier
                        && !self.players[player].notes.contains(&Note::Poisoned))
                    // if mayor
                    && !(self.players[player].role.id() == RoleId::Mayor
                        && !self.players[player].notes.contains(&Note::Poisoned))
                {
                    self.players[player].alive = false;

                    self.players[player].notes.push(Note::DiedTonight);

                    // self.actions.push(Action::Died(player));

                    // if they kill themselves in the night
                    if player == id {
                        if let Some(minion) =
                            self.get_rand_player(&mut |(_, x): &(PlayerId, &Player)| {
                                x.alive && x.role.id().is_minion()
                            })
                        {
                            // minions do not demon info
                            self.players[minion].role = Role::Imp; // { bluffs: [None; 3] };
                            self.actions.push(Action::ImpTransfer(minion));
                        }
                    } else {
                        self.win(true).into()
                    }
                } else if self.players[player].role.id() == RoleId::Mayor {
                    let Some(alt) = self.get_rand_player(&mut |(_, y): &(PlayerId, &Player)| {
                        y.alive && y.role.id() != RoleId::Mayor
                    }) else {
                        panic!()
                    };

                    self.actions.push(Action::MayorDeflection(alt));

                    if !self.players[alt].notes.contains(&Note::MonkProtected)
                        && !(self.players[alt].role.id() == RoleId::Soldier
                            && !self.players[alt].notes.contains(&Note::Poisoned))
                    {
                        // self.actions.push(Action::Died(alt));

                        self.players[alt].alive = false;

                        self.players[alt].notes.push(Note::DiedTonight);

                        // if they kill themselves in the night
                        if alt == id {
                            if let Some(minion) =
                                self.get_rand_player(&mut |(_, x): &(PlayerId, &Player)| {
                                    x.alive && x.role.id().is_minion()
                                })
                            {
                                self.actions.push(Action::ImpTransfer(minion));

                                // minions do not demon info
                                self.players[minion].role = Role::Imp; // { bluffs: [None; 3] };
                            }
                        } else {
                            todo!()
                        }
                    }
                }
            }
            _ => panic!(),
        }
    }

    fn poisoned_exec(&mut self, mut role: RoleId) {
        let Some(mut id) = self.get_role(role) else {
            panic!();
        };

        if let Role::Drunk { role: drunk_role } = self.players[id].role {
            role = drunk_role;
        }

        if role == RoleId::Imp && !self.players[id].alive {
            let Some(scarlet) = self.get_rand_player(&mut |(_, x): &(PlayerId, &Player)| {
                x.role.id().is_demon() && x.alive
            }) else {
                // this means no demon is alive
                panic!();
            };

            id = scarlet;
        }

        // only the ravenskeeper activates after they are dead
        if !self.players[id].alive && role != RoleId::RavensKeeper {
            return;
        }

        assert!(self.players[id].notes.contains(&Note::Poisoned));

        match role {
            RoleId::WasherWoman | RoleId::Librarian | RoleId::Investigator => {
                let play1 = self.get_rand_player(&mut |(_, _)| true).unwrap();
                let play2 = self.get_rand_player(&mut |(i, _)| *i != play1).unwrap();

                let mut players = [play1, play2];

                players.sort();

                // self.tell(id, Info::Role(self.players[play1].role.id()));
                let roles: Vec<_> = RoleId::all()
                    .into_iter()
                    .filter(|x| match role {
                        RoleId::WasherWoman => x.is_townsfolk(),
                        RoleId::Librarian => x.is_outsider(),
                        RoleId::Investigator => x.is_minion(),
                        _ => panic!(),
                    })
                    .collect();

                let role = roles[Rng::random_range(&mut self.rand, 0..roles.len())];

                self.actions.push(match role {
                    RoleId::WasherWoman => Action::WasherWoman {
                        players,
                        role: self.players[play1].role.id(),
                    },
                    RoleId::Librarian => Action::Librarian {
                        players,
                        role: self.players[play1].role.id(),
                    },
                    RoleId::Investigator => Action::Investigator {
                        players,
                        role: self.players[play1].role.id(),
                    },
                    _ => unreachable!(),
                });

                self.tell(id, Info::Role(role));
                self.tell(id, Info::Player(players[0]));
                self.tell(id, Info::Player(players[1]));
            }
            RoleId::Chef => {
                let pairs = self
                    .get_rand((self.players.iter().filter(|x| x.role.id().is_evil())).count() - 1)
                    as u32;

                self.actions.push(Action::Chef { pairs });

                self.tell(id, Info::Number(pairs));
            }
            RoleId::Empath => {
                let count = self.get_rand(3) as u32;

                self.actions.push(Action::Empath(count));
                self.tell(id, Info::Number(count));
            }
            RoleId::FortuneTeller => {
                let target = [self.prompt_player(id), self.prompt_player(id)];

                let num = self.get_rand(2) == 1;

                self.actions.push(Action::FortuneTeller {
                    target,
                    response: num,
                });

                self.tell(id, Info::Bool(num));
            }
            RoleId::Undertaker => {
                let Role::Undertaker { last_exec } = self.players[id].role else {
                    panic!()
                };

                if let Some(_) = last_exec {
                    let vec: Vec<_> = RoleId::all();
                    let role = vec[self.get_rand(vec.len())];
                    self.actions.push(Action::Undertaker(role));
                    self.tell(id, Info::Role(role));
                }

                let Role::Undertaker { ref mut last_exec } = self.players[id].role else {
                    panic!()
                };

                *last_exec = None;
            }
            RoleId::Monk => {
                let protected = self.prompt_player(id);
                self.actions.push(Action::Monk { protected });
            }
            RoleId::RavensKeeper => {
                assert!(self.players[id].notes.contains(&Note::DiedTonight));

                let target = self.prompt_player(id);

                let vec: Vec<_> = RoleId::all();
                let role = vec[self.get_rand(vec.len())];

                self.actions.push(Action::RavensKeeper {
                    target,
                    response: role,
                });
                self.tell(id, Info::Role(role));
            }
            RoleId::Butler => {
                let player = self.prompt_player(id);

                self.actions.push(Action::Butler(player));

                let Role::Butler { ref mut butlered } = self.players[id].role else {
                    panic!()
                };

                *butlered = Some(player);
            }

            RoleId::Poisoner => {
                let target = self.prompt_player(id);

                self.actions.push(Action::Poisoner { target });
            }
            RoleId::Spy => {
                self.tell(id, Info::Grim(todo!()));
                self.actions.push(Action::Spy(todo!()));
            }
            RoleId::Imp => {
                let player = self.prompt_player(id);
                self.actions.push(Action::ImpKill(player));
            }
            _ => panic!(),
        }
    }

    fn get_role(&self, role: RoleId) -> Option<PlayerId> {
        self.players.iter().position(|x| x.role.id() == role)
    }

    fn get_rand(&mut self, end: usize) -> usize {
        Rng::random_range(&mut self.rand, 0..end)
    }

    fn get_rand_player(
        &mut self,
        filter: &mut impl FnMut(&(PlayerId, &Player)) -> bool,
    ) -> Option<PlayerId> {
        let vec: Vec<_> = self
            .players
            .iter()
            .enumerate()
            .filter(filter)
            .map(|(i, _)| i)
            .collect();

        if vec.len() == 0 {
            None
        } else {
            vec.get(rand::Rng::random_range(&mut self.rand, 0..vec.len()))
                .map(|x| *x)
        }
    }
}
