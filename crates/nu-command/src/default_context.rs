use nu_protocol::engine::{EngineState, StateWorkingSet};

use crate::*;

pub fn create_default_context() -> EngineState {
    let mut engine_state = EngineState::new();

    let delta = {
        let mut working_set = StateWorkingSet::new(&engine_state);

        macro_rules! bind_command {
            ( $( $command:expr ),* $(,)? ) => {
                $( working_set.add_decl(Box::new($command)); )*
            };
        }

        // If there are commands that have the same name as default declarations,
        // they have to be registered before the main declarations. This helps to make
        // them only accessible if the correct input value category is used with the
        // declaration
        #[cfg(feature = "dataframe")]
        add_dataframe_decls(&mut working_set);

        // Core
        bind_command! {
            Alias,
            Debug,
            Def,
            Describe,
            Do,
            Echo,
            ExportCommand,
            ExportDef,
            ExportEnv,
            For,
            Help,
            Hide,
            If,
            Let,
            Module,
            Source,
            Use,
            Version,
        };

        // Filters
        bind_command! {
            All,
            Any,
            Append,
            Collect,
            Drop,
            Each,
            First,
            Flatten,
            Get,
            Last,
            Length,
            Lines,
            Nth,
            ParEach,
            Prepend,
            Range,
            Reject,
            Reverse,
            Select,
            Shuffle,
            Skip,
            SkipUntil,
            SkipWhile,
            Uniq,
            Update,
            Where,
            Wrap,
            Zip,
        };

        // System
        bind_command! {
            Benchmark,
            External,
            Ps,
            Sys,
        };

        // Strings
        bind_command! {
            BuildString,
            Format,
            Parse,
            Size,
            Split,
            SplitChars,
            SplitColumn,
            SplitRow,
            Str,
            StrCamelCase,
            StrCapitalize,
            StrCollect,
            StrContains,
            StrDowncase,
            StrEndswith,
            StrFindReplace,
            StrIndexOf,
            StrKebabCase,
            StrLength,
            StrLpad,
            StrPascalCase,
            StrReverse,
            StrRpad,
            StrScreamingSnakeCase,
            StrSnakeCase,
            StrStartsWith,
            StrSubstring,
            StrTrim,
            StrUpcase,
        };

        // FileSystem
        bind_command! {
            Cd,
            Cp,
            Ls,
            Mkdir,
            Mv,
            Rm,
            Touch,
        };

        // Platform
        bind_command! {
            Clear,
            Kill,
            Sleep,
        };

        // Date
        bind_command! {
            Date,
            DateFormat,
            DateHumanize,
            DateListTimezones,
            DateNow,
            DateToTable,
            DateToTimezone,
        };

        // Shells
        bind_command! {
            Exit,
        };

        // Formats
        bind_command! {
            From,
            FromCsv,
            FromEml,
            FromIcs,
            FromIni,
            FromJson,
            FromOds,
            FromSsv,
            FromToml,
            FromTsv,
            FromUrl,
            FromVcf,
            FromXlsx,
            FromXml,
            FromYaml,
            FromYml,
            To,
            ToCsv,
            ToHtml,
            ToJson,
            ToMd,
            ToToml,
            ToTsv,
            ToCsv,
            Touch,
            Use,
            Update,
            Where,
            ToUrl,
            ToXml,
            ToYaml,
        };

        // Viewers
        bind_command! {
            Griddle,
            Table,
        };

        // Conversions
        bind_command! {
            Into,
            IntoBinary,
            IntoDatetime,
            IntoDecimal,
            IntoFilesize,
            IntoInt,
            IntoString,
        };

        // Env
        bind_command! {
            LetEnv,
            WithEnv,
        };

        // Math
        bind_command! {
            Math,
            MathAbs,
            MathAvg,
            MathCeil,
            MathEval,
            MathFloor,
            MathMax,
            MathMedian,
            MathMin,
            MathMode,
            MathProduct,
            MathRound,
            MathSqrt,
            MathStddev,
            MathSum,
            MathVariance,
        };

        // Random
        bind_command! {
            Random,
        };

        // Generators
        bind_command! {
            Cal,
        };

        // Hash
        bind_command! {
            Hash,
            HashMd5::default(),
            HashSha256::default(),
        };

        #[cfg(feature = "plugin")]
        bind_command!(Register);

        // This is a WIP proof of concept
        // bind_command!(ListGitBranches, Git, GitCheckout, Source);

        working_set.render()
    };

    let _ = engine_state.merge_delta(delta);

    engine_state
}
