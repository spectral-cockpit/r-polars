use crate::utils::r_result_list;
use crate::utils::wrappers::Wrap;
use extendr_api::prelude::*;
use polars::prelude as pl;
use polars_core::prelude::QuantileInterpolOptions;
//expose polars DateType in R
use crate::robj_to;
use crate::rpolarserr::{polars_to_rpolars_err, rerr, RResult, WithRctx};
use crate::utils::collect_hinted_result;
use crate::utils::wrappers::null_to_opt;
use std::result::Result;
#[derive(Debug, Clone, PartialEq)]
pub struct RField(pub pl::Field);
use pl::UniqueKeepStrategy;

#[extendr]
impl RField {
    fn new(name: String, datatype: &RPolarsDataType) -> RField {
        let name = name.into();
        RField(pl::Field {
            name,
            dtype: datatype.0.clone(),
        })
    }

    pub fn print(&self) {
        rprintln!("{:#?}", self.0);
    }

    #[allow(clippy::should_implement_trait)]
    pub fn clone(&self) -> Self {
        RField(self.0.clone())
    }

    pub fn get_name(&self) -> String {
        self.0.name.to_string()
    }
    pub fn get_datatype(&self) -> RPolarsDataType {
        RPolarsDataType(self.0.dtype.clone())
    }

    pub fn set_name_mut(&mut self, name: &str) {
        self.0.name = name.into()
    }

    pub fn set_datatype_mut(&mut self, datatype: &RPolarsDataType) {
        self.0.dtype = datatype.0.clone()
    }

    // pub fn inner_from_robj_clone(robj: &Robj) -> std::result::Result<Self, &'static str> {
    //     if robj.check_external_ptr_type::<RField>() {
    //         let x: RField = unsafe { &mut *robj.external_ptr_addr::<RField>() }.clone();
    //         Ok(x)
    //     } else {
    //         Err("expected RField")
    //     }
    // }

    // pub fn any_robj_to_pl_RField_result(robj: &Robj) -> pl::PolarsResult<pl::RField> {
    //     let s = if !&robj.inherits("RField") {
    //         robjname2series(&robj, &"")?
    //     } else {
    //         Series::inner_from_robj_clone(&robj)
    //             .map_err(|err| {
    //                 //convert any error from R to a polars error
    //                 pl::PolarsError::ComputeError(err.into()))
    //             })?
    //             .0
    //     };
    //     Ok(s)
    // }
}

// impl TryFrom<Robj> for RField {
//     type Error = String;

//     fn try_from(robj: Robj) -> std::result::Result<Self, Self::Error> {
//         if  &robj.inherits("RField") {
//             lrobj.try_into()
//             Ok(EvenNumber(value))
//         } else {
//             Err(())
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct RPolarsDataType(pub pl::DataType);

#[extendr]
impl RPolarsDataType {
    pub fn new(s: &str) -> RPolarsDataType {
        //, inner: Nullable<&DataType>
        //let inner = Box::new(null_to_opt(inner).map_or(pl::DataType::Null, |x| x.0.clone()));

        let pl_datatype = match s {
            "Boolean" | "logical" => pl::DataType::Boolean,
            "UInt8" | "uinteger8" => pl::DataType::UInt8,
            "UInt16" | "uinteger16" => pl::DataType::UInt16,
            "UInt32" | "uinteger32" => pl::DataType::UInt32,
            "UInt64" | "uinteger64" => pl::DataType::UInt64,
            "Int8" | "integer8" => pl::DataType::Int8,
            "Int16" | "integer16" => pl::DataType::Int16,
            "Int32" | "integer32" | "integer" => pl::DataType::Int32,
            "Int64" | "integer64" => pl::DataType::Int64,
            "Float32" | "float32" | "double" => pl::DataType::Float32,
            "Float64" | "float64" => pl::DataType::Float64,

            "Utf8" | "character" => pl::DataType::Utf8,
            "Binary" | "binary" => pl::DataType::Binary,
            "Date" | "date" => pl::DataType::Date,
            "Time" | "time" => pl::DataType::Time,
            "Null" | "null" => pl::DataType::Null,
            "Categorical" | "factor" => pl::DataType::Categorical(None),
            "Unknown" | "unknown" => pl::DataType::Unknown,

            _ => panic!("data type not recgnized "),
        };
        RPolarsDataType(pl_datatype)
    }

