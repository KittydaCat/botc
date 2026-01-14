use std::{cell::RefCell, rc::Rc};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{List, ListItem, ListState, Paragraph, StatefulWidget, Tabs, Widget},
};

use crate::{GrimIO, Grimoir, Info, PlayerId, RoleId, dbg_win};

#[enum_ids::enum_ids]
enum InputType {
    Player(PlayerId),
    OptionPlayer(Option<PlayerId>),
    Tell(Option<(PlayerId, PlayerId, RoleId)>),
}

pub struct Tui {
    grim: Rc<RefCell<Grimoir>>,
    player_tabs: Rc<RefCell<[(Vec<Info>, ListState); 15]>>, // probs list state
    selected_tab: usize,

    input: String,
    player_input: Option<(usize, InputTypeId)>,

    terminal: RefCell<ratatui::DefaultTerminal>,
}

impl Widget for &Tui {
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
        let items = self.player_tabs.borrow()[self.selected_tab]
            .0
            .iter()
            .map(|x| ListItem::from(format!("{x:?}")))
            .collect::<Vec<_>>();

        let list = List::new(items);

        StatefulWidget::render(
            list,
            inner,
            buf,
            &mut self.player_tabs.borrow_mut()[self.selected_tab].1,
        );

        let prompt_text = match self.player_input.as_ref().unwrap() {
            (x, InputTypeId::Player) => format!("Player {x}: {}", self.input),
            (x, InputTypeId::OptionPlayer) => format!("Option Player {x}: {}", self.input),
            (x, InputTypeId::Tell) => format!("Tell Player {x}: {}", self.input),
        };

        Paragraph::new(prompt_text).render(prompt, buf);
    }
}

impl Tui {
    fn prompt(&mut self, player: usize, input: InputTypeId) -> InputType {
        self.player_input = Some((player, input));

        self.run()
    }
    // fn prompt(tui: Rc<RefCell<Option<Self>>>, player: usize, input: InputTypeId) -> InputType {
    //     tui.borrow_mut().as_mut().unwrap().player_input = Some((player, input));

    //     tui.borrow_mut().as_mut().unwrap().run()
    // }

    pub fn new() -> Rc<RefCell<Option<Self>>> {
        let player_tabs: Rc<RefCell<[(Vec<Info>, ListState); 15]>> =
            Rc::new(RefCell::new(Default::default()));

        let tabs = Rc::clone(&player_tabs);

        // might need to be modified to allow stepping
        let tell = Box::new(move |player: usize, info: Info| {
            tabs.borrow_mut()[player].0.push(info);
        });

        let prompt_tui = Rc::new(RefCell::new(None));

        let prompt_player = {
            let tui = Rc::clone(&prompt_tui);
            Box::new(move |player: usize| {
                let InputType::Player(x) = Self::prompt(
                    tui.borrow_mut().as_mut().unwrap(),
                    player,
                    InputTypeId::Player,
                ) else {
                    panic!()
                };
                x
            })
        };

        let prompt_player_optional = {
            let tui = Rc::clone(&prompt_tui);
            Box::new(move |player: usize| {
                let InputType::OptionPlayer(x) = Self::prompt(
                    tui.borrow_mut().as_mut().unwrap(),
                    player,
                    InputTypeId::OptionPlayer,
                ) else {
                    panic!()
                };
                x
            })
        };

        let prompt_tell = {
            let tui = Rc::clone(&prompt_tui);
            Box::new(move |player: usize| {
                let InputType::Tell(x) = Self::prompt(
                    tui.borrow_mut().as_mut().unwrap(),
                    player,
                    InputTypeId::Tell,
                ) else {
                    panic!()
                };
                x
            })
        };

        let io = GrimIO {
            tell,
            win: Box::new(dbg_win),
            prompt_player,
            prompt_player_optional,
            prompt_tell,
        };

        let tui = Tui {
            grim: Rc::new(RefCell::new(Grimoir::new(
                Grimoir::gen_roles(None),
                None,
                io,
            ))),
            player_tabs,
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
                    KeyCode::Down => self.player_tabs.borrow_mut()[self.selected_tab]
                        .1
                        .select_next(),

                    KeyCode::Up => self.player_tabs.borrow_mut()[self.selected_tab]
                        .1
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
                            let x = if self.input.is_empty() {
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

                            self.input = String::new();

                            return Some(InputType::Tell(x));
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
