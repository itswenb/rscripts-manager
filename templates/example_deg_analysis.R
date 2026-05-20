#' @title DEG Analysis
#' @description Differential expression analysis using DESeq2
#' @param method character "DESeq2" Analysis method
#' @param pvalue_cutoff number 0.05 P-value threshold
#' @param log2fc_cutoff number 1.0 Log2 fold-change threshold
#' @input counts_matrix.csv Raw count matrix (genes x samples)
#' @input sample_info.csv Sample metadata with condition column
#' @output deg_results.csv Differential expression results table
#' @output volcano_plot.png Volcano plot

source("rflow.R")

params <- rflow_params()
inputs <- rflow_inputs()
out_dir <- rflow_output_dir()

# Read input files
counts <- read.csv(inputs[["counts_matrix.csv"]])
sample_info <- read.csv(inputs[["sample_info.csv"]])

# Parameters with defaults
pval <- as.numeric(params$pvalue_cutoff %||% 0.05)
lfc <- as.numeric(params$log2fc_cutoff %||% 1.0)

# --- Analysis ---
library(DESeq2)

dds <- DESeqDataSetFromMatrix(
  countData = counts,
  colData = sample_info,
  design = ~ condition
)
dds <- DESeq(dds)
res <- results(dds, alpha = pval)
res_df <- as.data.frame(res)
res_df$gene <- rownames(res_df)
res_df$significant <- abs(res_df$log2FoldChange) >= lfc & res_df$padj < pval

# Save results
write.csv(res_df, file.path(out_dir, "deg_results.csv"), row.names = FALSE)

# Volcano plot
png(file.path(out_dir, "volcano_plot.png"), width = 800, height = 600)
plot(res_df$log2FoldChange, -log10(res_df$padj),
     pch = 20, col = ifelse(res_df$significant, "red", "grey"),
     xlab = "Log2 Fold Change", ylab = "-Log10 Adjusted P-value",
     main = "Volcano Plot")
abline(h = -log10(pval), lty = 2)
abline(v = c(-lfc, lfc), lty = 2)
dev.off()

cat("DEG analysis complete.", sum(res_df$significant, na.rm = TRUE), "significant genes found.\n")
