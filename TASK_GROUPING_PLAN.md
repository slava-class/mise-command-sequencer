# Task Grouping Implementation Plan

## Status: SUPERSEDED BY MIKADO PRE-PLAN
**See TASK_GROUPING_MIKADO_PREPLAN.md for the actual implementation approach**

This file contains the original high-level requirements but lacks the dependency analysis needed for safe implementation.

## Original Overview
Implement automatic task grouping based on colon-separated namespaces in mise task names, with nested group support and global sequence steps.

## Core Requirements
- Parse colon-separated task names into nested groups (e.g., `frontend:build:dev`)
- Display grouped tasks with visual hierarchy
- Maintain global sequence steps (Step 1/2/3) across all tasks regardless of grouping
- Navigation works across all tasks (grouped and ungrouped)
- Groups themselves are navigable and can be renamed, assigned to sequence steps
- No collapsing for now - all groups expanded
- Ungrouped tasks appear after grouped tasks without spacing

## UI Layout Structure
```
┌─ Available Tasks ─────────────────────────────────────────────────────────┐
│ frontend                                                                  │
│   build                                                                   │
│   > dev       │   ●    │        │        │ [run] [cat] [edit]             │
│     prod      │        │        │        │ [run] [cat] [edit]             │
│   test                                                                    │
│     unit      │        │        │   ●    │ [run] [cat] [edit]             │
│ backend                                                                   │
│   api                                                                     │
│     start     │        │        │        │ [run] [cat] [edit]             │
│ build         │        │        │        │ [run] [cat] [edit]             │
│ test          │        │   ●    │        │ [run] [cat] [edit]             │
└───────────────────────────────────────────────────────────────────────────┘
```

## Implementation Steps

### 1. Data Structure Changes
- Modify `MiseTask` to include group path information
- Create new data structures for hierarchical task organization
- Update task list parsing to build group tree

### 2. Task Parsing Logic
- Split task names on `:` to create group hierarchy
- Build tree structure maintaining original task names for execution
- Handle edge cases (empty segments, single tasks)

### 3. UI Rendering Updates
- Modify `sequence_builder.rs` to render nested groups
- Add indentation logic for group levels
- Ensure sequence step columns align properly across all rows
- Update navigation to work across grouped structure

### 4. Navigation Updates
- Maintain flat navigation index for keyboard controls
- Map visual positions to actual executable tasks
- Ensure up/down arrows work seamlessly across group boundaries

### 5. Interaction Preservation
- Keep all existing keyboard shortcuts (1/2/3 for steps, x for run, etc.)
- Ensure mouse interactions work with new layout
- Maintain button positioning and click targets
- When hovering over actions of a nested task, highlight the parents of the task as well, also in green.

## Technical Considerations

### Data Structures
- Need tree structure for groups but flat list for navigation
- Consider using both structures simultaneously
- Group headers are display-only, not executable tasks

### Rendering Logic
- Group headers don't have sequence step controls or action buttons
- Only leaf tasks (actual mise tasks) have full controls
- Indentation levels need careful calculation
- Ensure consistent column alignment

### Edge Cases
- Tasks with no colons (ungrouped)
- Single task in a group
- Deep nesting levels
- Empty group names

## Files to Modify
- `src/models/mise_task.rs` - Add group structure support
- `src/ui/sequence_builder.rs` - Update rendering logic
- `src/app/event_handlers.rs` - Update navigation handling
- `src/mise/client.rs` - Update task parsing if needed

## Future Enhancements (not in scope)
- Group collapsing/expanding
- Group renaming
- Custom group ordering
- Group-level operations