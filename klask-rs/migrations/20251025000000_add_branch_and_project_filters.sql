-- Add branch and project filtering columns to repositories table
-- These columns support both explicit inclusion/exclusion lists and pattern matching

ALTER TABLE repositories
  ADD COLUMN included_branches TEXT,
  ADD COLUMN included_branches_patterns TEXT,
  ADD COLUMN excluded_branches TEXT,
  ADD COLUMN excluded_branches_patterns TEXT,
  ADD COLUMN included_projects TEXT,
  ADD COLUMN included_projects_patterns TEXT;
