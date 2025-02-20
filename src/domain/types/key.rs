#[derive(Clone)]
pub enum Key<PK, SK> {
    Pk(PK),
    Sk(SK),
    Both((PK, SK))
}