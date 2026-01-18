use std::{cell::RefCell, rc::Rc};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{List, ListItem, ListState, Paragraph, StatefulWidget, Tabs, Widget},
};

use crate::game::{DebugIO, GrimIO, Grimoir, Info, Never, PlayerId, RoleId};

#[enum_ids::enum_ids]
enum InputType {
    Player(PlayerId),
    OptionPlayer(Option<PlayerId>),
    Tell(Option<(PlayerId, Info)>),
}

struct SinglePlayerIO {
    tui: Rc<RefCell<Option<SinglePlayerTui>>>,
}

impl SinglePlayerIO {
    fn prompt(tui: &mut SinglePlayerTui, player: usize, input: InputTypeId) -> InputType {
        tui.player_input = Some((player, input));

        tui.run()
    }
}
impl GrimIO for SinglePlayerIO {
    fn tell(&mut self, player: PlayerId, info: Info) {
        self.tui.borrow_mut().as_mut().unwrap().player_tabs[player]
            .0
            .push(info);
    }

    fn prompt_player(&mut self, player: PlayerId) -> PlayerId {
        let InputType::Player(x) = Self::prompt(
            self.tui.borrow_mut().as_mut().unwrap(),
            player,
            InputTypeId::Player,
        ) else {
            panic!()
        };
        x
    }

    fn prompt_player_option(&mut self, player: PlayerId) -> Option<PlayerId> {
        let InputType::OptionPlayer(x) = Self::prompt(
            self.tui.borrow_mut().as_mut().unwrap(),
            player,
            InputTypeId::OptionPlayer,
        ) else {
            panic!()
        };
        x
    }

    fn prompt_tell(&mut self, player: PlayerId) -> Option<(PlayerId, Info)> {
        let InputType::Tell(x) = Self::prompt(
            self.tui.borrow_mut().as_mut().unwrap(),
            player,
            InputTypeId::Tell,
        ) else {
            panic!()
        };
        x
    }

    fn win(&mut self, _: bool) -> Never {
        ratatui::restore();
        todo!();
    }
}

pub struct SinglePlayerTui {
    grim: Rc<RefCell<Grimoir<SinglePlayerIO>>>,
    player_tabs: [(Vec<Info>, RefCell<ListState>); 15],
    selected_tab: usize,

    input: String,
    player_input: Option<(usize, InputTypeId)>,

    terminal: RefCell<ratatui::DefaultTerminal>,
}

impl SinglePlayerTui {
    pub fn new() -> Rc<RefCell<Option<Self>>> {
        let prompt_tui = Rc::new(RefCell::new(None));

        // let prompt_player = {
        //     let tui = Rc::clone(&prompt_tui);
        //     Box::new(move |player: usize| {
        //         let InputType::Player(x) = Self::prompt(
        //             tui.borrow_mut().as_mut().unwrap(),
        //             player,
        //             InputTypeId::Player,
        //         ) else {
        //             panic!()
        //         };
        //         x
        //     })
        // };

        // let prompt_player_optional = {
        //     let tui = Rc::clone(&prompt_tui);
        //     Box::new(move |player: usize| {
        //         let InputType::OptionPlayer(x) = Self::prompt(
        //             tui.borrow_mut().as_mut().unwrap(),
        //             player,
        //             InputTypeId::OptionPlayer,
        //         ) else {
        //             panic!()
        //         };
        //         x
        //     })
        // };

        // let prompt_tell = {
        //     let tui = Rc::clone(&prompt_tui);
        //     Box::new(move |player: usize| {
        //
        //     })
        // };

        // let io = GrimIO {
        //     tell,
        //     win: Box::new(Self::win),
        //     prompt_player,
        //     prompt_player_optional,
        //     prompt_tell,
        // };

        let tui = SinglePlayerTui {
            grim: Rc::new(RefCell::new(Grimoir::<SinglePlayerIO>::new(
                Grimoir::<DebugIO>::gen_roles(None),
                None,
                SinglePlayerIO {
                    tui: Rc::clone(&prompt_tui),
                },
            ))),
            player_tabs: Default::default(),
            selected_tab: 0,
            input: String::new(),
            player_input: None,
            terminal: RefCell::new(ratatui::init()),
        };

        *(&mut *prompt_tui.borrow_mut()) = Some(tui);

        prompt_tui
    }

