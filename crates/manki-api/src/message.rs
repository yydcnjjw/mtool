use std::any::Any;

use serde::{Deserialize, Serialize};

fn is_unit(t: &(dyn Any + Send)) -> bool {
    t.is::<()>()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request<Params>
where
    Params: Send + 'static,
{
    pub action: String,
    #[serde(skip_serializing_if = "is_unit")]
    pub params: Params,
    pub version: usize,
}

impl Request<()> {
    pub fn new<Action>(action: Action) -> Request<()>
    where
        Action: Into<String>,
    {
        Request {
            action: action.into(),
            params: (),
            version: 6,
        }
    }
}

impl<Params> Request<Params>
where
    Params: Send + 'static,
{
    pub fn new_with_params<Action>(action: Action, params: Params) -> Request<Params>
    where
        Action: Into<String>,
    {
        Request {
            action: action.into(),
            params,
            version: 6,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<Result>
where
    Result: Send + 'static,
{
    pub result: Result,
    pub error: Option<String>,
}
