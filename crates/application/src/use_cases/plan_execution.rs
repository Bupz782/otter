use crate::services::strategy_planner::{PlannerError, StrategyPlanner};
use domain::models::execution_plan::ExecutionPlan;
use domain::models::intent::{ConditionalIntent, Intent};

pub struct PlanExecutionUseCase {
    planner: StrategyPlanner,
}

#[derive(Debug)]
pub enum PlanExecutionError {
    PlanningFailed(String),
    ValidationFailed(String),
    InvalidIntent(String),
}

impl From<PlannerError> for PlanExecutionError {
    fn from(err: PlannerError) -> Self {
        PlanExecutionError::PlanningFailed(format!("{:?}", err))
    }
}

impl PlanExecutionUseCase {
    pub fn new(planner: StrategyPlanner) -> Self {
        Self { planner }
    }

    /// Plan execution for a simple intent
    pub fn execute(&self, intent: &Intent) -> Result<ExecutionPlan, PlanExecutionError> {
        // 1. Validate intent feasibility
        self.planner
            .validate_plan_feasibility(intent)
            .map_err(|e| PlanExecutionError::ValidationFailed(format!("{:?}", e)))?;

        // 2. Generate execution plan
        let plan = self.planner.plan(intent)?;

        Ok(plan)
    }

    /// Plan execution for a conditional intent
    pub fn execute_conditional(
        &self,
        conditional: &ConditionalIntent,
    ) -> Result<ExecutionPlan, PlanExecutionError> {
        // 1. Validate the intent part
        self.planner
            .validate_plan_feasibility(&conditional.intent)
            .map_err(|e| PlanExecutionError::ValidationFailed(format!("{:?}", e)))?;

        // 2. Plan with condition validation
        let plan = self.planner.plan_conditional(conditional)?;

        Ok(plan)
    }
}
