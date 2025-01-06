pub mod boya {
    pub use crate::api::boya::{
        query_course::{BoyaCampus, BoyaCapacity, BoyaCourse, BoyaKind, BoyaTime},
        query_selected::BoyaSelected,
        query_statistic::{BoyaAssessment, BoyaStatistic},
    };
}
pub mod class {
    pub use crate::api::class::{ClassCourse, ClassSchedule};
}
pub mod evaluation {
    pub use crate::api::evaluation::data_struct::{
        EvaluationAnswer, EvaluationForm, EvaluationListItem,
    };
}
pub mod spoc {
    pub use crate::api::spoc::opt::{SpocSchedule, SpocTimeRange, SpocWeek};
}
