#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepAction {
    Approve,
    Call,
    Transfer,
}

pub type Address = String;  // Pour l'instant, on affinera plus tard

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub action: StepAction,
    pub contract: Address,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub steps: Vec<Step>,
}