    pub fn new_datetime(tu: Robj, tz: Nullable<String>) -> List {
        let result = robj_to_timeunit(tu)
            .map(|dt| RPolarsDataType(pl::DataType::Datetime(dt, null_to_opt(tz))));
        r_result_list(result)
    }

    pub fn new_duration() -> RPolarsDataType {
        todo!("duration not implemented")
    }

    pub fn new_list(inner: &RPolarsDataType) -> RPolarsDataType {
        RPolarsDataType(pl::DataType::List(Box::new(inner.0.clone())))
    }

    pub fn new_object() -> RPolarsDataType {
        todo!("object not implemented")
    }

    pub fn new_struct(l: Robj) -> List {
        let res = || -> std::result::Result<RPolarsDataType, String> {
            let len = l.len();

            //iterate over R list and collect Fields and place in a Struct-datatype
            l.as_list()
                .ok_or_else(|| "argument [l] is not a list".to_string())
                .map(|l| {
                    l.into_iter().enumerate().map(|(i, (name, robj))| {
                        let res: extendr_api::Result<ExternalPtr<RField>> = robj.try_into();
                        res.map_err(|err| {
                            format!(
                                "list element [[{}]] named {} is not a Field: {:?}",
                                i + 1,
                                name,
                                err
                            )
                        })
                        .map(|ok| ok.0.clone())
                    })
                })
                .and_then(|iter| collect_hinted_result(len, iter))
                .map_err(|err| format!("in pl$Struct: {}", err))
                .map(|v_field| RPolarsDataType(pl::DataType::Struct(v_field)))
        }();

        r_result_list(res)
    }

    pub fn get_all_simple_type_names() -> Vec<String> {
        vec![
            "Boolean".into(),
            "UInt8".into(),
            "UInt16".into(),
            "UInt32".into(),
            "UInt64".into(),
            "Int8".into(),
            "Int16".into(),
            "Int32".into(),
            "Int64".into(),
            "Float32".into(),
            "Float64".into(),
            "Utf8".into(),
            "Binary".into(),
            "Date".into(),
            "Time".into(),
            "Null".into(),
            "Categorical".into(),
            "Unknown".into(),
        ]
    }

    pub fn print(&self) {
        rprintln!("{:#?}", self.0);
    }

    #[allow(clippy::should_implement_trait)]
    pub fn eq(&self, other: &RPolarsDataType) -> bool {
        self.0.eq(&other.0)
    }

    pub fn ne(&self, other: &RPolarsDataType) -> bool {
        self.0.ne(&other.0)
    }

    fn same_outer_datatype(&self, other: &RPolarsDataType) -> bool {
        std::mem::discriminant(&self.0) == std::mem::discriminant(&other.0)
    }

    pub fn get_insides(&self) -> List {
        match self.0.clone() {
            pl::DataType::Datetime(tu, tz) => list!(
                tu = tu.to_string().into_robj(),
                tz = if let Some(tz) = tz {
                    tz.into_robj()
                } else {
                    extendr_api::NULL.into_robj()
                },
            ),
            pl::DataType::List(inner) => {
                list!(RPolarsDataType(*inner).into_robj())
            }
            _ => list!(),
        }
    }

    pub fn is_temporal(&self) -> bool {
        self.0.is_temporal()
    }
}

impl From<RPolarsDataType> for pl::DataType {
    fn from(x: RPolarsDataType) -> Self {
        x.0
    }
}

