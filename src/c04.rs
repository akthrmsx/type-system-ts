use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Boolean,
    Number,
    Function {
        parameters: Vec<Parameter>,
        return_ty: Box<Self>,
    },
}

impl Ty {
    pub fn function(parameters: Vec<Parameter>, return_ty: Self) -> Self {
        Self::Function {
            parameters,
            return_ty: Box::new(return_ty),
        }
    }

    fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Function {
                    parameters: parameters1,
                    return_ty: return_ty1,
                },
                Self::Function {
                    parameters: parameters2,
                    return_ty: return_ty2,
                },
            ) => {
                parameters1.len() == parameters2.len()
                    && parameters1
                        .iter()
                        .zip(parameters2.iter())
                        .all(|(parameter1, parameter2)| parameter1.ty.equals(&parameter2.ty))
                    && return_ty1.equals(return_ty2)
            }
            _ => self == other,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tm {
    True,
    False,
    If {
        condition: Box<Self>,
        consequent: Box<Self>,
        alternative: Box<Self>,
    },
    Number {
        value: usize,
    },
    Add {
        left: Box<Self>,
        right: Box<Self>,
    },
    Variable {
        name: String,
    },
    Function {
        parameters: Vec<Parameter>,
        body: Box<Self>,
    },
    Call {
        function: Box<Self>,
        arguments: Vec<Self>,
    },
    Sequence {
        first: Box<Self>,
        second: Box<Self>,
    },
    Let {
        name: String,
        value: Box<Self>,
        next: Box<Self>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub message: String,
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self { message }
    }
}

pub fn run(tm: &Tm, env: &HashMap<String, Ty>) -> Result<Ty, Error> {
    match tm {
        Tm::True | Tm::False => Ok(Ty::Boolean),
        Tm::If {
            condition,
            consequent,
            alternative,
        } => {
            if !run(condition, env)?.equals(&Ty::Boolean) {
                return Err("想定外の型です。（想定：boolean）".into());
            }
            match (run(consequent, env)?, run(alternative, env)?) {
                (consequent, alternative) if consequent.equals(&alternative) => Ok(consequent),
                _ => Err("then節とelse節の型が異なります。".into()),
            }
        }
        Tm::Number { .. } => Ok(Ty::Number),
        Tm::Add { left, right } => {
            if !run(left, env)?.equals(&Ty::Number) {
                return Err("想定外の型です。（想定：number）".into());
            }
            if !run(right, env)?.equals(&Ty::Number) {
                return Err("想定外の型です。（想定：number）".into());
            }
            Ok(Ty::Number)
        }
        Tm::Variable { name } => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("未定義の変数です。（変数：{}）", name).into()),
        Tm::Function { parameters, body } => {
            let mut env = env.clone();
            for parameter in parameters.iter() {
                env.insert(parameter.name.clone(), parameter.ty.clone());
            }
            let return_ty = run(body, &env)?;
            Ok(Ty::function(parameters.clone(), return_ty))
        }
        Tm::Call {
            function,
            arguments,
        } => {
            let (parameters, return_ty) = match run(function, env)? {
                Ty::Function {
                    parameters,
                    return_ty,
                } => (parameters, return_ty),
                _ => return Err("想定外の型です。（想定：function）".into()),
            };
            if arguments.len() != parameters.len() {
                return Err("引数の数が異なります。".into());
            }
            for (argument, parameter) in arguments.iter().zip(parameters.iter()) {
                if !run(argument, env)?.equals(&parameter.ty) {
                    return Err("引数の型が異なります。".into());
                }
            }
            Ok(*return_ty)
        }
        Tm::Sequence { first, second } => {
            run(first, env)?;
            run(second, env)
        }
        Tm::Let { name, value, next } => {
            let value = run(value, env)?;
            let mut env = env.clone();
            env.insert(name.clone(), value);
            run(next, &env)
        }
    }
}
