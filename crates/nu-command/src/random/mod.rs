mod bool;
mod chars;
mod command;
mod decimal;
mod dice;
mod integer;

pub use self::bool::SubCommand as Bool;
pub use self::chars::SubCommand as Chars;
pub use self::decimal::SubCommand as Decimal;
pub use self::dice::SubCommand as Dice;
pub use self::integer::SubCommand as Integer;
pub use command::RandomCommand as Random;