//struct for building a vector of optional named datatype,
//if all named will become a schema and passed to polars_io.csv.csvread.with_dtypes
//if any names are missing will become slice of dtypes and passed to polars_io.csv.csvread.with_dtypes_slice
//zero length vector will neither trigger with_dtypes() or with_dtypes_slice() method calls
#[derive(Debug, Clone, Default)]
pub struct DataTypeVector(pub Vec<(Option<String>, pl::DataType)>);

#[extendr]
impl DataTypeVector {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, colname: Nullable<String>, datatype: &RPolarsDataType) {
        self.0.push((Wrap(colname).into(), datatype.clone().into()));
    }

    pub fn print(&self) {
        rprintln!("{:#?}", self.0);
    }

    pub fn from_rlist(list: List) -> List {
        let mut dtv = DataTypeVector(Vec::with_capacity(list.len()));

        let result: std::result::Result<(), String> = list.iter().try_for_each(|(name, robj)| {
            if !robj.inherits("RPolarsDataType") || robj.rtype() != extendr_api::Rtype::ExternalPtr
            {
                return Err("Internal error: Object is not a RPolarsDataType".into());
            }
            //safety checks class and type before conversion
            let dt: RPolarsDataType =
                unsafe { &mut *robj.external_ptr_addr::<RPolarsDataType>() }.clone();
            let name = extendr_api::Nullable::NotNull(name.to_string());
            dtv.push(name, &dt);
            Ok(())
        });

        r_result_list(result.map(|_| dtv))
    }
}

impl DataTypeVector {
    pub fn dtv_to_vec(&self) -> Vec<pl::DataType> {
        let v: Vec<_> = self.0.iter().map(|(_, dt)| dt.clone()).collect();
        v
    }
}

pub fn new_join_type(s: &str) -> pl::JoinType {
    match s {
        "cross" => pl::JoinType::Cross,
        "inner" => pl::JoinType::Inner,
        "left" => pl::JoinType::Left,
        "outer" => pl::JoinType::Outer,
        "semi" => pl::JoinType::Semi,
        "anti" => pl::JoinType::Anti,
        _ => panic!("polars internal error: jointype not recognized"),
    }
}

pub fn new_asof_strategy(s: &str) -> Result<polars::chunked_array::object::AsofStrategy, String> {
    match s {
        "forward" => Ok(polars::chunked_array::object::AsofStrategy::Forward),
        "backward" => Ok(polars::chunked_array::object::AsofStrategy::Backward),
        _ => Err(format!(
            "asof strategy choice: [{}] is not any of 'forward' or 'backward'",
            s
        )),
    }
}

pub fn new_unique_keep_strategy(s: &str) -> std::result::Result<UniqueKeepStrategy, String> {
    match s {
        // "any" => Ok(pl::UniqueKeepStrategy::Any),
        "first" => Ok(pl::UniqueKeepStrategy::First),
        "last" => Ok(pl::UniqueKeepStrategy::Last),
        "none" => Ok(pl::UniqueKeepStrategy::None),
        _ => Err(format!(
            "keep strategy choice: [{}] is not any of 'any', 'first', 'last', 'none'",
            s
        )),
    }
}

pub fn new_quantile_interpolation_option(
    s: &str,
) -> std::result::Result<QuantileInterpolOptions, String> {
    use pl::QuantileInterpolOptions::*;
    match s {
        "nearest" => Ok(Nearest),
        "higher" => Ok(Higher),
        "lower" => Ok(Lower),
        "midpoint" => Ok(Midpoint),
        "linear" => Ok(Linear),
        _ => Err(format!("interpolation choice: [{}] is not any of 'nearest', 'higher', 'lower', 'midpoint', 'linear'",s))
    }
}

pub fn new_closed_window(s: &str) -> std::result::Result<pl::ClosedWindow, String> {
    use pl::ClosedWindow as CW;
    match s {
        "both" => Ok(CW::Both),
        "left" => Ok(CW::Left),
        "none" => Ok(CW::None),
        "right" => Ok(CW::Right),
        _ => Err(format!(
            "ClosedWindow choice: [{}] is not any of 'both', 'left', 'none' or 'right'",
            s
        )),
    }
}

