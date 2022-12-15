pub trait CursorView {
    type State;
    fn cursor_down(self, state: &mut Self::State);
    fn cursor_up(self, state: &mut Self::State);
}
