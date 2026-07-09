//! 数据模型

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoteChoice {
    Approve,
    Oppose,
    Abstain,
}

impl VoteChoice {
    pub fn label(self) -> &'static str {
        match self {
            VoteChoice::Approve => "赞成",
            VoteChoice::Oppose => "不赞成",
            VoteChoice::Abstain => "弃权",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub id: u32,
    pub name: String,
    pub approve: u32,
    pub oppose: u32,
    pub abstain: u32,
}

impl Candidate {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            approve: 0,
            oppose: 0,
            abstain: 0,
        }
    }
}

/// 所有需要持久化的数据（去掉 round）
#[derive(Serialize, Deserialize, Clone)]
pub struct SaveData {
    pub candidates: Vec<Candidate>,
}

impl SaveData {
    pub fn new(candidates: Vec<Candidate>) -> Self {
        Self { candidates }
    }
}
