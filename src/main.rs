#[derive(Clone, Debug)]
struct Player {
    role: Role,
    alive: bool,
    notes: Vec<Note>,
}

type PlayerId = usize;

#[enum_ids::enum_ids(derive = "Clone, Copy, PartialEq, Debug")]
#[derive(Clone, Debug)]
enum Role {
    // townsfolk
    WasherWoman {
        // players: [PlayerId; 2],
        // role: RoleId,
    },
    Librarian {
        // players: [PlayerId; 2],
        // role: RoleId,
    },
    Investigator {
        // players: [PlayerId; 2],
        // role: RoleId,
    },
    Chef {
        // pairs: u32,
    },
    Empath,
    FortuneTeller {
        red_herring: PlayerId,
    },
    Undertaker {
        last_exec: Option<RoleId>,
    },
    Monk {
        protected: Option<PlayerId>,
    },
    RavensKeeper,
    Virgin {
        ability_used: bool,
    },
    Slayer {
        ability_used: bool,
    },
    Soldier,
    Mayor,

    // outsiders
    Butler {
        butlered: Option<PlayerId>,
    },
    Drunk {
        role: RoleId,
    },
    Recluse,
    Saint,

    // minions
    Poisoner {
        target: Option<PlayerId>,
    },
    Spy,
    ScarletWoman,
    Baron,

    // demon
    Imp {
        bluffs: [Option<RoleId>; 3],
    },
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
        target: Option<PlayerId>,
    },
    Monk {
        protected: Option<PlayerId>,
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
    Nomination(PlayerId),
    Vote {
        yay: Vec<PlayerId>,
        nay: Vec<PlayerId>,
    },
    Died,
    Executed,

    Init(Vec<Player>),
}

// each is attached to a player
#[derive(Clone, Debug, PartialEq)]
enum Note {
    Drunk,
    Poisoned,
    // RedHerring
    MonkProtected,
    // RavensKeeperDied,
    DiedTonight, // ???
}

// how??? should this be raw info or what it conveys
//
// this is just the very basic
#[derive(Clone, Debug)]
enum Info {
    // IsRole(PlayerId, RoleId),
    Number(u32),
    Bool(bool),
    Players(Vec<PlayerId>),
    Player(PlayerId),
    Role(RoleId),
    Roles(Vec<RoleId>),
    Grim(Grimoir),
}

#[enum_ids::enum_ids]
enum Prompt {
    Player(PlayerId),
    TwoPlayer([PlayerId; 2]),
}

#[derive(Clone, Debug)]
struct Grimoir {
    actions: Vec<Action>,
    players: Vec<Player>,
    rand: rand::rngs::ThreadRng,
    // tell: FnMut(&mut Grimoir, Info),
}

impl Grimoir {
    fn tell(&mut self, player: PlayerId, info: Info) {
        todo!()
    }

    fn prompt(&mut self, player: PlayerId, prompt_id: PromptId) -> Prompt {
        todo!()
    }

