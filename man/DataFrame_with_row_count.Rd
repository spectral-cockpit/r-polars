% Generated by roxygen2: do not edit by hand
% Please edit documentation in R/dataframe__frame.R
\name{DataFrame_with_row_count}
\alias{DataFrame_with_row_count}
\title{Eager with_row_count}
\usage{
DataFrame_with_row_count(name, offset = NULL)
}
\arguments{
\item{name}{string name of the created column}

\item{offset}{positive integer offset for the start of the counter}
}
\value{
A new \code{DataFrame} object with a counter column in front
}
\description{
Add a new column at index 0 that counts the rows
}
\keyword{DataFrame}