    pub fn init(tui: Rc<RefCell<Option<Self>>>) {
        let grim = Rc::clone(&tui.borrow().as_ref().unwrap().grim);

        assert!(tui.try_borrow().is_ok());

        grim.borrow_mut().first_night();

        loop {
            grim.borrow_mut().day();
            grim.borrow_mut().night();
        }
    }

    fn run(&mut self) -> InputType {
        loop {
            self.terminal
                .borrow_mut()
                .draw(|frame| frame.render_widget(&*self, frame.area()))
                .unwrap();

            let Some(x) = self.handle_events() else {
                continue;
            };

            return x;
        }
    }

    fn handle_events(&mut self) -> Option<InputType> {
        use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Left => {
                        if self.selected_tab == 0 {
                            self.selected_tab = 14
                        } else {
                            self.selected_tab -= 1
                        };
                    }
                    KeyCode::Right => {
                        self.selected_tab = (self.selected_tab + 1) % 15;
                    }
                    KeyCode::Down => self.player_tabs[self.selected_tab]
                        .1
                        .borrow_mut()
                        .select_next(),

                    KeyCode::Up => self.player_tabs[self.selected_tab]
                        .1
                        .borrow_mut()
                        .select_previous(),

                    KeyCode::Char('q') => {
                        ratatui::restore();
                        todo!("Quit");
                    }

                    KeyCode::Char(c @ ('0'..'9' | 'a'..'z' | ' ')) => self.input.push(c),

                    KeyCode::Enter => match self.player_input.as_ref().unwrap().1 {
                        InputTypeId::Player => {
                            let x = self.input.parse().unwrap();

                            self.input = String::new();

                            return Some(InputType::Player(x));
                        }
                        InputTypeId::OptionPlayer => {
                            let x = if self.input.is_empty() {
                                None
                            } else {
                                Some(self.input.parse().unwrap())
                            };

                            self.input = String::new();

                            return Some(InputType::OptionPlayer(x));
                        }
                        InputTypeId::Tell => {
                            /*
                            let x = if true {
                                // self.input.is_empty() {
                                None
                            } else {
                                // num num role
                                let mut chunks = self.input.split_whitespace();

                                let target = chunks.next().unwrap().parse().unwrap();
                                let player = chunks.next().unwrap().parse().unwrap();

                                let role = {
                                    let chunk = chunks.next().unwrap();

                                    let roles: Vec<RoleId> = RoleId::all()
                                        .into_iter()
                                        .map(|x| (x, format!("{x:?}")))
                                        .filter(|(_, x)| {
                                            x.to_lowercase().strip_prefix(chunk).is_some()
                                        })
                                        .map(|x| x.0)
                                        .collect();
                                    // this might filter out perfect matches

                                    if roles.len() > 1 {
                                        self.input.push('?');
                                        return None;
                                    } else if roles.len() == 0 {
                                        self.input = format!("{target} {player} ?");
                                        return None;
                                    } else {
                                        assert_eq!(roles.len(), 1);

                                        roles[0]
                                    }
                                };

                                Some((target, player, role))
                            };

                            */

                            self.input = String::new();

                            // return Some(InputType::Tell(x));
                            return Some(InputType::Tell(None));
                        }
                    },

                    KeyCode::Backspace => {
                        self.input.pop();
                    }

                    _ => {} // do nothing
                }
            }
        }

        None
    }
}

impl Widget for &SinglePlayerTui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::layout::{
            Constraint::{Length, Min},
            Layout,
        };

        let [tabs, inner, prompt] = Layout::vertical([Length(1), Min(0), Length(1)]).areas(area);

        // render tabs
        let titles = (0..15).map(|x| ratatui::text::Line::from(x.to_string()));
        let selected = self.selected_tab;
        Tabs::new(titles).select(selected).render(tabs, buf);

        // render the info given to to player
        let items = self.player_tabs[self.selected_tab]
            .0
            .iter()
            .map(|x| ListItem::from(format!("{x:?}")))
            .collect::<Vec<_>>();

        let list = List::new(items);

        StatefulWidget::render(
            list,
            inner,
            buf,
            &mut self.player_tabs[self.selected_tab].1.borrow_mut(),
        );

        let prompt_text = match self.player_input.as_ref().unwrap() {
            (x, InputTypeId::Player) => format!("Player {x}: {}", self.input),
            (x, InputTypeId::OptionPlayer) => format!("Option Player {x}: {}", self.input),
            (x, InputTypeId::Tell) => format!("Tell Player {x}: {}", self.input),
        };

        Paragraph::new(prompt_text).render(prompt, buf);
    }
}
