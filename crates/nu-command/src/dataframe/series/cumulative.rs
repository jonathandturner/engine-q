use super::super::values::{Column, NuDataFrame};

use nu_engine::CallExt;
use nu_protocol::{
    ast::Call,
    engine::{Command, EngineState, Stack},
    Category, Example, PipelineData, ShellError, Signature, Span, Spanned, SyntaxShape,
};
use polars::prelude::{DataType, IntoSeries};

enum CumType {
    Min,
    Max,
    Sum,
}

impl CumType {
    fn from_str(roll_type: &str, span: Span) -> Result<Self, ShellError> {
        match roll_type {
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "sum" => Ok(Self::Sum),
            _ => Err(ShellError::SpannedLabeledErrorHelp(
                "Wrong operation".into(),
                "Operation not valid for cumulative".into(),
                span,
                "Allowed values: max, min, sum".into(),
            )),
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            CumType::Min => "cum_min",
            CumType::Max => "cum_max",
            CumType::Sum => "cum_sum",
        }
    }
}

#[derive(Clone)]
pub struct Cumulative;

impl Command for Cumulative {
    fn name(&self) -> &str {
        "df cum"
    }

    fn usage(&self) -> &str {
        "Cumulative calculation for a series"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("type", SyntaxShape::String, "rolling operation")
            .switch("reverse", "Reverse cumulative calculation", Some('r'))
            .category(Category::Custom("dataframe".into()))
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Cumulative sum for a series",
            example: "[1 2 3 4 5] | df to-df | df cum sum",
            result: Some(
                NuDataFrame::try_from_columns(vec![Column::new(
                    "0_cum_sum".to_string(),
                    vec![1.into(), 3.into(), 6.into(), 10.into(), 15.into()],
                )])
                .expect("simple df for test should not fail")
                .into_value(Span::unknown()),
            ),
        }]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        command(engine_state, stack, call, input)
    }
}

fn command(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
    input: PipelineData,
) -> Result<PipelineData, ShellError> {
    let cum_type: Spanned<String> = call.req(engine_state, stack, 0)?;
    let reverse = call.has_flag("reverse");

    let df = NuDataFrame::try_from_pipeline(input, call.head)?;
    let series = df.as_series(call.head)?;

    if let DataType::Object(_) = series.dtype() {
        return Err(ShellError::SpannedLabeledError(
            "Found object series".into(),
            "Series of type object cannot be used for cumulative operation".into(),
            call.head,
        ));
    }

    let cum_type = CumType::from_str(&cum_type.item, cum_type.span)?;
    let mut res = match cum_type {
        CumType::Max => series.cummax(reverse),
        CumType::Min => series.cummin(reverse),
        CumType::Sum => series.cumsum(reverse),
    };

    let name = format!("{}_{}", series.name(), cum_type.to_str());
    res.rename(&name);

    NuDataFrame::try_from_series(vec![res.into_series()], call.head)
        .map(|df| PipelineData::Value(NuDataFrame::into_value(df, call.head), None))
}

#[cfg(test)]
mod test {
    use super::super::super::test_dataframe::test_dataframe;
    use super::*;

    #[test]
    fn test_examples() {
        test_dataframe(Cumulative {})
    }
}
