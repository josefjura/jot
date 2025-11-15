# Note Search Command

The `note search` command provides a unified interface for finding and listing notes with flexible output formats and filtering options.

## Basic Usage

```bash
# List all notes
jot note search

# Search notes containing term
jot note search "search term"

# Search with filters
jot note search --tag meeting
jot note search --date "last week"
jot note search "term" --tag meeting --date "last week"
```

## Output Formats

### Pretty Output (Default)
```bash
jot note search
```
```
📝 Mar 16  [meeting, project-x]
First few lines of content...

📝 Mar 15  [ideas, project-x]
Initial thoughts on...
```

### Plain Output
```bash
jot note search --output plain
```
```
1    2024-03-16    [meeting,project-x]    First few lines of content...
2    2024-03-15    [ideas,project-x]    Initial thoughts on...
```
Suitable for piping to tools like grep or fzf:
```bash
jot note search --output plain | grep "todo"
jot note search --output plain | fzf
```

### JSON Output
```bash
jot note search --output json
```
```json
{
  "notes": [
    {
      "id": 1,
      "date": "2024-03-16",
      "tags": ["meeting", "project-x"],
      "content": "First few lines of content..."
    }
  ]
}
```

## Content Display Control

Control how much of the note content is downloaded and displayed:

```bash
# Set number of lines to display (default: full content)
jot note search --lines 5

# Download and display first line only
jot note search --lines 1
```

Note: The --lines setting affects search capability. Only downloaded content can be searched client-side.

## Filter Options

### Tags
```bash
jot note search --tag meeting
jot note search --tag "meeting,project-x"  # Multiple tags
```

### Date
```bash
jot note search --date today
jot note search --date "last week"
jot note search --date "2024-03-16"
```

### Combined Filters
```bash
jot note search "term" --tag meeting --date "last week" --output plain
```

## Usage Examples

```bash
# Find recent meeting notes
jot note search --tag meeting --date "this week"

# Search within today's project notes
jot note search --tag project-x --date today --output plain | fzf

# Export filtered notes as JSON
jot note search --tag important --date "last month" --output json > notes.json

# Search within first line of notes
jot note search --tag meeting --lines 1 | grep "action item"
```