% Generated by roxygen2: do not edit by hand
% Please edit documentation in R/lazyframe__lazy.R
\name{LazyFrame_collect}
\alias{LazyFrame_collect}
\title{New DataFrame from LazyFrame_object$collect()}
\usage{
LazyFrame_collect(
  type_coercion = TRUE,
  predicate_pushdown = TRUE,
  projection_pushdown = TRUE,
  simplify_expression = TRUE,
  slice_pushdown = TRUE,
  common_subplan_elimination = TRUE,
  no_optimization = FALSE,
  streaming = FALSE,
  collect_in_background = FALSE
)
}
\arguments{
\item{type_coercion}{Boolean. Coerce types such that operations succeed and
run on minimal required memory.}

\item{predicate_pushdown}{Boolean. Applies filters as early as possible at
scan level.}

\item{projection_pushdown}{Boolean. Select only the columns that are needed at the scan level.}

\item{simplify_expression}{Boolean. Various optimizations, such as constant folding
and replacing expensive operations with faster alternatives.}

\item{slice_pushdown}{Boolean. Only load the required slice from the scan
Don't materialize sliced outputs
level. Don't materialize sliced outputs (e.g. \code{join$head(10)}).}

\item{common_subplan_elimination}{Boolean. Cache subtrees/file scans that
are used by multiple subtrees in the query plan.}

\item{no_optimization}{Boolean. Turn off the following optimizations:
predicate_pushdown = FALSE
projection_pushdown = FALSE
slice_pushdown = FALSE
common_subplan_elimination = FALSE}

\item{streaming}{Boolean. Run parts of the query in a streaming fashion
(this is in an alpha state).}

\item{collect_in_background}{Boolean. Detach this query from R session.
Computation will start in background. Get a handle which later can be converted
into the resulting DataFrame. Useful in interactive mode to not lock R session.}
}
\value{
A \code{DataFrame}
}
\description{
collect DataFrame by lazy query
}
\details{
Note: use \verb{$fetch(n)} if you want to run your query on the first \code{n} rows only.
This can be a huge time saver in debugging queries.
}
\examples{
pl$LazyFrame(iris)$filter(pl$col("Species") == "setosa")$collect()
}
\seealso{
\itemize{
\item \code{\link[=LazyFrame_sink_parquet]{$sink_parquet()}} stream query to a parquet file.
\item \code{\link[=LazyFrame_sink_ipc]{$sink_ipc()}} stream query to a arrow file.
}
}
\keyword{DataFrame_new}
\keyword{LazyFrame}