    fn first_night(players: Vec<Player>) -> Grimoir {
        let mut grim = Grimoir {
            players,
            // tell,
            actions: Vec::new(),
            rand: rand::rng(),
        };

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

    fn night(&mut self) {
        todo!()
    }

    fn exec(&mut self, role: RoleId) {
        let Some(id) = self.get_role(role) else {
            return;
        };

        match role {
            RoleId::WasherWoman | RoleId::Librarian | RoleId::Investigator => {
                let play1 = self
                    .get_rand(&mut |(_, x)| match role {
                        RoleId::WasherWoman => x.role.id().is_townsfolk(),
                        RoleId::Librarian => x.role.id().is_outsider(),
                        RoleId::Investigator => x.role.id().is_minion(),
                        _ => unreachable!(),
                    })
                    .unwrap();

                let play2 = self.get_rand(&mut |(i, _)| *i != play1).unwrap();

                let mut players = vec![play1, play2];

                players.sort();

                self.tell(id, Info::Role(self.players[play1].role.id()));
                self.tell(id, Info::Players(players));
            }
            RoleId::Chef => {
                let pairs = self
                    .players
                    .iter()
                    .fold(
                        (0, self.players.last().unwrap().role.id().is_evil()),
                        |(pairs, last), player| {
                            let evil = player.role.id().is_evil();

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

                for i in 1..self.players.len() {
                    if self.players[(id + i) % self.players.len()].alive {
                        if self.players[(id + i) % self.players.len()]
                            .role
                            .id()
                            .is_evil()
                        {
                            count += 1;
                        }
                        break;
                    }

                    for i in (1..self.players.len()).rev() {
                        if self.players[(id + i) % self.players.len()].alive {
                            if self.players[(id + i) % self.players.len()]
                                .role
                                .id()
                                .is_evil()
                            {
                                count += 1;
                            }
                            break;
                        }
                    }
                }

                self.actions.push(Action::Empath(count));
                self.tell(id, Info::Number(count));
            }
            RoleId::FortuneTeller => {
                let Role::FortuneTeller { red_herring } = self.players[id].role else {
                    panic!()
                };

                let Prompt::TwoPlayer([play1, play2]) = self.prompt(id, PromptId::TwoPlayer) else {
                    panic!()
                };

                self.tell(
                    id,
                    Info::Bool(
                        play1 == red_herring
                            || play2 == red_herring
                            || self.players[play1].role.id().is_demon()
                            || self.players[play2].role.id().is_demon(),
                    ),
                );
            }
            RoleId::Undertaker => {
                let Role::Undertaker { last_exec } = self.players[id].role else {
                    panic!()
                };

                if let Some(role) = last_exec {
                    self.tell(id, Info::Role(role));
                    self.actions.push(Action::Undertaker(role))
                }

                let Role::Undertaker { ref mut last_exec } = self.players[id].role else {
                    panic!()
                };

                *last_exec = None;
            }
            RoleId::Monk => {
                let Role::Monk { mut protected } = self.players[id].role else {
                    panic!()
                };

                assert!(protected.is_none());

                let Prompt::Player(player) = self.prompt(id, PromptId::Player) else {
                    panic!()
                };

                let Role::Monk { ref mut protected } = self.players[id].role else {
                    panic!()
                };

                *protected = Some(player);

                self.players[player].notes.push(Note::MonkProtected);
            }
            RoleId::RavensKeeper => {
                assert!(self.players[id].notes.contains(&Note::DiedTonight));

                let Prompt::Player(player) = self.prompt(id, PromptId::Player) else {
                    panic!()
                };

                self.tell(id, Info::Role(self.players[player].role.id()));
            }
            // RoleId::Virgin => todo!(), // passive
            // RoleId::Slayer => todo!(),
            // RoleId::Soldier => todo!(),
            // RoleId::Mayor => todo!(),
            RoleId::Butler => {
                let Prompt::Player(player) = self.prompt(id, PromptId::Player) else {
                    panic!()
                };

                let Role::Butler { ref mut butlered } = self.players[id].role else {
                    panic!()
                };

                *butlered = Some(player);
            }
            // RoleId::Drunk => todo!(), // TODO this should give them fake info
            // RoleId::Recluse => todo!(),
            // RoleId::Saint => todo!(), // passive
            RoleId::Poisoner => {
                let Role::Poisoner { target } = self.players[id].role else {
                    panic!()
                };

                assert!(target.is_none());

                let Prompt::Player(player) = self.prompt(id, PromptId::Player) else {
                    panic!()
                };

                let Role::Poisoner { ref mut target } = self.players[id].role else {
                    panic!()
                };

                *target = Some(player);

                self.players[player].notes.push(Note::Poisoned);
            }
            RoleId::Spy => {
                self.tell(id, Info::Grim(self.clone()));
                self.actions.push(Action::Spy);
            }
            // RoleId::ScarletWoman => todo!(), // passive
            // RoleId::Baron => todo!(),
            RoleId::Imp => {
                let Prompt::Player(player) = self.prompt(id, PromptId::Player) else {
                    panic!()
                };

                self.players[player].alive = false;

                self.players[player].notes.push(Note::DiedTonight);

                // if they kill themselves in the night
                if player == id {
                    if let Some(minion) = self.get_rand(&mut |(_, x): &(PlayerId, &Player)| {
                        x.alive && x.role.id().is_minion()
                    }) {
                        self.players[minion].role = self.players[id].role.clone();
                    }
                }
            }
            _ => todo!(),
        }
    }
    fn get_role(&self, role: RoleId) -> Option<PlayerId> {
        self.players.iter().position(|x| x.role.id() == role)
    }

    fn get_rand(
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

fn main() {
    println!("Hello, world!");
}
