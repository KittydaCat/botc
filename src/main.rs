use rand::{SeedableRng, rngs::SmallRng};

mod tui;

#[derive(Clone, Debug)]
pub struct Player {
    role: Role,
    alive: bool,
    notes: Vec<Note>,
    ghost_vote: bool,
}

type PlayerId = usize;

#[enum_ids::enum_ids(derive = "Clone, Copy, PartialEq, Debug")]
#[derive(Clone, Debug)]
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
    Imp { bluffs: [Option<RoleId>; 3] },
}

impl RoleId {
    fn all() -> Vec<RoleId> {
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

// each should be accomanyed by a player id or the assumption of singletons
//
// we will be assuming singletons
//
// selections are true info is what is seen by the player if drunk
#[derive(Clone, Debug)]
enum Action {
    MinionInfo(PlayerId),
    ImpInfo {
        bluffs: [Option<RoleId>; 3],
        minions: Vec<PlayerId>,
    },

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
    ScarletWoman,
    Imp(PlayerId),
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
    Spy,

    // day abilities
    Virgin(PlayerId),
    Slayer(PlayerId),

    // victories
    TownsfolkWin,
    MayorWin,
    ImpWin,

    // other things
    Slay(usize, usize),
    Nomination(PlayerId, PlayerId),
    Vote {
        yay: Vec<PlayerId>,
        nay: Vec<PlayerId>,
    },
    Died(PlayerId),
    Executed(PlayerId),

    Init(Vec<Player>),
}

// each is attached to a player
#[derive(Clone, Debug, PartialEq)]
enum Note {
    Poisoned,
    MonkProtected,
    DiedTonight,
    ExecToday,
}

// how to turn into thing
#[derive(Clone, Debug)]
pub enum Info {
    // IsRole(PlayerId, RoleId),
    Number(u32),
    Bool(bool),
    Players(Vec<PlayerId>),
    Player(PlayerId),
    Role(RoleId),
    Roles(Vec<RoleId>),
    Grim(Vec<Player>),
    Slays(PlayerId, PlayerId),
}

struct GrimIO {
    tell: Box<dyn FnMut(PlayerId, Info)>,
    prompt: Box<dyn FnMut(PlayerId) -> PlayerId>,
    win: Box<dyn FnMut(bool)>,
}

impl std::fmt::Debug for GrimIO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Result::Ok(()) // ????
    }
}

#[derive(Debug)]
pub struct Grimoir {
    actions: Vec<Action>,
    players: Vec<Player>,
    rand: SmallRng,
    io: GrimIO,
}

impl Grimoir {
    fn gen_roles(seed: Option<u64>) -> Vec<Role> {
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
            ids.push(minions.remove(rand::random_range(0..minions.len())));
        }

        for _ in 0..(if ids.contains(&RoleId::Baron) { 4 } else { 2 }) {
            ids.push(outside.remove(rand::random_range(0..outside.len())));
        }

        for _ in 0..(if ids.contains(&RoleId::Baron) { 7 } else { 9 }) {
            ids.push(towns.remove(rand::random_range(0..towns.len())));
        }

        ids.push(RoleId::Imp);

