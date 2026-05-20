# Rflow R Helper
# source() this file at the top of your scripts to access params, inputs, and output_dir.

rflow_dir <- Sys.getenv("RFLOW_RUN_DIR", getwd())

rflow_params <- function() {
  f <- file.path(rflow_dir, "params.json")
  if (file.exists(f)) jsonlite::fromJSON(f) else list()
}

rflow_inputs <- function() {
  f <- file.path(rflow_dir, "inputs.json")
  if (file.exists(f)) jsonlite::fromJSON(f) else list()
}

rflow_output_dir <- function() {
  d <- file.path(rflow_dir, "outputs")
  if (!dir.exists(d)) dir.create(d, recursive = TRUE)
  d
}

rflow_save <- function(obj, filename) {
  path <- file.path(rflow_output_dir(), filename)
  if (grepl("\\.(rds|RDS)$", filename)) {
    saveRDS(obj, path)
  } else if (grepl("\\.(csv|CSV)$", filename)) {
    write.csv(obj, path, row.names = FALSE)
  } else if (grepl("\\.(tsv|TSV)$", filename)) {
    write.table(obj, path, sep = "\t", row.names = FALSE)
  } else {
    saveRDS(obj, path)
  }
  invisible(path)
}

rflow_save_plot <- function(filename, width = 8, height = 6, ...) {
  path <- file.path(rflow_output_dir(), filename)
  if (grepl("\\.pdf$", filename)) {
    pdf(path, width = width, height = height, ...)
  } else {
    png(path, width = width * 100, height = height * 100, res = 100, ...)
  }
  path
}
