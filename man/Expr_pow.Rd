% Generated by roxygen2: do not edit by hand
% Please edit documentation in R/expr__expr.R
\name{Expr_pow}
\alias{Expr_pow}
\alias{pow}
\title{Exponentiation}
\usage{
Expr_pow(exponent)
}
\arguments{
\item{exponent}{exponent}
}
\value{
Expr
}
\description{
Raise expression to the power of exponent.
}
\details{
The R interpreter will replace the \verb{**} with \code{^}, such that \verb{**} means \code{^} (except in
strings e.g. "**"). Read further at \code{?"**"}. In py-polars python \code{^} is the XOR operator and
\verb{**} is the exponentiation operator.
}
\examples{
# use via `pow`-method and the `^`-operator
pl$DataFrame(a = -1:3)$select(
  pl$lit(2)$pow(pl$col("a"))$alias("with $pow()"),
  2^pl$lit(-2:2), # brief use
  pl$lit(2)$alias("left hand side name")^pl$lit(-3:1)$alias("right hand side name dropped")
)

# exotic case where '**' will not work, but "^" will
safe_chr = \(...) tryCatch(..., error = as.character)
get("^")(2, pl$lit(2)) |> safe_chr()
get("**")(2, pl$lit(2)) |> safe_chr()
get("**")(2, 2) |> safe_chr()
}
\keyword{Expr}
