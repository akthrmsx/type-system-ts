#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Boolean,
    Number,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tm {
    True,
    False,
    If {
        condition: Box<Tm>,
        consequent: Box<Tm>,
        alternative: Box<Tm>,
    },
    Number {
        value: usize,
    },
    Add {
        left: Box<Tm>,
        right: Box<Tm>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub message: String,
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error {
            message: message.into(),
        }
    }
}

pub fn run(tm: &Tm) -> Result<Ty, Error> {
    match tm {
        Tm::True | Tm::False => Ok(Ty::Boolean),
        Tm::If {
            condition,
            consequent,
            alternative,
        } => {
            if run(condition)? != Ty::Boolean {
                return Err("想定外の型です。（想定：boolean）".into());
            }
            match (run(consequent)?, run(alternative)?) {
                (consequent, alternative) if consequent == alternative => Ok(consequent),
                _ => Err("then節とelse節の型が異なります。".into()),
            }
        }
        Tm::Number { .. } => Ok(Ty::Number),
        Tm::Add { left, right } => {
            if run(left)? != Ty::Number {
                return Err("想定外の型です。（想定：number）".into());
            }
            if run(right)? != Ty::Number {
                return Err("想定外の型です。（想定：number）".into());
            }
            Ok(Ty::Number)
        }
    }
}
