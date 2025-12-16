# Feature 057: Interactive Analyze Generation

**Feature Branch**: `057-interactive-analyze-generation`
**Created**: 2024-12-16
**Status**: Draft

## Overview

Interactive TUI workflow for `/speckit.analyze` command. Enables cross-artifact consistency analysis with visual feedback.

## Dependencies

**Depends on:**
- Feature 051 (Interactive Specify Flow) - UI patterns and components
- Feature 052 (Internalize Spec Generation) - Rust implementation patterns

## Problem Statement

Current analysis produces text output that's hard to navigate and act upon.

## User Stories

### As an rstn user
- I want to see analysis results in a structured format
- So that I can quickly identify issues

### As an rstn user
- I want to navigate to problematic artifacts
- So that I can fix inconsistencies

## Requirements

### Functional Requirements

- **FR-1**: Analyze consistency across spec.md, plan.md, tasks.md
- **FR-2**: Display issues in categorized format
- **FR-3**: Allow navigation to source artifacts
- **FR-4**: Track issue resolution status

## Success Criteria

- Analysis results are clearly presented
- User can navigate to issues easily
- Issues can be tracked to resolution
