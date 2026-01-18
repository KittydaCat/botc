use crate::game::{GrimIO, Info, PlayerId, RoleId};

trait BotcAI {
    fn tell(&mut self, info: Info);
    fn prompt_player(&mut self) -> PlayerId;
    fn prompt_player_option(&mut self) -> Option<PlayerId>;
    fn prompt_tell(&mut self) -> Option<(PlayerId, Info)>;
}

impl<AI: BotcAI> GrimIO for &mut [AI; 15] {
    fn tell(&mut self, id: PlayerId, info: Info) {
        self[id].tell(info);
    }

    fn prompt_player(&mut self, id: PlayerId) -> PlayerId {
        self[id].prompt_player()
    }

    fn prompt_player_option(&mut self, id: PlayerId) -> Option<PlayerId> {
        self[id].prompt_player_option()
    }

    fn prompt_tell(&mut self, id: PlayerId) -> Option<(PlayerId, Info)> {
        self[id].prompt_tell()
    }

    fn win(&mut self, _: bool) -> crate::game::Never {
        todo!()
    }
}

enum Logic {}