pub fn new_null_behavior(
    s: &str,
) -> std::result::Result<polars::series::ops::NullBehavior, String> {
    use polars::series::ops::NullBehavior as NB;
    match s {
        "ignore" => Ok(NB::Ignore),
        "drop" => Ok(NB::Drop),

        _ => Err(format!(
            "NullBehavior choice: [{}] is not any of 'drop' or 'ignore'",
            s
        )),
    }
}

pub fn new_rank_method(s: &str) -> std::result::Result<pl::RankMethod, String> {
    use pl::RankMethod as RM;
    let s_low = s.to_lowercase();
    match s_low.as_str() {
        "average" => Ok(RM::Average),
        "dense" => Ok(RM::Dense),
        "max" => Ok(RM::Max),
        "min" => Ok(RM::Min),
        "ordinal" => Ok(RM::Ordinal),
        "random" => Ok(RM::Random),
        _ => Err(format!(
            "RankMethod choice: [{}] is not any 'average','dense', 'min', 'max', 'ordinal', 'random'",
            s_low.as_str()
        )),
    }
}

pub fn literal_to_any_value(
    litval: pl::LiteralValue,
) -> std::result::Result<pl::AnyValue<'static>, String> {
    use pl::AnyValue as av;
    use pl::LiteralValue as lv;
    use smartstring::alias::String as SString;
    match litval {
        lv::Boolean(x) => Ok(av::Boolean(x)),
        //lv::Datetime(datetime, unit) => Ok(av::Datetime(datetime, unit, &None)), #check how to convert
        //lv::Duration(duration, unit) => Ok(av::Duration(duration, unit)), #check how to convert
        lv::Float32(x) => Ok(av::Float32(x)),
        lv::Float64(x) => Ok(av::Float64(x)),
        lv::Int16(x) => Ok(av::Int16(x)),
        lv::Int32(x) => Ok(av::Int32(x)),
        lv::Int64(x) => Ok(av::Int64(x)),
        lv::Int8(x) => Ok(av::Int8(x)),
        lv::Null => Ok(av::Null),
        // lv::Range {
        //     low,
        //     high,
        //     data_type,
        // } => Ok(av::(low, high, data_type)),
        //lv::Series(s) => no counter part
        lv::UInt16(x) => Ok(av::UInt16(x)),
        lv::UInt32(x) => Ok(av::UInt32(x)),
        lv::UInt64(x) => Ok(av::UInt64(x)),
        lv::UInt8(x) => Ok(av::UInt8(x)),
        // lv::Utf8(x) => Ok(av::Utf8(x.as_str())),
        lv::Utf8(x) => {
            let mut s = SString::new();

            s.push_str(x.as_str());
            Ok(av::Utf8Owned(s))
        }
        x => Err(format!("cannot convert LiteralValue {:?} to AnyValue", x)),
    }
}

pub fn expr_to_any_value(e: pl::Expr) -> std::result::Result<pl::AnyValue<'static>, String> {
    use pl::*;
    pl::DataFrame::default()
        .lazy()
        .select(&[e])
        .collect()
        .map_err(|err| err.to_string())?
        .iter()
        .next()
        .ok_or_else(|| String::from("expr made no series"))?
        .iter()
        .next()
        .ok_or_else(|| String::from("series had no first value"))?
        .into_static()
        .map_err(|err| err.to_string())
}

pub fn new_interpolation_method(s: &str) -> std::result::Result<pl::InterpolationMethod, String> {
    use pl::InterpolationMethod as IM;
    match s {
        "linear" => Ok(IM::Linear),
        "nearest" => Ok(IM::Nearest),

        _ => Err(format!(
            "InterpolationMethod choice: [{}] is not any of 'linear' or 'nearest'",
            s
        )),
    }
}

