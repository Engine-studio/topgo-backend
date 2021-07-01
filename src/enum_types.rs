#[derive(DbEnum)]
pub enum PayMethod {
    cash,  // All variants must be fieldless
    card,
    already_payed,
}
