use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Boolean,
    Number,
    Function {
        parameters: Vec<Parameter>,
        return_ty: Box<Self>,
    },
    Object {
        properties: Vec<TyProperty>,
    },
}

impl Ty {
    pub fn function(parameters: Vec<Parameter>, return_ty: Self) -> Self {
        Self::Function {
            parameters,
            return_ty: Box::new(return_ty),
        }
    }

    pub fn object(properties: Vec<TyProperty>) -> Self {
        Self::Object { properties }
    }

    fn is_subtype(&self, other: &Self) -> bool {
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
                        .all(|(parameter1, parameter2)| parameter2.ty.is_subtype(&parameter1.ty))
                    && return_ty1.is_subtype(return_ty2)
            }
            (
                Self::Object {
                    properties: properties1,
                },
                Self::Object {
                    properties: properties2,
                },
            ) => properties2.iter().all(|property2| {
                properties1.iter().any(|property1| {
                    property1.name == property2.name && property1.ty.is_subtype(&property2.ty)
                })
            }),
            _ => self == other,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TyProperty {
    pub name: String,
    pub ty: Ty,
}

impl TyProperty {
    pub fn new(name: impl Into<String>, ty: Ty) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tm {
    True,
    False,
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
    Object {
        properties: Vec<TmProperty>,
    },
    Get {
        object: Box<Self>,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TmProperty {
    pub name: String,
    pub tm: Tm,
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
        Tm::Number { .. } => Ok(Ty::Number),
        Tm::Add { left, right } => {
            if !run(left, env)?.is_subtype(&Ty::Number) {
                return Err("想定外の型です。（想定：number）".into());
            }
            if !run(right, env)?.is_subtype(&Ty::Number) {
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
                if !run(argument, env)?.is_subtype(&parameter.ty) {
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
        Tm::Object { properties } => {
            let properties = properties
                .iter()
                .map(|property| {
                    run(&property.tm, env).map(|ty| TyProperty::new(property.name.clone(), ty))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Ty::object(properties))
        }
        Tm::Get { object, name } => {
            let properties = match run(object, env)? {
                Ty::Object { properties } => properties,
                _ => return Err("想定外の型です。（想定：object）".into()),
            };
            properties
                .iter()
                .find_map(|property| {
                    if property.name == *name {
                        Some(property.ty.clone())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| format!("未定義のプロパティです。（プロパティ：{}）", name).into())
        }
    }
}