pub fn new_width_strategy(s: &str) -> std::result::Result<pl::ListToStructWidthStrategy, String> {
    use pl::ListToStructWidthStrategy as WS;
    match s {
        "first_non_null" => Ok(WS::FirstNonNull),
        "max_width" => Ok(WS::MaxWidth),

        _ => Err(format!(
            "n_field_strategy: [{}] is not any of 'first_non_null' or 'max_width'",
            s
        )),
    }
}

pub fn robj_to_timeunit(robj: Robj) -> std::result::Result<pl::TimeUnit, String> {
    let s = robj.as_str().ok_or_else(|| {
        format!(
            "Robj must be a string to be matched as TimeUnit, got a [{:?}]",
            robj
        )
    })?;

    match s {
        "ns" => Ok(pl::TimeUnit::Nanoseconds),
        "us" | "μs" => Ok(pl::TimeUnit::Microseconds),
        "ms" => Ok(pl::TimeUnit::Milliseconds),

        _ => Err(format!(
            "str to polars TimeUnit: [{}] is not any of 'ns', 'us/μs' or 'ms' ",
            s
        )),
    }
}

pub fn time_unit_converson(tu: pl::TimeUnit) -> i64 {
    let tu_i64: i64 = match tu {
        pl::TimeUnit::Nanoseconds => 1_000_000_000,
        pl::TimeUnit::Microseconds => 1_000_000,
        pl::TimeUnit::Milliseconds => 1_000,
    };
    tu_i64
}

pub fn new_categorical_ordering(s: &str) -> Result<pl::CategoricalOrdering, String> {
    use pl::CategoricalOrdering as CO;
    match s {
        "physical" => Ok(CO::Physical),
        "lexical" => Ok(CO::Lexical),
        _ => Err(format!(
            "CategoricalOrdering: [{}] is not any of 'physical' or 'lexical'",
            s
        )),
    }
}

pub fn new_parquet_compression(
    compression_method: Robj,
    compression_level: Robj,
) -> RResult<pl::ParquetCompression> {
    use pl::ParquetCompression::*;
    match robj_to!(String, compression_method)?.as_str() {
        "uncompressed" => Ok(Uncompressed),
        "snappy" => Ok(Snappy),
        "gzip" => robj_to!(Option, u8, compression_level)?
            .map(polars::prelude::GzipLevel::try_new)
            .transpose()
            .map(Gzip),
        "lzo" => Ok(Lzo),
        "brotli" => robj_to!(Option, u32, compression_level)?
            .map(polars::prelude::BrotliLevel::try_new)
            .transpose()
            .map(Brotli),
        "zstd" => robj_to!(Option, i32, compression_level)?
            .map(polars::prelude::ZstdLevel::try_new)
            .transpose()
            .map(Zstd),
        m => Err(polars::prelude::PolarsError::ComputeError(
            format!("Failed to set parquet compression method as [{m}]").into(),
        )),
    }
    .map_err(polars_to_rpolars_err)
    .misvalued("should be one of ['uncompressed', 'snappy', 'gzip', 'brotli', 'zstd']")
}

pub fn new_ipc_compression(compression_method: Robj) -> RResult<Option<pl::IpcCompression>> {
    use pl::IpcCompression::*;
    robj_to!(Option, String, compression_method)?
        .map(|cm| match cm.as_str() {
            "lz4" => Ok(LZ4),
            "zstd" => Ok(ZSTD),
            m => rerr()
                .bad_val(m)
                .misvalued("should be one of ['lz4', 'zstd']"),
        })
        .transpose()
}

pub fn new_rolling_cov_options(
    window_size: Robj,
    min_periods: Robj,
    ddof: Robj,
) -> RResult<pl::RollingCovOptions> {
    Ok(pl::RollingCovOptions {
        window_size: robj_to!(u32, window_size)?,
        min_periods: robj_to!(u32, min_periods)?,
        ddof: robj_to!(u8, ddof)?,
    })
}

extendr_module! {
    mod rdatatype;
    impl RPolarsDataType;
    impl DataTypeVector;
    impl RField;
}