        assert_eq!(ids.len(), 15);
        for _ in 0..rand::random_range(5..10) {
            ids.swap(rand::random_range(0..15), rand::random_range(0..15));
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

                        let r = goods[rand::random_range(0..goods.len())];

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
                    role: { towns.remove(rand::random_range(0..towns.len())) },
                },
                RoleId::Recluse => Role::Recluse,
                RoleId::Saint => Role::Saint,
                RoleId::Poisoner => Role::Poisoner,
                RoleId::Spy => Role::Spy,
                RoleId::ScarletWoman => Role::ScarletWoman,
                RoleId::Baron => Role::Baron,
                RoleId::Imp => Role::Imp {
                    bluffs: {
                        let mut i = towns.iter();
                        // todo make random
                        [i.next().cloned(), i.next().cloned(), i.next().cloned()]
                    },
                },
            })
            .collect()
    }

    fn tell(&mut self, player: PlayerId, info: Info) {
        (*self.io.tell)(player, info);
    }

    fn tell_all(&mut self, info: Info) {
        for id in 0..self.players.len() {
            self.tell(id, info.clone());
        }
    }

    fn prompt(&mut self, player: PlayerId) -> PlayerId {
        (*self.io.prompt)(player)
    }

    fn win(&mut self, team: bool) {
        (*self.io.win)(team);
    }

    pub fn first_night(roles: Vec<Role>, seed: Option<u64>, io: GrimIO) -> Grimoir {
        let players = roles
            .iter()
            .map(|r| Player {
                role: r.clone(),
                alive: true,
                notes: Vec::new(),
                ghost_vote: true,
            })
            .collect();

        let mut grim = Grimoir {
            players,
            io,
            actions: Vec::new(),
            rand: SmallRng::seed_from_u64(seed.unwrap_or(0)),
        };

        roles.iter().enumerate().for_each(|(i, x)| {
            if let Role::Drunk { role: role } = x {
                grim.tell(i, Info::Role(role.clone()));
            } else {
                grim.tell(i, Info::Role(x.id()));
            }
        });

        // minion and demon info
        let minions: Vec<_> = grim
            .players
            .iter()
            .enumerate()
            .filter(|(_, x)| x.role.id().is_minion())
            .map(|(i, _)| i)
            .collect();

        let demon = grim.get_role(RoleId::Imp).unwrap();

        // minion info
        minions
            .iter()
            .for_each(|x| grim.tell(*x, Info::Player(demon)));

        grim.actions.push(Action::MinionInfo(demon));

        // demon info
        let all = RoleId::all();

        let mut free_roles: Vec<_> = all
            .into_iter()
            .filter(|x| x.is_good() && grim.get_role(*x).is_none())
            .collect();

        let mut bluffs = [None; 3];
        for i in 0..3 {
            let bluff = rand::Rng::random_range(&mut grim.rand, 0..free_roles.len());

            if bluff == 0 {
                break;
            } else {
                bluffs[i] = Some(free_roles.remove(bluff));
            }
        }

        grim.tell(demon, Info::Players(minions.clone()));
        grim.tell(
            demon,
            Info::Roles(
                bluffs
                    .iter()
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap())
                    .collect(),
            ),
        );

        dbg!();

        grim.actions.push(Action::ImpInfo { bluffs, minions });

        grim.exec(RoleId::Poisoner);
        grim.exec(RoleId::WasherWoman);
        grim.exec(RoleId::Librarian);
        grim.exec(RoleId::Investigator);
        grim.exec(RoleId::Chef);
        grim.exec(RoleId::Empath);
        grim.exec(RoleId::FortuneTeller);
        grim.exec(RoleId::Butler);
        grim.exec(RoleId::Spy);

        grim
    }

    pub fn night(&mut self) {
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
        // self.exec(RoleId::ScarletWoman);
        self.exec(RoleId::Imp);
        self.exec(RoleId::RavensKeeper);
        self.exec(RoleId::Empath);
        self.exec(RoleId::FortuneTeller);
        self.exec(RoleId::Butler);
        self.exec(RoleId::Undertaker);
        self.exec(RoleId::Spy);
    }

    pub fn day(&mut self) {
        for player in &mut self.players {
            player.notes = player
                .notes
                .iter()
                .filter(|x| ![Note::ExecToday, Note::DiedTonight].contains(x))
                .cloned()
                .collect();
        }

        // slays
        for id in 0..self.players.len() {
            let target = self.prompt(id);

            // this is the way to slay no one
            if id != target {
                self.actions.push(Action::Slay(id, target));
                self.tell_all(Info::Slays(id, target));

                if let Role::Slayer {
                    ability_used: false,
                } = self.players[id].role
                {
                    if self.players[target].role.id().is_demon()
                        && !self.players[id].notes.contains(&Note::Poisoned)
                    {
                        self.actions.push(Action::Slayer(target));
                        self.players[target].alive = false;
                        if let Some(scarlet) = self.get_role(RoleId::ScarletWoman) {
                            self.actions.push(Action::ScarletWoman);
                            self.players[scarlet].role = Role::Imp { bluffs: [None; 3] };
                        } else {
                            self.win(true);
                        }
                    }
                }

                if let Role::Slayer { ability_used } = &mut self.players[id].role {
                    *ability_used = true;
                }
            }
        }

        // noms

        // (nom, target, votes)
        let mut last_nom: Option<usize> = None;
        let mut i = 0;

        let mut voting_his = Vec::new();

        if loop {
            if self.players[i].alive && voting_his.iter().all(|(x, _, _)| *x != i) {
                let nom = self.prompt(i);

                // nop
                if nom != i && voting_his.iter().all(|(_, x, _)| *x != nom) {
                    self.actions.push(Action::Nomination(i, nom));

                    // exec virgin ???
                    if let Role::Virgin {
                        ability_used: false,
                    } = self.players[nom].role
                    {
                        if self.players[i].role.id().is_townsfolk()
                            && !self.players[nom].notes.contains(&Note::Poisoned)
                        {
                            self.actions.push(Action::Virgin(i));

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

                    let mut votes = 0;
                    for j in 0..self.players.len() {
                        if self.players[j].alive {
                            if self.prompt(j) != j {
                                votes += 1;
                            }
                        } else if self.players[j].ghost_vote {
                            if self.prompt(j) != j {
                                votes += 1;
                                self.players[j].ghost_vote = false;
                            }
                        }
                    }

                    last_nom = Some(i);

                    voting_his.push((i, nom, votes));
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

    fn execute(&mut self, player: usize) {
        self.actions.push(Action::Executed(player));
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

        if let Some(drunk) = self.get_role(RoleId::Drunk) {
            if let Role::Drunk {
                role: RoleId::Undertaker,
            } = self.players[drunk].role
            {
                self.players[drunk].notes.push(Note::ExecToday);
            }
        }

        if self.players[player].role.id().is_demon() {
            if let Some(scarlet) = self.get_role(RoleId::ScarletWoman) {
                self.actions.push(Action::ScarletWoman);
                self.players[scarlet].role = Role::Imp { bluffs: [None; 3] };
            } else {
                self.win(true);
            }
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
        }

        // only the ravenskeeper activates after they are dead
        if !self.players[id].alive && role != RoleId::RavensKeeper {
            return;
        }

        match role {
            // add recuse
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

                let mut players = vec![play1, play2];

                players.sort();

                self.tell(id, Info::Role(self.players[play1].role.id()));
                self.tell(id, Info::Players(players));
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

                let play1 = self.prompt(id);
                let play2 = self.prompt(id);

                self.tell(
                    id,
                    Info::Bool(
                        play1 == red_herring
                            || play2 == red_herring
                            || self.players[play1].role.id().is_demon()
                            || self.players[play2].role.id().is_demon()
                            || self.players[play1].role.id() == RoleId::Recluse
                            || self.players[play2].role.id() == RoleId::Recluse,
                    ),
                );
            }
            RoleId::Undertaker => {
                let Role::Undertaker { last_exec } = self.players[id].role else {
                    panic!()
                };

                if let Some(role) = last_exec {
                    if role == RoleId::Spy {
                        let vec: Vec<_> =
                            RoleId::all().into_iter().filter(|x| x.is_good()).collect();
                        let seen = vec[rand::Rng::random_range(&mut self.rand, 0..vec.len())];
                        self.tell(id, Info::Role(seen));
                    } else if role == RoleId::Recluse {
                        let vec: Vec<_> =
                            RoleId::all().into_iter().filter(|x| x.is_evil()).collect();
                        let seen = vec[rand::Rng::random_range(&mut self.rand, 0..vec.len())];
                        self.tell(id, Info::Role(seen));
                    } else {
                        self.tell(id, Info::Role(role));
                    }
                }

                let Role::Undertaker { ref mut last_exec } = self.players[id].role else {
                    panic!()
                };

                *last_exec = None;
            }
            RoleId::Monk => {
                let player = self.prompt(id);

                self.players[player].notes.push(Note::MonkProtected);
            }
            RoleId::RavensKeeper => {
                assert!(self.players[id].notes.contains(&Note::DiedTonight));

                let player = self.prompt(id);

                if self.players[player].role.id() == RoleId::Spy {
                    let vec: Vec<_> = RoleId::all().into_iter().filter(|x| x.is_good()).collect();
                    let role = vec[rand::Rng::random_range(&mut self.rand, 0..vec.len())];
                    self.tell(id, Info::Role(role));
                } else if self.players[player].role.id() == RoleId::Recluse {
                    let vec: Vec<_> = RoleId::all().into_iter().filter(|x| x.is_evil()).collect();
                    let role = vec[rand::Rng::random_range(&mut self.rand, 0..vec.len())];
                    self.tell(id, Info::Role(role));
                } else {
                    self.tell(id, Info::Role(self.players[player].role.id()));
                }
            }
            RoleId::Butler => {
                let player = self.prompt(id);

                let Role::Butler { ref mut butlered } = self.players[id].role else {
                    panic!()
                };

                *butlered = Some(player);
            }

            // RoleId::Drunk => todo!(), // TODO this should give them fake info this is also equal
            // to the poisoned version of their role
            RoleId::Poisoner => {
                let player = self.prompt(id);

                self.players[player].notes.push(Note::Poisoned);
            }
            RoleId::Spy => {
                self.tell(id, Info::Grim(self.players.clone()));
                self.actions.push(Action::Spy);
            }
            RoleId::Imp => {
                let player = self.prompt(id);

                if !self.players[player].notes.contains(&Note::MonkProtected)
                    && !(self.players[player].role.id() == RoleId::Soldier
                        && !self.players[player].notes.contains(&Note::Poisoned))
                    && !(self.players[player].role.id() == RoleId::Mayor
                        && !self.players[player].notes.contains(&Note::Poisoned))
                {
                    self.players[player].alive = false;

                    self.players[player].notes.push(Note::DiedTonight);

                    // if they kill themselves in the night
                    if player == id {
                        if let Some(minion) =
                            self.get_rand_player(&mut |(_, x): &(PlayerId, &Player)| {
                                x.alive && x.role.id().is_minion()
                            })
                        {
                            // minions do not demon info
                            self.players[minion].role = Role::Imp { bluffs: [None; 3] };
                        }
                    } else {
                        todo!()
                    }
                } else if self.players[player].role.id() == RoleId::Mayor {
                    let Some(alt) = self.get_rand_player(&mut |(_, y): &(PlayerId, &Player)| {
                        y.alive && y.role.id() != RoleId::Mayor
                    }) else {
                        panic!()
                    };

                    if !self.players[alt].notes.contains(&Note::MonkProtected)
                        && !(self.players[alt].role.id() == RoleId::Soldier
                            && !self.players[alt].notes.contains(&Note::Poisoned))
                    {
                        self.players[alt].alive = false;

                        self.players[alt].notes.push(Note::DiedTonight);

                        // if they kill themselves in the night
                        if alt == id {
                            if let Some(minion) =
                                self.get_rand_player(&mut |(_, x): &(PlayerId, &Player)| {
                                    x.alive && x.role.id().is_minion()
                                })
                            {
                                // minions do not demon info
                                self.players[minion].role = Role::Imp { bluffs: [None; 3] };
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

                let mut players = vec![play1, play2];

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

                let role = roles[rand::Rng::random_range(&mut self.rand, 0..roles.len())];

                self.tell(id, Info::Role(role));
                self.tell(id, Info::Players(players));
            }
            RoleId::Chef => {
                let num = self
                    .get_rand((self.players.iter().filter(|x| x.role.id().is_evil())).count() - 1)
                    as u32;

                self.tell(id, Info::Number(num));
            }
            RoleId::Empath => {
                let count = self.get_rand(3) as u32;

                self.actions.push(Action::Empath(count));
                self.tell(id, Info::Number(count));
            }
            RoleId::FortuneTeller => {
                let _ = self.prompt(id);
                let _ = self.prompt(id);

                let num = self.get_rand(2) == 1;

                self.tell(id, Info::Bool(num));
            }
            RoleId::Undertaker => {
                let Role::Undertaker { last_exec } = self.players[id].role else {
                    panic!()
                };

                if let Some(_) = last_exec {
                    let vec: Vec<_> = RoleId::all();
                    let role = vec[self.get_rand(vec.len())];
                    self.tell(id, Info::Role(role));
                }

                let Role::Undertaker { ref mut last_exec } = self.players[id].role else {
                    panic!()
                };

                *last_exec = None;
            }
            RoleId::Monk => {
                let _ = self.prompt(id);
            }
            RoleId::RavensKeeper => {
                assert!(self.players[id].notes.contains(&Note::DiedTonight));

                let _ = self.prompt(id);

                let vec: Vec<_> = RoleId::all();
                let role = vec[self.get_rand(vec.len())];
                self.tell(id, Info::Role(role));
            }
            RoleId::Butler => {
                let player = self.prompt(id);

                let Role::Butler { ref mut butlered } = self.players[id].role else {
                    panic!()
                };

                *butlered = Some(player);
            }

            // RoleId::Drunk => todo!(), // TODO this should give them fake info this is also equal
            // to the poisoned version of their role
            RoleId::Poisoner => {
                let _ = self.prompt(id);
            }
            RoleId::Spy => {
                todo!();
                self.tell(id, Info::Grim(self.players.clone()));
                self.actions.push(Action::Spy);
            }
            RoleId::Imp => {
                let _ = self.prompt(id);
            }
            _ => panic!(),
        }
    }

    fn get_role(&self, role: RoleId) -> Option<PlayerId> {
        self.players.iter().position(|x| x.role.id() == role)
    }

    fn get_rand(&mut self, end: usize) -> usize {
        rand::Rng::random_range(&mut self.rand, 0..end)
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

fn dbg_tell(player: usize, info: Info) {
    println!("Player {player}: {info:?}.");
}

fn dbg_prompt(player: usize) -> usize {
    println!("PLayer {player} is prompted.");

    let mut str = String::new();

    std::io::stdin().read_line(&mut str).unwrap();

    str.parse().unwrap()
}

fn dbg_win(_: bool) {}

fn main() {
    let players = dbg!(Grimoir::gen_roles(None));
    let io = GrimIO {
        tell: Box::new(dbg_tell),
        prompt: Box::new(dbg_prompt),
        win: Box::new(dbg_win),
    };

    let grim = Grimoir::first_night(players, None, io);
}
