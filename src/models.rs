//! 数据模型（候选人、投票记录）

use serde::{Deserialize, Serialize};

/// 单个候选人的得票记录
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

/// 单轮投票的选择
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum VoteChoice {
    Approve,  // 赞成
    Oppose,   // 不赞成
    Abstain,  // 弃权
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

/// 所有需要持久化的数据
#[derive(Serialize, Deserialize, Clone)]
pub struct SaveData {
    pub candidates: Vec<Candidate>,
    pub round: u32, // 已完成投票轮次
}

impl SaveData {
    pub fn new(candidates: Vec<Candidate>) -> Self {
        Self {
            candidates,
            round: 0,
        }
    }
}
